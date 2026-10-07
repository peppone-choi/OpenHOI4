"""Offline Natural Earth preview bake. Never downloads missing input."""
import argparse
import hashlib
import io
import json
import platform
from pathlib import Path
import struct
import tomllib
import zipfile
import numpy as np
import scipy
from scipy import ndimage
from scipy.spatial import cKDTree
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import dijkstra


def tiff_lzw(data,expected):
    """TIFF MSB LZW with early code-width change, bounded output."""
    table=[bytes([i]) for i in range(256)]+[b'',b''];bits=9;position=0;previous=None;output=bytearray()
    def read(width):
        nonlocal position
        if position+width>len(data)*8:
            raise ValueError('truncated LZW')
        code=0
        for _ in range(width):
            code=(code<<1)|((data[position//8]>>(7-position%8))&1);position+=1
        return code
    while True:
        code=read(bits)
        if code==256:
            table=[bytes([i]) for i in range(256)]+[b'',b''];bits=9;previous=None;continue
        if code==257:
            break
        if code<len(table):entry=table[code]
        elif code==len(table) and previous is not None:entry=previous+previous[:1]
        else:raise ValueError('invalid LZW code')
        output.extend(entry)
        if len(output)>expected:raise ValueError('LZW output overflow')
        if previous is not None and len(table)<4096:
            table.append(previous+entry[:1])
            if len(table)==(1<<bits)-1 and bits<12:bits+=1
        previous=entry
    if len(output)!=expected:raise ValueError('LZW byte length mismatch')
    return bytes(output)


def read_dem(data):
    """Strict classic tiled F32 GeoTIFF reader for fixed NOAA subset only."""
    if len(data)<8 or data[:4]!=b'II*\x00':raise ValueError('expected little-endian classic GeoTIFF')
    offset=struct.unpack_from('<I',data,4)[0]
    if offset+2>len(data):raise ValueError('TIFF IFD out of range')
    count=struct.unpack_from('<H',data,offset)[0];tags={};sizes={1:1,2:1,3:2,4:4,12:8};formats={3:'H',4:'I',12:'d'}
    for i in range(count):
        entry=offset+2+i*12
        if entry+12>len(data):raise ValueError('TIFF entry out of range')
        tag,kind,n,value=struct.unpack_from('<HHII',data,entry)
        if kind not in sizes:raise ValueError('unsupported TIFF field')
        size=n*sizes[kind];start=entry+8 if size<=4 else value
        if start+size>len(data):raise ValueError('TIFF field out of range')
        tags[tag]=struct.unpack_from('<'+formats[kind]*n,data,start) if kind in formats else data[start:start+size]
    get=lambda tag:tags[tag][0]
    if get(258)!=32 or get(339)!=3 or get(277)!=1 or get(317)!=1 or get(259)!=5:raise ValueError('unsupported DEM pixel encoding')
    if get(256)*get(257)>8*1024*1024:raise ValueError('DEM pixel budget exceeded')
    keys=tags[34735]
    if not any(keys[i]==2048 and keys[i+3]==4326 for i in range(4,len(keys),4)):raise ValueError('DEM CRS mismatch')
    if not any(keys[i]==1025 and keys[i+3]==1 for i in range(4,len(keys),4)):raise ValueError('DEM must use PixelIsArea')
    w,h=get(256),get(257);tw,th=get(322),get(323);raster=np.zeros((h,w),dtype=np.float32)
    columns=(w+tw-1)//tw;rows=(h+th-1)//th
    if len(tags[324])!=columns*rows or len(tags[325])!=columns*rows:raise ValueError('DEM tile count mismatch')
    for i,(start,size) in enumerate(zip(tags[324],tags[325])):
        if start+size>len(data):raise ValueError('DEM tile out of range')
        tile=np.frombuffer(tiff_lzw(data[start:start+size],tw*th*4),dtype='<f4').reshape(th,tw)
        y=(i//columns)*th;x=(i%columns)*tw;dy=min(th,h-y);dx=min(tw,w-x);raster[y:y+dy,x:x+dx]=tile[:dy,:dx]
    sx,sy,_=tags[33550];tie=tags[33922];west=tie[3]-tie[0]*sx;north=tie[4]+tie[1]*sy
    extent=[west,north-h*sy,west+w*sx,north]
    if not np.isfinite(raster).all() or (np.abs(raster)>20000).any():raise ValueError('DEM NoData/nonfinite/out-of-range cells')
    return raster,extent


def sha(data):
    return hashlib.sha256(data).hexdigest()


def verify_sources(root):
    manifest=json.loads((root/'source-manifest.json').read_text())
    for source in manifest['sources']:
        path=root/source['file']
        if not path.resolve().is_relative_to(root.resolve()):
            raise ValueError('source path escape')
        data=path.read_bytes()
        if sha(data)!=source['sha256']:
            raise ValueError('source hash mismatch: '+source['file'])
    return manifest


def shapes(path,shape_type):
    """Read Polygon/PolyLine SHP records; no DBF/third-party package."""
    with zipfile.ZipFile(path) as archive:
        names=[n for n in archive.namelist() if n.endswith('.shp')]
        if len(names)!=1:
            raise ValueError('one polygon SHP required')
        data=archive.read(names[0])
    if struct.unpack_from('>i',data)[0]!=9994 or struct.unpack_from('<i',data,32)[0]!=shape_type:
        raise ValueError('invalid SHP type')
    offset=100
    while offset<len(data):
        size=struct.unpack_from('>i',data,offset+4)[0]*2
        record=data[offset+8:offset+8+size];offset+=8+size
        if struct.unpack_from('<i',record)[0]==0:
            continue
        if struct.unpack_from('<i',record)[0]!=shape_type:
            raise ValueError('unexpected shape type')
        parts,count=struct.unpack_from('<ii',record,36)
        starts=list(struct.unpack_from('<'+'i'*parts,record,44))+[count]
        points=np.frombuffer(record,dtype='<f8',count=count*2,offset=44+4*parts).reshape(-1,2)
        yield [points[starts[i]:starts[i+1]] for i in range(parts)]


def polygons(path):
    return shapes(path,5)


def raster_lines(items,width,height):
    mask=np.zeros((height,width),dtype=bool)
    for parts in items:
        for part in parts:
            x=(part[:,0]+180)*width/360;y=(90-part[:,1])*height/180
            for a,b,c,d in zip(x[:-1],y[:-1],x[1:],y[1:]):
                # Dateline crossings split rather than drawing across the world.
                if abs(a-c)>width/2:continue
                steps=int(np.ceil(max(abs(c-a),abs(d-b))))+1
                xx=np.floor(np.linspace(a,c,steps)).astype(int);yy=np.floor(np.linspace(b,d,steps)).astype(int)
                valid=(xx>=0)&(xx<width)&(yy>=0)&(yy<height);mask[yy[valid],xx[valid]]=True
    return mask


def geodesic_labels(kind,k,seeds,river,elevation,options,extent=None):
    """Offline art partition: same-kind graph distance, physical geographic inputs."""
    h,w=kind.shape;mask=kind==k;flat=np.flatnonzero(mask);n=len(flat)
    nodes=np.full(kind.shape,-1,dtype=np.int32);nodes.ravel()[flat]=np.arange(n,dtype=np.int32)
    rises=np.where(np.isfinite(elevation),elevation,0)
    minimum=ndimage.minimum_filter(np.where(np.isfinite(elevation),elevation,np.inf),size=options['ridge_window'])
    ridge=np.where(np.isfinite(elevation),np.maximum(0,rises-minimum)/options['relief_meters'],0)
    parts=[]
    def edges(a,b,yy,dx,dy,valid):
        na=nodes[a];nb=nodes[b];good=(na>=0)&(nb>=0)&valid
        if not np.any(good):return
        west,south,east,north=extent or [-180,-90,180,90]
        lat=np.radians(north-(yy+.5)*(north-south)/h)
        ratio=((east-west)/w)/((north-south)/h) if extent else 1
        metric=np.broadcast_to(np.sqrt((np.cos(lat)*dx*ratio)**2+dy**2),na.shape)
        dem_pair=np.isfinite(elevation[a])&np.isfinite(elevation[b])
        cost=metric*(1+options['river_crossing_penalty']*(river[a]|river[b])*(k==1)+dem_pair*np.abs(rises[a]-rises[b])/options['relief_meters']+dem_pair*options['ridge_penalty']*np.maximum(ridge[a],ridge[b]))
        parts.append((na[good],nb[good],cost[good]))
    edges((slice(None),slice(None,-1)),(slice(None),slice(1,None)),np.arange(h)[:,None],1,0,True)
    edges((slice(None,-1),slice(None)),(slice(1,None),slice(None)),np.arange(h-1)[:,None],0,1,True)
    corner=(kind[:-1,1:]==k)&(kind[1:,:-1]==k)
    edges((slice(None,-1),slice(None,-1)),(slice(1,None),slice(1,None)),np.arange(h-1)[:,None],1,1,corner)
    corner=(kind[:-1,:-1]==k)&(kind[1:,1:]==k)
    edges((slice(None,-1),slice(1,None)),(slice(1,None),slice(None,-1)),np.arange(h-1)[:,None],1,1,corner)
    if extent is None or extent[2]-extent[0]==360:
        edges((slice(None),-1),(slice(None),0),np.arange(h),1,0,True)
    graph=csr_matrix((np.concatenate([p[2] for p in parts]),(np.concatenate([p[0] for p in parts]),np.concatenate([p[1] for p in parts]))),shape=(n,n)) if parts else csr_matrix((n,n),dtype=float)
    del parts
    source_nodes=nodes[seeds[:,0],seeds[:,1]]
    if (source_nodes<0).any() or len(np.unique(source_nodes))!=len(source_nodes):raise ValueError('invalid geographic seed')
    _,_,origins=dijkstra(graph,directed=False,indices=source_nodes,min_only=True,return_predecessors=True)
    labels=np.full(n,-1,dtype=np.int32);labels[source_nodes]=np.arange(len(seeds),dtype=np.int32)
    output=np.full(kind.shape,-1,dtype=np.int32);reachable=origins>=0
    output.ravel()[flat[reachable]]=labels[origins[reachable]]
    return output


def rasterize(items,width,height,extent=None):
    """Even-odd fill at cell centers; rings retain polygon holes."""
    mask=np.zeros((height,width),dtype=bool)
    west,south,east,north=extent or [-180,-90,180,90]
    for rings in items:
        all_points=np.concatenate(rings)
        px=(all_points[:,0]-west)*width/(east-west)
        py=(north-all_points[:,1])*height/(north-south)
        x0=max(0,int(np.floor(px.min())));x1=min(width,int(np.ceil(px.max()))+1)
        y0=max(0,int(np.floor(py.min())));y1=min(height,int(np.ceil(py.max()))+1)
        if x1<=x0 or y1<=y0:continue
        patch=np.zeros((y1-y0,x1-x0),dtype=bool)
        for ring in rings:
            p=np.array(ring,dtype=float);x=(p[:,0]-west)*width/(east-west);y=(north-p[:,1])*height/(north-south)
            xa=x[:-1];xb=x[1:];ya=y[:-1];yb=y[1:]
            start=max(y0,int(np.ceil(y.min()-.5)));end=min(y1,int(np.ceil(y.max()-.5)))
            for row in range(start,end):
                center=row+.5;cross=(ya>center)!=(yb>center)
                intersections=np.sort(xa[cross]+(center-ya[cross])*(xb[cross]-xa[cross])/(yb[cross]-ya[cross]))
                if len(intersections)%2:
                    raise ValueError('odd ring intersections')
                for left,right in intersections.reshape(-1,2):
                    a=max(x0,int(np.ceil(left-.5)));b=min(x1,int(np.ceil(right-.5)))
                    if b>a:
                        patch[row-y0,a-x0:b-x0]^=True
        mask[y0:y1,x0:x1]|=patch
    return mask


def sphere(x,y,width,height):
    lon=np.radians((x+.5)*360/width-180);lat=np.radians(90-(y+.5)*180/height)
    return np.column_stack((np.cos(lat)*np.cos(lon),np.cos(lat)*np.sin(lon),np.sin(lat)))


def city_points(path):
    """Read point coordinates only; no city names, countries or DBF values."""
    with zipfile.ZipFile(path) as archive:
        names=[n for n in archive.namelist() if n.endswith('.shp')]
        if len(names)!=1:raise ValueError('one point SHP required')
        data=archive.read(names[0])
    if len(data)<100 or struct.unpack_from('>i',data)[0]!=9994 or struct.unpack_from('<i',data,32)[0]!=1:raise ValueError('invalid point SHP')
    offset=100;points=[]
    while offset<len(data):
        if offset+8>len(data):raise ValueError('truncated point record')
        size=struct.unpack_from('>i',data,offset+4)[0]*2;record=data[offset+8:offset+8+size];offset+=8+size
        if size!=20 or len(record)!=20 or struct.unpack_from('<i',record)[0]!=1:raise ValueError('invalid city point record')
        lon,lat=struct.unpack_from('<dd',record,4)
        if not np.isfinite([lon,lat]).all() or abs(lon)>180 or abs(lat)>90:raise ValueError('invalid city coordinate')
        points.append((lon,lat))
    return np.unique(np.array(points),axis=0)


def lonlat_sphere(lon,lat):
    lon=np.radians(lon);lat=np.radians(lat)
    return np.column_stack((np.cos(lat)*np.cos(lon),np.cos(lat)*np.sin(lon),np.sin(lat)))


def city_distances(cities,extent,width,height,radius_km):
    if len(cities)==0:return np.full((height,width),np.inf)
    west,south,east,north=extent
    yy,xx=np.indices((height,width));lon=west+(xx.ravel()+.5)*(east-west)/width;lat=north-(yy.ravel()+.5)*(north-south)/height
    distance,_=cKDTree(lonlat_sphere(cities[:,0],cities[:,1])).query(lonlat_sphere(lon,lat),workers=1)
    return (2*np.arcsin(np.minimum(1,distance/2))*radius_km).reshape(height,width)


def dem_grid(dems,width,height,extent=None):
    west,south,east,north=extent or [-180,-90,180,90]
    output=np.full((height,width),np.nan);lat=north-(np.arange(height)+.5)*(north-south)/height;lon=west+(np.arange(width)+.5)*(east-west)/width
    for raster,box in dems:
        dw,ds,de,dn=box;yy=np.flatnonzero((lat>=ds)&(lat<dn));xx=np.flatnonzero((lon>=dw)&(lon<de))
        ry=((dn-lat[yy])/(dn-ds)*raster.shape[0]).astype(int);rx=((lon[xx]-dw)/(de-dw)*raster.shape[1]).astype(int)
        output[np.ix_(yy,xx)]=raster[np.ix_(ry,rx)]
    return output


def elevation_at(dems,lon,lat):
    value=None
    for raster,(west,south,east,north) in dems:
        if west<=lon<east and south<=lat<north:
            y=int((north-lat)/(north-south)*raster.shape[0]);x=int((lon-west)/(east-west)*raster.shape[1]);value=round(float(raster[y,x]))
    return value


def regional_refine(raw,kind,rivers,cities,dems,definitions,seed):
    h,w=kind.shape;next_id=int(raw.max())+1;reports=[]
    for region in definitions.get('density_regions',[]):
        west,south,east,north=region['extent'];x0=max(0,int(np.ceil((west+180)*w/360-.5)));x1=min(w,int(np.ceil((east+180)*w/360-.5)))
        y0=max(0,int(np.ceil((90-north)*h/180-.5)));y1=min(h,int(np.ceil((90-south)*h/180-.5)))
        box=(slice(y0,y1),slice(x0,x1));local=kind[box];hh,ww=local.shape
        if min(hh,ww)<3:raise ValueError('density region too small')
        extent=[x0*360/w-180,90-y1*180/h,x1*360/w-180,90-y0*180/h]
        actual_cities=cities[(cities[:,0]>=west)&(cities[:,0]<east)&(cities[:,1]>=south)&(cities[:,1]<north)]
        distance=city_distances(actual_cities,extent,ww,hh,definitions['quality']['earth_radius_km']);heights=dem_grid(dems,ww,hh,extent)
        ys,xs=np.nonzero((local==1)&(np.indices(local.shape)[0]>0)&(np.indices(local.shape)[0]<hh-1)&(np.indices(local.shape)[1]>0)&(np.indices(local.shape)[1]<ww-1))
        weights=1+region['city_density']*np.exp(-(distance[ys,xs]/region['city_radius_km'])**2)
        river_distance=ndimage.distance_transform_edt(~rivers[box]);weights*=1+definitions['partition']['river_density']/(1+river_distance[ys,xs])
        interior=ndimage.binary_erosion(np.isfinite(heights));gradient=np.hypot(*np.gradient(np.where(np.isfinite(heights),heights,0)))
        weights*=1+np.minimum(definitions['partition']['relief_density_cap'],gradient[ys,xs]/definitions['partition']['relief_meters'])*interior[ys,xs]
        weights/=weights.sum();rng=np.random.Generator(np.random.PCG64(seed+len(reports)+1));chosen=rng.choice(len(ys),size=min(region['additional_land_seeds'],len(ys)),replace=False,p=weights)
        boundary=np.zeros(local.shape,dtype=bool);boundary[[0,-1],:]=True;boundary[:,[0,-1]]=True;by,bx=np.nonzero(boundary&(local==1))
        seeds=np.concatenate((np.column_stack((by,bx)),np.column_stack((ys[chosen],xs[chosen]))))
        # Existing boundary IDs remain anchors; new source-backed seeds fill interior.
        ids=np.concatenate((raw[box][by,bx],np.arange(next_id,next_id+len(chosen),dtype=np.int32)));next_id+=len(chosen)
        if len(seeds):
            labels=geodesic_labels(local,1,seeds,rivers[box],heights,definitions['partition'],extent=extent)
            valid=labels>=0;raw[box][valid]=ids[labels[valid]]
        reports.append({'id':region['id'],'extent':extent,'width':ww,'height':hh,'source_city_points':len(actual_cities),'added_seeds':len(chosen),'boundary_anchors':len(by)})
        print('region '+json.dumps(reports[-1]),flush=True)
    return raw,reports


def refine_coast(coarse,coarse_kind,fine_kind):
    h,w=fine_kind.shape;ch,cw=coarse.shape
    if h%ch or w%cw:raise ValueError('fine coast must use integer scale')
    sy=h//ch;sx=w//cw;output=np.repeat(np.repeat(coarse,sy,0),sx,1)
    inherited=np.repeat(np.repeat(coarse_kind,sy,0),sx,1)
    for k in (0,1,2):
        mismatch=(fine_kind==k)&(inherited!=k)
        if not mismatch.any():continue
        if not np.any(inherited==k):raise ValueError('fine coast has missing coarse kind')
        y,x=np.nonzero(mismatch)
        # nearest cell assignment only for actual high-res source coast slivers.
        nearest=ndimage.distance_transform_edt(inherited!=k,return_distances=False,return_indices=True)
        # Read immutable coarse labels: earlier kind passes mutate output.
        output[y,x]=coarse[nearest[0,y,x]//sy,nearest[1,y,x]//sx]
        del nearest
    return output


def partition(kind,options,river=None,elevation=None,geographic_options=None):
    height,width=kind.shape;rng=np.random.Generator(np.random.PCG64(options['seed']))
    raw=np.zeros(kind.shape,dtype=np.int32);offset=0
    for k,name in enumerate(['sea','land','lake']):
        ys,xs=np.nonzero(kind==k)
        count=min(options[name+'_seeds'],len(ys))
        if not count:
            continue
        weights=np.cos(np.radians(90-(ys+.5)*180/height))
        if k==1 and geographic_options:
            distance=ndimage.distance_transform_edt(~river)
            weights*=1+geographic_options['river_density']/(1+distance[ys,xs])
            heights=np.where(np.isfinite(elevation),elevation,0);slope=np.hypot(*np.gradient(heights))
            interior=ndimage.binary_erosion(np.isfinite(elevation))
            weights*=1+np.minimum(geographic_options['relief_density_cap'],slope[ys,xs]/geographic_options['relief_meters'])*interior[ys,xs]
        weights/=weights.sum()
        chosen=rng.choice(len(ys),size=count,replace=False,p=weights)
        tree=cKDTree(sphere(xs[chosen],ys[chosen],width,height))
        for row in range(0,height,options['query_rows']):
            y,x=np.nonzero(kind[row:row+options['query_rows']]==k);y+=row
            if len(y):
                _,nearest=tree.query(sphere(x,y,width,height),workers=1)
                raw[y,x]=nearest+offset
        if geographic_options and k in (0,1):
            print('geodesic '+name,flush=True)
            mapped=geodesic_labels(kind,k,np.column_stack((ys[chosen],xs[chosen])),river,elevation,geographic_options)
            reached=mapped>=0;raw[reached]=mapped[reached]+offset
        offset+=count
    return raw


def connected_ids(raw):
    output=np.zeros(raw.shape,dtype=np.uint16);next_id=0
    boxes=ndimage.find_objects(raw+1)
    for seed,box in enumerate(boxes):
        if box is None:
            continue
        parts,count=ndimage.label(raw[box]==seed)
        for component in range(1,count+1):
            if next_id>=65535:
                raise ValueError('u16 ID overflow')
            output[box][parts==component]=next_id;next_id+=1
    return output


def adjacency(index):
    pairs=[]
    for a,b in [(index[:,:-1],index[:,1:]),(index[:-1],index[1:]),(index[:,-1],index[:,0])]:
        changed=a!=b
        pair=np.sort(np.column_stack((a[changed],b[changed])).astype(np.uint32)+1,axis=1)
        pairs.append(pair)
    return [tuple(map(int,p)) for p in np.unique(np.concatenate(pairs),axis=0)]


def index_bytes(index):
    return index.astype('<u2').tobytes(order='C')


def classify_mask(land,lake):
    kind=np.zeros(land.shape,dtype=np.uint8);kind[land]=1;kind[lake]=2
    return kind


def write_json(path,value):
    data=(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n').encode()
    path.write_bytes(data)
    return sha(data)


def build(root,out):
    sources=verify_sources(root);definitions=tomllib.loads((root/'defines.toml').read_text())
    options=definitions['generation'];w=options['width'];h=options['height']
    source={s['id']:root/s['file'] for s in sources['sources']}
    quality=definitions.get('quality');gw=quality['graph_width'] if quality else w;gh=quality['graph_height'] if quality else h
    if w>8192 or h>8192 or w<1 or h<1 or w*h>32*1024*1024:raise ValueError('preview raster budget/texture upper limit')
    dems=[read_dem(path.read_bytes()) for name,path in source.items() if name.endswith('_dem')]
    print('raster coarse land/lakes',flush=True)
    land=rasterize(polygons(source['land']),gw,gh);lake=rasterize(polygons(source['lakes']),gw,gh)
    kind=classify_mask(land,lake)
    river=raster_lines(shapes(source['rivers'],3),gw,gh) if 'rivers' in source else np.zeros((gh,gw),dtype=bool)
    elevation=dem_grid(dems,gw,gh);dem_extent=dems[0][1] if dems else None
    print('partition coarse',flush=True);raw=partition(kind,options,river,elevation,definitions.get('partition'))
    reports=[];cities=city_points(source['cities']) if 'cities' in source else np.empty((0,2))
    del land,lake,river,elevation
    if (gw,gh)!=(w,h):
        print('raster fine coast',flush=True)
        fine_kind=classify_mask(rasterize(polygons(source['land']),w,h),rasterize(polygons(source['lakes']),w,h))
        raw=refine_coast(raw,kind,fine_kind);kind=fine_kind
        river=raster_lines(shapes(source['rivers'],3),w,h)
        raw,reports=regional_refine(raw,kind,river,cities,dems,definitions,options['seed']);del river
    index=connected_ids(raw);del raw
    count=int(index.max())+1;rows=[];names=['sea','land','lake'];colors=definitions['colors']
    for dense,box in enumerate(ndimage.find_objects(index.astype(np.int32)+1)):
        cells=index[box]==dense;y,x=np.nonzero(cells);y+=box[0].start;x+=box[1].start
        # Actual representative cell, rather than an offshore polygon centroid.
        mid=len(x)//2;px=int(x[mid]);py=int(y[mid]);name=names[int(kind[py,px])]
        base=colors[name];variation=((dense*73)% (colors['province_variation']*2+1))-colors['province_variation']
        height_value=elevation_at(dems,(px+.5)*360/w-180,90-(py+.5)*180/h)
        terrain_color=base
        if height_value is not None and name=='land':
            for band in definitions['terrain']['bands']:
                if height_value>=band['minimum_m']:terrain_color=band['color']
        rows.append(dict(id=dense+1,kind=name,pixels=len(x),bounds=[int(x.min()),int(y.min()),int(x.max())+1,int(y.max())+1],longitude=round((px+.5)*360/w-180,6),latitude=round(90-(py+.5)*180/h,6),geography_color=base,province_color=[max(0,min(255,c+variation)) for c in base],terrain_color=terrain_color,elevation_m=height_value))
    out.mkdir(parents=True,exist_ok=True);raw=index_bytes(index);(out/'index.bin').write_bytes(raw)
    hashes={'index.bin':sha(raw),'provinces.json':write_json(out/'provinces.json',rows),'adjacency.json':write_json(out/'adjacency.json',adjacency(index))}
    limitations=['no-global-elevation','sparse-modern-city-samples','generalized-coast-river','subpixel-islands-lakes','no-game-state','no-ridge-pass-accuracy-guarantee'] if reports else ['no-global-elevation','no-city-density','generalized-coast-river','subpixel-islands-lakes','no-game-state','no-ridge-pass-accuracy-guarantee']
    metadata=dict(schema='world-preview-v1',map_id='world_preview',width=w,height=h,projection='EPSG:4326 Plate Carree',extent=[-180,-90,180,90],axis='x east / y south',cell_degrees=360/w,byte_order='u16-le',province_ids=list(range(1,count+1)),counts={name:sum(p['kind']==name for p in rows) for name in names},style=definitions['style'],regions=definitions['regions'],options=options,partition=definitions.get('partition'),tools={'python':platform.python_version(),'numpy':np.__version__,'scipy':scipy.__version__},sources=sources['sources'],hashes=hashes,definitions_sha256=sha((root/'defines.toml').read_bytes()),limitations=limitations,elevation=bool(dem_extent),elevation_extent=dem_extent,elevation_extents=[d[1] for d in dems],river_input='rivers' in source,quality=quality,density_regions=reports,city_input=bool(reports),coast_antialias=bool(quality and quality.get('coast_antialias')))
    write_json(out/'metadata.json',metadata)
    print(json.dumps(dict(count=count,counts=metadata['counts'],hashes=hashes)),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--source',type=Path,default=Path('data/preview/world'));parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();build(args.source,args.out)
