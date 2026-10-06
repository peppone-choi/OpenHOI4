from run import *
import socket,shutil,urllib.request,re
sys.path.insert(0,str(ROOT/'crates/oh_server/tests'))
from lifecycle import ctrl_c
folder=OUT/'ui';folder.mkdir(exist_ok=True)
shutil.copytree(OUT/'own/pack',folder/'packs/testland',dirs_exist_ok=True)
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
exe=ROOT/'target/debug/oh_server.exe';args=[str(exe),'--port',str(port),'--pack-root',str(folder/'packs'),'--load-save',str(OUT/'own/paused.ohsave')]
startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
p=subprocess.Popen(args,cwd=ROOT,env=os.environ,stdout=open(folder/'stdout.log','wb'),stderr=open(folder/'stderr.log','wb'),creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=startup)
url=f'http://127.0.0.1:{port}/'
try:
 deadline=time.monotonic()+15
 while True:
  try:html=urllib.request.urlopen(url,timeout=1).read();break
  except OSError:
   assert p.poll() is None
   if time.monotonic()>deadline:raise
   time.sleep(.05)
 path=re.search(rb'<script[^>]*src="([^"]+)"',html).group(1).decode();js=urllib.request.urlopen(url.rstrip('/')+path).read();assert js==(ROOT/'client/dist'/path.lstrip('/')).read_bytes()
 record=dict(head=BASE['head'],pid=p.pid,exclusive_freeport=port,command=args,exe_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),js_path=path,served_js_sha256=hashlib.sha256(js).hexdigest(),save_sha256=hashlib.sha256((OUT/'own/paused.ohsave').read_bytes()).hexdigest(),url=url)
 (folder/'identity.json').write_text(json.dumps(record,indent=2));print(json.dumps(record),flush=True)
 while not (folder/'stop').exists():
  assert p.poll() is None;time.sleep(.2)
finally:
 if p.poll() is None:ctrl_c(p)
 code=p.wait(timeout=10);(folder/'exit.json').write_text(json.dumps(dict(own_pid=p.pid,exit=code)));print('own server exit='+str(code),flush=True)
