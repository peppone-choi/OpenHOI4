"""Read-only geographic component/urban/sea quality audit, not a game model."""
import argparse
import json
from pathlib import Path
import tomllib
import numpy as np
from scipy import ndimage
import world_preview as wp

SAMPLES={
 'ireland':(-8,53),'great_britain':(-2,54),'honshu':(138,36),'hokkaido':(143,43),'kyushu':(131,33),'shikoku':(133.5,33.75),
 'eurasia':(90,45),'africa':(20,0),'north_america':(-100,40),'south_america':(-60,-15),'australia':(133,-25),'antarctica':(0,-80),'greenland':(-42,73),'madagascar':(47,-19),
}
CITY_SAMPLES={'dublin':(-6.26,53.35),'london':(-.12,51.5),'seoul':(127,37.566),'tokyo':(139.75,35.68)}
WINDOWS={'europe':[-12,35,45,72],'africa':[-20,-36,52,36],'east_asia':[100,5,150,55],'north_america':[-170,5,-50,75],'south_america':[-82,-57,-34,13],'australia':[112,-45,154,-10]}
SEA_SAMPLES={'irish_sea':[-7,51,-2,55.5],'english_channel':[-5.5,48,2,51.5],'japan_nearshore':[128,30,145,46],'atlantic':[-50,10,-20,40]}

def area_rows(width,height,radius):
    latitude=np.radians(90-np.arange(height+1)*180/height)
    return radius**2*np.radians(360/width)*(np.sin(latitude[:-1])-np.sin(latitude[1:]))

def area_totals(index,count,areas):
    totals=np.zeros(count)
    for y in range(0,len(index),64):
        patch=index[y:y+64];totals+=np.bincount(patch.ravel(),weights=np.repeat(areas[y:y+len(patch)],patch.shape[1]),minlength=count)
    return totals

def distribution(values):
    return {str(q):float(np.percentile(values,q)) for q in [0,10,50,90,100]} if len(values) else None

def audit(root,out,report,urban_source=None):
    definitions=tomllib.loads((root/'defines.toml').read_text());radius=definitions['quality']['earth_radius_km']
    meta=json.loads((out/'metadata.json').read_text());rows=json.loads((out/'provinces.json').read_text());h,w=meta['height'],meta['width']
    index=np.frombuffer((out/'index.bin').read_bytes(),dtype='<u2').reshape(h,w);kind_table=np.array([['sea','land','lake'].index(p['kind']) for p in rows],dtype=np.uint8);kind=kind_table[index]
    areas=area_rows(w,h,radius);province_areas=area_totals(index,len(rows),areas);pixels=np.array([p['pixels'] for p in rows])
    cities=wp.city_points(root/'sources/ne_10m_populated_places_simple.zip')
    result={'candidate':str(out),'index_sha256':wp.sha((out/'index.bin').read_bytes()),'resolution':[w,h],'area_method':'Mean-radius sphere R^2*dLongitude*(sin north - sin south), raster approximation, not ellipsoidal survey/game values','radius_km':radius,'counts':meta['counts'],'province_area_km2_percentiles':{},'one_pixel':{},'source_kind_component_one_pixel':{},'components':{},'windows':{},'sea_windows':{},'cities':{}}
    labels,num=ndimage.label(kind==1);component_areas=area_totals(labels,num+1,areas);boxes=ndimage.find_objects(labels)
    city_x=np.clip(((cities[:,0]+180)*w/360).astype(int),0,w-1);city_y=np.clip(((90-cities[:,1])*h/180).astype(int),0,h-1);city_labels=labels[city_y,city_x]
    city_counts=np.bincount(city_labels,minlength=num+1)
    result['uncovered_large_land_components']=[{'component':i,'area_km2':float(component_areas[i]),'source_city_points':int(city_counts[i])} for i in range(1,num+1) if component_areas[i]>=10000 and city_counts[i]==0]
    for name,(lon,lat) in SAMPLES.items():
        x=int((lon+180)*w/360);y=int((90-lat)*h/180);component=int(labels[y,x])
        if component==0:result['components'][name]={'error':'sample source cell is not land'};continue
        box=boxes[component-1];mask=labels[box]==component;ids,counts=np.unique(index[box][mask],return_counts=True)
        local_areas=np.broadcast_to(areas[box[0]][:,None],mask.shape)[mask];area_by_id=np.bincount(index[box][mask],weights=local_areas,minlength=len(rows));used=area_by_id[ids]
        result['components'][name]={'component':component,'anchor':[lon,lat],'bbox_pixels':[box[1].start,box[0].start,box[1].stop,box[0].stop],'pixels':int(mask.sum()),'area_km2':float(component_areas[component]),'province_ids':(ids+1).tolist(),'land_id_count':len(ids),'largest_pixel_share':float(counts.max()/counts.sum()),'largest_area_share':float(used.max()/used.sum()),'intersection_area_km2_percentiles':distribution(used),'province_pixel_percentiles':distribution(counts),'source_city_points':int(city_counts[component]),'bbox_degrees':[(box[1].stop-box[1].start)*360/w,(box[0].stop-box[0].start)*180/h]}
    del labels,boxes
    for k,name in enumerate(['sea','land','lake']):
        used=kind_table==k;result['one_pixel'][name]=int(np.sum(used&(pixels==1)));result['province_area_km2_percentiles'][name]=distribution(province_areas[used])
        source_labels,n=ndimage.label(kind==k);sizes=np.bincount(source_labels.ravel(),minlength=n+1);result['source_kind_component_one_pixel'][name]=int(np.sum(sizes[1:]==1));del source_labels
    def window(extent,k):
        west,south,east,north=extent;box=(slice(int((90-north)*h/180),int((90-south)*h/180)),slice(int((west+180)*w/360),int((east+180)*w/360)));mask=kind[box]==k
        ids,counts=np.unique(index[box][mask],return_counts=True);local_areas=np.broadcast_to(areas[box[0]][:,None],mask.shape)[mask];used=np.bincount(index[box][mask],weights=local_areas,minlength=len(rows))[ids]
        return {'extent':extent,'scope':'Geographic crop intersection, not territory or complete province area','ids':len(ids),'one_pixel_intersections':int(np.sum(counts==1)),'area_km2':float(used.sum()),'intersection_area_km2_percentiles':distribution(used),'whole_province_area_km2_percentiles':distribution(province_areas[ids])}
    result['windows']={name:window(extent,1) for name,extent in WINDOWS.items()};result['sea_windows']={name:window(extent,0) for name,extent in SEA_SAMPLES.items()}
    urban=None
    if urban_source:
        urban=wp.rasterize(wp.polygons(urban_source),w,h)&(kind==1);urban_labels,n=ndimage.label(urban);urban_areas=area_totals(urban_labels,n+1,areas);result['urban_source_mask']={'components':n,'pixels':int(urban.sum()),'area_km2':float(urban_areas[1:].sum()),'one_pixel_components':int(np.sum(np.bincount(urban_labels.ravel(),minlength=n+1)[1:]==1)),'input_sha256':wp.sha(urban_source.read_bytes())}
    for name,(lon,lat) in CITY_SAMPLES.items():
        point=cities[np.argmin(np.linalg.norm(wp.lonlat_sphere(cities[:,0],cities[:,1])-wp.lonlat_sphere(np.array([lon]),np.array([lat])),axis=1))];x=int((point[0]+180)*w/360);y=int((90-point[1])*h/180);dense=int(index[y,x]);p=rows[dense]
        city={'reference_coordinate':[lon,lat],'nearest_actual_source_point':point.tolist(),'province_id':dense+1,'kind':p['kind'],'province_area_km2':float(province_areas[dense]),'province_pixels':p['pixels']}
        if urban is not None:
            u=int(urban_labels[y,x]);city.update({'source_urban_at_point':bool(urban[y,x]),'urban_component':u,'urban_component_area_km2':float(urban_areas[u]) if u else None})
            box=(slice(p['bounds'][1],p['bounds'][3]),slice(p['bounds'][0],p['bounds'][2]));cells=index[box]==dense;city['urban_fraction_in_current_province']=float(np.sum(urban[box]&cells)/np.sum(cells))
        result['cities'][name]=city
    report.write_bytes((json.dumps(result,indent=2)+'\n').encode());print(json.dumps({'report':str(report),'components':{name:{k:p.get(k) for k in ['land_id_count','area_km2','largest_pixel_share','source_city_points']} for name,p in result['components'].items()},'one_pixel':result['one_pixel'],'true_source_kind_1pixel':result['source_kind_component_one_pixel'],'urban_source_mask':result.get('urban_source_mask'),'cities':result['cities']}));return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',type=Path,default=Path('data/preview/world'));p.add_argument('--out',type=Path,required=True);p.add_argument('--report',type=Path,required=True);p.add_argument('--urban-source',type=Path);a=p.parse_args();audit(a.source,a.out,a.report,a.urban_source)
