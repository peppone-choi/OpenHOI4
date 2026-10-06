import socket,subprocess,pathlib,json,os,hashlib,re,urllib.request,time,shutil
r=pathlib.Path.cwd();o=r/'target/wp11-verify2/ui';o.mkdir(exist_ok=True);packs=o/'packs';shutil.copytree(r/'target/wp11-verify2/own-pack',packs/'testland',dirs_exist_ok=True)
with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
exe=r/'target/debug/oh_server.exe';save=r/'target/wp11-verify2/own-native/paused.ohsave'
cmd=[str(exe),'--port',str(port),'--pack-root',str(packs),'--load-save',str(save)]
startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
p=subprocess.Popen(cmd,cwd=r,stdout=(o/'stdout').open('wb'),stderr=(o/'stderr').open('wb'),creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=startup)
url=f'http://127.0.0.1:{port}/'
for i in range(100):
 try:html=urllib.request.urlopen(url).read();break
 except OSError:assert p.poll() is None;time.sleep(.1)
script=re.search(rb'<script[^>]*src="([^"]+)"',html).group(1).decode();js=urllib.request.urlopen(url.rstrip('/')+script).read();assert js==(r/'client/dist'/script.lstrip('/')).read_bytes()
query=subprocess.run(['node','crates/oh_server/tests/save_query.cjs',url],cwd=r,capture_output=True,text=True);(o/'initial-query.stdout').write_text(query.stdout);(o/'initial-query.stderr').write_text(query.stderr)
record={'pid':p.pid,'exe':str(exe),'exe_sha':hashlib.sha256(exe.read_bytes()).hexdigest(),'cmd':cmd,'url':url,'js':script,'js_sha':hashlib.sha256(js).hexdigest(),'save_sha':hashlib.sha256(save.read_bytes()).hexdigest(),'query_exit':query.returncode}
(o/'identity.json').write_text(json.dumps(record,indent=2));print(json.dumps(record))
