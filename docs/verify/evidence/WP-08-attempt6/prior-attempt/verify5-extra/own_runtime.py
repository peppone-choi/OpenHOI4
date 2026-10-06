import run_checks as m,subprocess,socket,urllib.request,time,json,re,hashlib
class Runtime:
 def __init__(self,root,port,label):self.root=root;self.port=port;self.label=label;self.p=None
 def __enter__(self):
  with socket.socket() as s:s.bind(('127.0.0.1',self.port))
  exe=self.root/'target/debug/oh_server.exe';args=[str(exe),'--port',str(self.port),'--pack-root',str(self.root/'data/packs')];self.started=m.now()
  self.f=(m.OUT/(self.label+'-server.log')).open('wb');self.p=subprocess.Popen(args,cwd=self.root,stdout=self.f,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
  self.data={'start':self.started,'pid':self.p.pid,'root':str(self.root),'exe':str(exe),'exeSHA':hashlib.sha256(exe.read_bytes()).hexdigest(),'args':args,'cwd':str(self.root),'port':self.port}
  try:
   deadline=time.monotonic()+15
   while True:
    assert self.p.poll() is None,'own server died before HTTP ready'
    try:
     with urllib.request.urlopen(f'http://127.0.0.1:{self.port}/',timeout=1) as r:html=r.read();assert r.status==200
     break
    except OSError:
     if time.monotonic()>deadline:raise
     time.sleep(.05)
   tcp=subprocess.check_output(['netstat','-ano','-p','tcp']).decode();lines=[l for l in tcp.splitlines() if f'127.0.0.1:{self.port} ' in l and 'LISTENING' in l];assert len(lines)==1 and int(lines[0].split()[-1])==self.p.pid
   assets=[]
   for path in re.findall(r'(?:src|href)="(/assets/[^"]+)"',html.decode()):
    with urllib.request.urlopen(f'http://127.0.0.1:{self.port}'+path) as r:b=r.read();assert r.status==200
    assert b==(self.root/'client/dist'/path.lstrip('/')).read_bytes();assets.append({'path':path,'length':len(b),'sha':hashlib.sha256(b).hexdigest()})
   self.data.update(http200=True,assets=assets,netstat=lines);self.save();return self
  except: self.__exit__(None,None,None);raise
 def save(self):(m.OUT/(self.label+'-runtime.json')).write_text(json.dumps(self.data,indent=2))
 def __exit__(self,*exc):
  if self.p and self.p.poll() is None:self.p.terminate();self.p.wait(timeout=10)
  if self.p:self.data.update(end=m.now(),exit=self.p.returncode);self.save()
  if hasattr(self,'f'):self.f.close()
  with socket.socket() as s:s.bind(('127.0.0.1',self.port))
  if self.p:self.data['portReleased']=True;self.save()
