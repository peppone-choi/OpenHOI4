"""Read-only exhaustive baked candidate integrity and geographic comparison."""
import argparse
import json
from pathlib import Path
import subprocess
import numpy as np
from scipy import ndimage
import world_preview as wp

def validate(root,out,report):
    meta=json.loads((out/'metadata.json').read_text());rows=json.loads((out/'provinces.json').read_text())
    w,h=meta['width'],meta['height'];index=np.frombuffer((out/'index.bin').read_bytes(),dtype='<u2').reshape(h,w)
    assert meta['province_ids']==list(range(1,len(rows)+1)) and len(rows)<=65535
    assert int(index.max())+1==len(rows) and sum(p['pixels'] for p in rows)==index.size
    for file,hash_value in meta['hashes'].items():assert wp.sha((out/file).read_bytes())==hash_value,file
    sources=wp.verify_sources(root);paths={s['id']:root/s['file'] for s in sources['sources']}
    assert sources['sources']==meta['sources']
    expected_kind=wp.classify_mask(wp.rasterize(wp.polygons(paths['land']),w,h),wp.rasterize(wp.polygons(paths['lakes']),w,h))
    kind_table=np.array([['sea','land','lake'].index(p['kind']) for p in rows],dtype=np.uint8)
    assert np.array_equal(kind_table[index],expected_kind),'source classification mismatch'
    del expected_kind
    measured_elevations=0;dems=[wp.read_dem(path.read_bytes()) for name,path in paths.items() if name.endswith('_dem')]
    for dense,box in enumerate(ndimage.find_objects(index.astype(np.int32)+1)):
        assert box is not None
        cells=index[box]==dense;_,count=ndimage.label(cells);assert count==1,dense+1
        p=rows[dense];assert p['id']==dense+1 and p['pixels']==int(cells.sum())
        yy,xx=np.nonzero(cells);yy+=box[0].start;xx+=box[1].start
        assert p['bounds']==[int(xx.min()),int(yy.min()),int(xx.max())+1,int(yy.max())+1]
        x=int((p['longitude']+180)*w/360);y=int((90-p['latitude'])*h/180);assert int(index[y,x])==dense
        assert p['elevation_m']==wp.elevation_at(dems,(x+.5)*360/w-180,90-(y+.5)*180/h)
        measured_elevations+=p['elevation_m'] is not None
    pairs=wp.adjacency(index);assert json.loads((out/'adjacency.json').read_text())==[list(p) for p in pairs]
    for lon,lat,kind in [(127,37,'land'),(85,29,'land'),(-100,40,'land'),(-140,0,'sea'),(25,-75,'land'),(33,-1,'lake'),(-150,65,'land'),(0,0,'sea')]:
        assert rows[int(index[int((90-lat)*h/180),int((lon+180)*w/360)])]['kind']==kind
    baseline='2d920ea86c6d8045adb7d3115007230e52c3ccf2'
    oldmeta=json.loads(subprocess.check_output(['git','show',baseline+':client/public/preview/world/metadata.json']))
    oldrows=json.loads(subprocess.check_output(['git','show',baseline+':client/public/preview/world/provinces.json']))
    oldindex=np.frombuffer(subprocess.check_output(['git','show',baseline+':client/public/preview/world/index.bin']),dtype='<u2').reshape(oldmeta['height'],oldmeta['width'])
    def region_summary(meta,rows,index,extent):
        west,south,east,north=extent;hh,ww=index.shape
        x0=int((west+180)*ww/360);x1=int((east+180)*ww/360);y0=int((90-north)*hh/180);y1=int((90-south)*hh/180)
        ids,counts=np.unique(index[y0:y1,x0:x1],return_counts=True)
        land=[(int(i)+1,int(c)) for i,c in zip(ids,counts) if rows[int(i)]['kind']=='land']
        return {'land_ids_intersecting_window':len(land),'median_land_intersection_square_degrees':float(np.median([c*(360/ww)*(180/hh) for _,c in land]))}
    regions={r['id']:{'extent':r['extent'],'before':region_summary(oldmeta,oldrows,oldindex,r['extent']),'after':region_summary(meta,rows,index,r['extent'])} for r in meta['density_regions']}
    result={'candidate':str(out),'baseline_inputs_commit':baseline,'status':'checks passed (implementation self-check; no independent gate)','width':w,'height':h,'ids':len(rows),'counts':meta['counts'],'adjacency_pairs':len(pairs),'representative_elevations':measured_elevations,'all_ids_4connected':True,'source_kind_all_pixels_equal':True,'source_sha_verified':len(sources['sources']),'regions':regions,'hashes':meta['hashes']}
    report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));return result

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',type=Path,default=Path('data/preview/world'));p.add_argument('--out',type=Path,required=True);p.add_argument('--report',type=Path,required=True);a=p.parse_args();validate(a.source,a.out,a.report)
