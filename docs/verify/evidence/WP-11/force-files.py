from run import *
from reference import header,u,fnv,vec,s,i
import shutil,struct,ctypes
folder=OUT/'force-files';folder.mkdir(exist_ok=True);pack=OUT/'own/pack';original=(OUT/'own/saved.ohsave').read_bytes();h=json.load(open(OUT/'own/capture.json'))['header'];cli=ROOT/'target/debug/oh_cli.exe';records=[]
old=folder/'old.ohsave';old.write_bytes(original);oldsha=hashlib.sha256(original).hexdigest()
def invoke(name,load,packroot,force,expected,save=True):
 args=[str(cli),'resume','--load',str(load),'--pack',str(packroot),'--ticks','0','--hash-out']
 if force:args+=['--force']
 if save:args+=['--save-out',str(old)]
 before=hashlib.sha256(old.read_bytes()).hexdigest();r=subprocess.run(args,cwd=ROOT,env=os.environ,capture_output=True)
 (folder/(name+'.stdout')).write_bytes(r.stdout);(folder/(name+'.stderr')).write_bytes(r.stderr)
 assert r.returncode==expected,(name,r.returncode,r.stderr)
 after=hashlib.sha256(old.read_bytes()).hexdigest()
 if expected:assert old.read_bytes()==original and before==oldsha==after
 records.append(dict(name=name,command=args,exit=r.returncode,before_sha=before,after_sha=after,stdout=r.stdout.decode().strip(),stderr=r.stderr.decode(errors='replace')))
changed=folder/'content-pack';shutil.copytree(pack,changed);(changed/'SOURCES.md').write_text('independent source annotation\n')
invoke('content-normal',OUT/'own/saved.ohsave',changed,False,1)
invoke('content-force',OUT/'own/saved.ohsave',changed,True,0,False)
assert records[-1]['stdout']=='dc381123cd314328' and 'PackMismatch' in records[-1]['stderr']
defines=folder/'defines-pack';shutil.copytree(pack,defines);f=defines/'scenarios/m1/defines.toml';f.write_text(f.read_text().replace('500, 200','501, 200'))
invoke('defines-force',OUT/'own/saved.ohsave',defines,True,1)
definitions=folder/'definitions-pack';shutil.copytree(pack,definitions);f=definitions/'scenarios/m1/nations/NTH.toml';f.write_text(f.read_text().replace('40, 100, 180','41, 100, 180'))
invoke('definitions-force',OUT/'own/saved.ohsave',definitions,True,1)
for name,fn in [('badmagic',lambda b:b.__setitem__(0,0)),('format0',lambda b:b.__setitem__(slice(4,6),b'\0\0')),('formatfuture',lambda b:b.__setitem__(slice(4,6),b'\2\0')),('length',lambda b:b.__setitem__(slice(6,10),b'\xff'*4)),('checksum',lambda b:b.__setitem__(-1,b[-1]^1))]:
 b=bytearray(original);fn(b);p=folder/(name+'.ohsave');p.write_bytes(b);invoke(name,p,pack,True,1)
end=10+struct.unpack('<I',original[6:10])[0]
for name,field,value in [('engine','engine_version','0.2.0'),('statehash','state_hash',1),('playerref','player_nations',[19])]:
 hh={**h,field:value};hb=header(hh);p=folder/(name+'.ohsave');p.write_bytes(original[:6]+struct.pack('<I',len(hb))+hb+original[end:]);invoke(name,p,pack,True,1)
invoke('paused-advance',OUT/'own/paused.ohsave',pack,False,0,False)
# Host resume rejects positive advancement from a paused file without a current-tick resume.
args=[str(cli),'resume','--load',str(OUT/'own/paused.ohsave'),'--pack',str(pack),'--ticks','1','--save-out',str(old),'--hash-out'];r=subprocess.run(args,cwd=ROOT,env=os.environ,capture_output=True);assert r.returncode==1 and b'PausedCannotAdvance' in r.stderr and old.read_bytes()==original;records.append(dict(name='paused-positive',command=args,exit=r.returncode,old_sha=oldsha,stderr=r.stderr.decode()))
# A real Windows share-denied replacement failure, using the public production CLI.
kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.CreateFileW.argtypes=[ctypes.c_wchar_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p,ctypes.c_uint32,ctypes.c_uint32,ctypes.c_void_p];kernel.CreateFileW.restype=ctypes.c_void_p;kernel.CloseHandle.argtypes=[ctypes.c_void_p]
handle=kernel.CreateFileW(str(old),0x80000000,0,None,3,0,None);assert handle not in (None,ctypes.c_void_p(-1).value)
try:r=subprocess.run([str(cli),'resume','--load',str(OUT/'own/saved.ohsave'),'--pack',str(pack),'--ticks','0','--save-out',str(old),'--hash-out'],cwd=ROOT,env=os.environ,capture_output=True);assert r.returncode==1 and b'SaveIO: commit' in r.stderr
finally:kernel.CloseHandle(handle)
assert old.read_bytes()==original;records.append(dict(name='actual-windows-commit-denied',exit=r.returncode,old_sha=hashlib.sha256(old.read_bytes()).hexdigest(),stderr=r.stderr.decode()))
linkpack=folder/'junction-pack';shutil.copytree(pack,linkpack);link=linkpack/'junction'
r=subprocess.run(['powershell','-NoProfile','-Command','New-Item -ItemType Junction -Path "'+str(link)+'" -Target "'+str(pack/'maps')+'" | Out-Null'],cwd=ROOT,env=os.environ,capture_output=True);assert r.returncode==0
invoke('junction-pack',OUT/'own/saved.ohsave',linkpack,True,1);assert 'symlink/reparse unsupported' in records[-1]['stderr']
oldbytes=(OUT/'transactions/old.bin').read_bytes();newbytes=(OUT/'transactions/new.bin').read_bytes();transactions=[]
for stage in range(6):
 for exists in [False,True]:
  p=OUT/'transactions'/f'stage-{stage}-existing-{str(exists).lower()}'/'save.ohsave'
  if stage<4:assert p.read_bytes()==oldbytes if exists else not p.exists()
  else:assert p.read_bytes()==newbytes
  transactions.append(dict(stage=stage,existing=exists,expected='old unchanged' if stage<4 and exists else 'absent' if stage<4 else 'new committed',actual_sha=hashlib.sha256(p.read_bytes()).hexdigest() if p.exists() else None))
(OUT/'force-file-results.json').write_text(json.dumps(dict(records=records,transactions=transactions,old_transaction_sha=hashlib.sha256(oldbytes).hexdigest(),new_transaction_sha=hashlib.sha256(newbytes).hexdigest()),indent=2));print(json.dumps(dict(native_cases=len(records),transaction_cases=len(transactions),old_sha=oldsha)))
