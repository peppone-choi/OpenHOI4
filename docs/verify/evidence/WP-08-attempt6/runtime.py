import pathlib,subprocess,socket,urllib.request,time,json,re,hashlib,datetime
from process_cwd import process_cwd
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6'
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
class Runtime:
 def __init__(self,root,port,label):self.root=root;self.port=port;self.label=label;self.p=None
 def __enter__(self):
  with socket.socket() as s:s.bind(('127.0.0.1',self.port))
  self.exe=self.root/'target/debug/oh_server.exe';self.args=[str(self.exe),'--port',str(self.port),'--pack-root',str(self.root/'data/packs')];self.start=now()
  self.file=(OUT/(self.label+'-server.log')).open('wb');self.p=subprocess.Popen(self.args,cwd=self.root,stdout=self.file,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
  self.data={'start':self.start,'pid':self.p.pid,'root':str(self.root),'exe':str(self.exe),'exeSHA256':hashlib.sha256(self.exe.read_bytes()).hexdigest(),'args':self.args,'port':self.port,'source': 'exact9fbc' if self.root==ROOT else self.root.name}
  try:
   deadline=time.monotonic()+15
   while True:
    assert self.p.poll() is None,'OWN SERVER EXITED BEFORE BIND: HTTP/browser prohibited'
    lines=[l for l in subprocess.check_output(['netstat','-ano','-p','tcp'],text=True).splitlines() if f'127.0.0.1:{self.port} ' in l and 'LISTENING' in l]
    if lines:
     assert len(lines)==1 and int(lines[0].split()[-1])==self.p.pid,lines
     break
    assert time.monotonic()<deadline,'bind deadline';time.sleep(.05)
   self.data['netstat_before_HTTP']=lines;self.data['actual_cwd']=process_cwd(self.p.pid);assert pathlib.Path(self.data['actual_cwd']).resolve()==self.root.resolve()
   with urllib.request.urlopen(f'http://127.0.0.1:{self.port}/') as r:html=r.read();assert r.status==200
   assets=[]
   for asset in re.findall(r'(?:src|href)="(/assets/[^"]+)"',html.decode()):
    with urllib.request.urlopen(f'http://127.0.0.1:{self.port}'+asset) as r:b=r.read();assert r.status==200
    assert b==(self.root/'client/dist'/asset.lstrip('/')).read_bytes();assets.append({'path':asset,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'own_dist_equal':True})
   self.data.update(http200=True,assets=assets,metadata=json.load(urllib.request.urlopen(f'http://127.0.0.1:{self.port}/maps/testland/metadata')));self.save();return self
  except: self.__exit__(None,None,None);raise
 def save(self):(OUT/(self.label+'-runtime.json')).write_text(json.dumps(self.data,indent=2),encoding='utf8')
 def __exit__(self,*exc):
  if self.p and self.p.poll() is None:self.p.terminate();self.p.wait(timeout=10)
  if self.p:self.data.update(end=now(),exit=self.p.returncode);self.save()
  if hasattr(self,'file'):self.file.close()
  with socket.socket() as s:s.bind(('127.0.0.1',self.port))
  if self.p:self.data['portReleased']=True;self.save()
