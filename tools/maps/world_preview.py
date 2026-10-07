"""Offline Natural Earth preview bake. Never downloads missing input."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import struct
import tomllib
import zipfile
import numpy as np
import scipy
from scipy import ndimage
from scipy.spatial import cKDTree


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


def polygons(path):
    """Read only Polygon SHP records; no DBF/third-party shapefile package."""
    with zipfile.ZipFile(path) as archive:
        names=[n for n in archive.namelist() if n.endswith('.shp')]
        if len(names)!=1:
            raise ValueError('one polygon SHP required')
        data=archive.read(names[0])
    if struct.unpack_from('>i',data)[0]!=9994 or struct.unpack_from('<i',data,32)[0]!=5:
        raise ValueError('invalid Polygon SHP')
    offset=100
    while offset<len(data):
        size=struct.unpack_from('>i',data,offset+4)[0]*2
        record=data[offset+8:offset+8+size];offset+=8+size
        if struct.unpack_from('<i',record)[0]==0:
            continue
        if struct.unpack_from('<i',record)[0]!=5:
            raise ValueError('unexpected shape type')
        parts,count=struct.unpack_from('<ii',record,36)
        starts=list(struct.unpack_from('<'+'i'*parts,record,44))+[count]
        points=np.frombuffer(record,dtype='<f8',count=count*2,offset=44+4*parts).reshape(-1,2)
        yield [points[starts[i]:starts[i+1]] for i in range(parts)]


def rasterize(items,width,height):
    """Even-odd fill at cell centers; rings retain polygon holes."""
    mask=np.zeros((height,width),dtype=bool)
    for rings in items:
        all_points=np.concatenate(rings)
        px=(all_points[:,0]+180)*width/360
        py=(90-all_points[:,1])*height/180
        x0=max(0,int(np.floor(px.min())));x1=min(width,int(np.ceil(px.max()))+1)
        y0=max(0,int(np.floor(py.min())));y1=min(height,int(np.ceil(py.max()))+1)
        patch=np.zeros((y1-y0,x1-x0),dtype=bool)
        for ring in rings:
            p=np.array(ring,dtype=float);x=(p[:,0]+180)*width/360;y=(90-p[:,1])*height/180
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


def partition(kind,options):
    height,width=kind.shape;rng=np.random.Generator(np.random.PCG64(options['seed']))
    raw=np.zeros(kind.shape,dtype=np.int32);offset=0
    for k,name in enumerate(['sea','land','lake']):
        ys,xs=np.nonzero(kind==k)
        count=min(options[name+'_seeds'],len(ys))
        if not count:
            continue
        weights=np.cos(np.radians(90-(ys+.5)*180/height));weights/=weights.sum()
        chosen=rng.choice(len(ys),size=count,replace=False,p=weights)
        tree=cKDTree(sphere(xs[chosen],ys[chosen],width,height))
        for row in range(0,height,options['query_rows']):
            y,x=np.nonzero(kind[row:row+options['query_rows']]==k);y+=row
            if len(y):
                _,nearest=tree.query(sphere(x,y,width,height),workers=1)
                raw[y,x]=nearest+offset
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
            if next_id>=65536:
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


def write_json(path,value):
    data=(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n').encode()
    path.write_bytes(data)
    return sha(data)


def build(root,out):
    sources=verify_sources(root);definitions=tomllib.loads((root/'defines.toml').read_text())
    options=definitions['generation'];w=options['width'];h=options['height']
    source={s['id']:root/s['file'] for s in sources['sources']}
    print('raster land/lakes',flush=True)
    land=rasterize(polygons(source['land']),w,h);lake=rasterize(polygons(source['lakes']),w,h)
    kind=np.zeros((h,w),dtype=np.uint8);kind[land]=1;kind[lake&land]=2
    print('partition',flush=True);index=connected_ids(partition(kind,options))
    count=int(index.max())+1;rows=[];names=['sea','land','lake'];colors=definitions['colors']
    for dense,box in enumerate(ndimage.find_objects(index.astype(np.int32)+1)):
        cells=index[box]==dense;y,x=np.nonzero(cells);y+=box[0].start;x+=box[1].start
        # Actual representative cell, rather than an offshore polygon centroid.
        mid=len(x)//2;px=int(x[mid]);py=int(y[mid]);name=names[int(kind[py,px])]
        base=colors[name];variation=((dense*73)% (colors['province_variation']*2+1))-colors['province_variation']
        rows.append(dict(id=dense+1,kind=name,pixels=len(x),bounds=[int(x.min()),int(y.min()),int(x.max())+1,int(y.max())+1],longitude=round((px+.5)*360/w-180,6),latitude=round(90-(py+.5)*180/h,6),geography_color=base,province_color=[max(0,min(255,c+variation)) for c in base],elevation_m=None))
    out.mkdir(parents=True,exist_ok=True);raw=index_bytes(index);(out/'index.bin').write_bytes(raw)
    hashes={'index.bin':sha(raw),'provinces.json':write_json(out/'provinces.json',rows),'adjacency.json':write_json(out/'adjacency.json',adjacency(index))}
    metadata=dict(schema='world-preview-v1',map_id='world_preview',width=w,height=h,projection='EPSG:4326 Plate Carree',extent=[-180,-90,180,90],axis='x east / y south',cell_degrees=360/w,byte_order='u16-le',province_ids=list(range(1,count+1)),counts={name:sum(p['kind']==name for p in rows) for name in names},style=definitions['style'],regions=definitions['regions'],options=options,tools={'python':'3.11.0','numpy':np.__version__,'scipy':scipy.__version__},sources=sources['sources'],hashes=hashes,definitions_sha256=sha((root/'defines.toml').read_bytes()),limitations=['no-elevation','no-river-input','no-city-density','generalized-coast','subpixel-islands-lakes','no-game-state'],elevation=False)
    write_json(out/'metadata.json',metadata)
    print(json.dumps(dict(count=count,counts=metadata['counts'],hashes=hashes)),flush=True)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--source',type=Path,default=Path('data/preview/world'));parser.add_argument('--out',type=Path,required=True)
    args=parser.parse_args();build(args.source,args.out)
