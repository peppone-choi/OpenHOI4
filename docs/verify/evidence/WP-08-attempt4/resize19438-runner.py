from pathlib import Path
import subprocess,time,datetime,hashlib,json,re,urllib.request
r=Path.cwd();o=r/'target/wp08-verify4';src=(o/'resize.cjs').read_text(encoding='utf-8-sig').replace('19437','19438').replace('resize-${label}','resize19438-${label}').replace('resize-results.json','resize19438-results.json');(o/'resize19438.cjs').write_text(src,encoding='utf-8');exe=r/'target/debug/oh_server.exe';cmd=[str(exe),'--port','19438','--pack-root',str(r/'data/packs')];log=(o/'server19438.log').open('wb');start=datetime.datetime.now().astimezone().isoformat();proc=subprocess.Popen(cmd,cwd=r,stdout=log,stderr=subprocess.STDOUT);identity={'pid':proc.pid,'command':cmd,'started':start,'exeSHA256':hashlib.sha256(exe.read_bytes()).hexdigest()}
try:
 for i in range(80):
  if proc.poll()is not None:raise RuntimeError('own server failed to bind: '+str(proc.returncode))
  try:
   html=urllib.request.urlopen('http://127.0.0.1:19438/',timeout=1).read().decode();break
  except OSError:time.sleep(.1)
 else:raise RuntimeError('server not ready')
 asset=re.search(r'src="([^"]+\.js)"',html).group(1);served=urllib.request.urlopen('http://127.0.0.1:19438'+asset).read();local=(r/'client/dist'/asset.lstrip('/')).read_bytes();assert served==local;meta=json.load(urllib.request.urlopen('http://127.0.0.1:19438/maps/testland/metadata'));identity.update(servedJS=asset,servedJSSHA256=hashlib.sha256(served).hexdigest(),packHash=meta['pack_hash'],probeStarted=datetime.datetime.now().astimezone().isoformat())
 with(o/'resize19438.log').open('wb')as f:q=subprocess.run(['node',str(o/'resize19438.cjs')],cwd=r,stdout=f,stderr=subprocess.STDOUT)
 identity.update(probeExit=q.returncode,probeFinished=datetime.datetime.now().astimezone().isoformat(),serverAlive=proc.poll()is None);assert q.returncode==0
finally:
 if proc.poll()is None:proc.terminate()
 identity.update(serverExit=proc.wait(),serverEnded=datetime.datetime.now().astimezone().isoformat());log.close();(o/'server19438-identity.json').write_text(json.dumps(identity,indent=2),encoding='utf-8')
print(json.dumps(identity,indent=2))
