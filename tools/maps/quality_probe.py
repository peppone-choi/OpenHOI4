"""Bounded regional measurement only; never writes shipped map outputs."""
import argparse
import ctypes
import json
from pathlib import Path
import sys
import time
import tomllib
import numpy as np
import world_preview as wp


def memory():
    class Counters(ctypes.Structure):
        _fields_=[('cb',ctypes.c_uint32),('PageFaultCount',ctypes.c_uint32)]+[(n,ctypes.c_size_t) for n in ['PeakWorkingSetSize','WorkingSetSize','QuotaPeakPagedPoolUsage','QuotaPagedPoolUsage','QuotaPeakNonPagedPoolUsage','QuotaNonPagedPoolUsage','PagefileUsage','PeakPagefileUsage','PrivateUsage']]
    c=Counters();c.cb=ctypes.sizeof(c)
    kernel=ctypes.WinDLL('kernel32');kernel.GetCurrentProcess.restype=ctypes.c_void_p
    psapi=ctypes.WinDLL('psapi');psapi.GetProcessMemoryInfo.argtypes=[ctypes.c_void_p,ctypes.POINTER(Counters),ctypes.c_uint32]
    if not psapi.GetProcessMemoryInfo(kernel.GetCurrentProcess(),ctypes.byref(c),c.cb):raise OSError('memory query failed')
    return {'peak_working_set':c.PeakWorkingSetSize,'working_set':c.WorkingSetSize,'private_bytes':c.PrivateUsage}


def region_items(items,extent):
    west,south,east,north=extent
    for rings in items:
        points=np.concatenate(rings)
        if points[:,0].max()<west or points[:,0].min()>east or points[:,1].max()<south or points[:,1].min()>north:continue
        transformed=[]
        for p in rings:
            copy=p.copy();copy[:,0]=(p[:,0]-west)/(east-west)*360-180;copy[:,1]=(p[:,1]-south)/(north-south)*180-90;transformed.append(copy)
        yield transformed


def run(size):
    root=Path(__file__).resolve().parents[2];source=root/'data/preview/world';wp.verify_sources(source)
    extent=[122,32,132,43];started=time.monotonic();stages=[]
    land=wp.rasterize(wp.polygons(source/'sources/ne_10m_land.zip'),size,size,extent);stages.append({'phase':'land-mask','seconds':time.monotonic()-started,**memory()})
    lakes=wp.rasterize(wp.polygons(source/'sources/ne_10m_lakes.zip'),size,size,extent)
    kind=wp.classify_mask(land,lakes)
    rivers=wp.raster_lines(region_items(wp.shapes(source/'sources/ne_10m_rivers_lake_centerlines.zip',3),extent),size,size)
    ys,xs=np.nonzero(kind==1);take=np.linspace(0,len(ys)-1,min(160,len(ys)),dtype=int);seeds=np.column_stack((ys[take],xs[take]))
    dem=wp.read_dem((source/'sources/ETOPO2022-korea.tif').read_bytes());elevation=wp.dem_grid([dem],size,size,extent)
    options=tomllib.loads((source/'defines.toml').read_text())['partition']
    labels=wp.geodesic_labels(kind,1,seeds,rivers,elevation,options,extent)
    stages.append({'phase':'region-graph','seconds':time.monotonic()-started,**memory()})
    return {'extent':extent,'width':size,'height':size,'dem_file':'ETOPO2022-korea.tif','valid_dem_pixels':int(np.isfinite(elevation).sum()),'note':'Memory/time allocation probe with evenly sampled seed positions; not a city-density visual proof. Earlier prototype logs used a global-coordinate metric and no DEM.','land_pixels':int(land.sum()),'river_pixels':int(rivers.sum()),'seed_count':len(seeds),'reachable_pixels':int((labels>=0).sum()),'seconds':time.monotonic()-started,'stages':stages,'numpy_buffers':sum(x.nbytes for x in [land,lakes,kind,rivers,labels,elevation])}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--size',type=int,required=True);p.add_argument('--out',type=Path,required=True);args=p.parse_args()
    if args.size not in [512,1024,2048]:raise ValueError('bounded region candidates only')
    result=run(args.size);args.out.write_bytes((json.dumps(result,indent=2)+'\n').encode());print(json.dumps(result))
