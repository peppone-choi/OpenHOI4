"""One owned bake process with explicit time/commit/working-set budgets."""
import argparse
import ctypes
import json
from pathlib import Path
import subprocess
import sys
import time
import tomllib

class Counters(ctypes.Structure):
    _fields_=[('cb',ctypes.c_uint32),('PageFaultCount',ctypes.c_uint32)]+[(n,ctypes.c_size_t) for n in ['PeakWorkingSetSize','WorkingSetSize','QuotaPeakPagedPoolUsage','QuotaPagedPoolUsage','QuotaPeakNonPagedPoolUsage','QuotaNonPagedPoolUsage','PagefileUsage','PeakPagefileUsage','PrivateUsage']]

def budget_reason(seconds,working_set,commit,limits):
    if max(working_set,commit)>limits['bake_memory_bytes']:return 'memory budget exceeded'
    if seconds>limits['bake_seconds']:return 'time budget exceeded'
    return None

def run(source,out,receipt):
    root=Path(__file__).resolve().parents[2];limits=tomllib.loads((source/'defines.toml').read_text())['quality']
    if out.exists():raise ValueError('new output directory required; preserve previous evidence')
    kernel=ctypes.WinDLL('kernel32');kernel.OpenProcess.argtypes=[ctypes.c_uint32,ctypes.c_bool,ctypes.c_uint32];kernel.OpenProcess.restype=ctypes.c_void_p;kernel.CloseHandle.argtypes=[ctypes.c_void_p]
    psapi=ctypes.WinDLL('psapi');psapi.GetProcessMemoryInfo.argtypes=[ctypes.c_void_p,ctypes.POINTER(Counters),ctypes.c_uint32]
    argv=[sys.executable,str(root/'tools/maps/world_preview.py'),'--source',str(source.resolve()),'--out',str(out.resolve())]
    receipt.parent.mkdir(parents=True,exist_ok=True);started=time.monotonic();samples=[];reason=None
    with receipt.with_suffix('.stdout.log').open('wb') as stdout,receipt.with_suffix('.stderr.log').open('wb') as stderr:
        child=subprocess.Popen(argv,cwd=root,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
        handle=kernel.OpenProcess(0x410,False,child.pid)
        if not handle:child.terminate();child.wait();raise OSError('owned process memory handle unavailable')
        try:
            while child.poll() is None:
                c=Counters();c.cb=ctypes.sizeof(c)
                if not psapi.GetProcessMemoryInfo(handle,ctypes.byref(c),c.cb):raise OSError('owned process memory query failed')
                elapsed=time.monotonic()-started;samples.append({'seconds':round(elapsed,3),'peak_working_set':c.PeakWorkingSetSize,'working_set':c.WorkingSetSize,'commit':c.PagefileUsage,'peak_commit':c.PeakPagefileUsage,'private':c.PrivateUsage})
                reason=budget_reason(elapsed,c.PeakWorkingSetSize,c.PeakPagefileUsage,limits)
                if reason:child.terminate();break
                time.sleep(.2)
            code=child.wait()
        except OSError as error:
            reason=str(error)
        finally:
            if child.poll() is None:child.terminate()
            code=child.wait();kernel.CloseHandle(handle)
    if code==0 and (out/'index.bin').stat().st_size>limits['max_index_bytes']:reason='index byte budget exceeded'
    result={'pid':child.pid,'argv':argv,'cwd':str(root),'native_exit':code,'reason':reason,'seconds':time.monotonic()-started,'limits':limits,'peak_working_set':max((s['peak_working_set'] for s in samples),default=0),'peak_commit':max((s['peak_commit'] for s in samples),default=0),'samples':samples}
    receipt.write_bytes((json.dumps(result,indent=2)+'\n').encode());print(json.dumps({k:v for k,v in result.items() if k!='samples'}));return int(code!=0 or reason is not None)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source',type=Path,default=Path('data/preview/world'));p.add_argument('--out',type=Path,required=True);p.add_argument('--receipt',type=Path,required=True);a=p.parse_args();sys.exit(run(a.source,a.out,a.receipt))
