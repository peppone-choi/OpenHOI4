import pathlib,subprocess,json
r=pathlib.Path.cwd();o=r/'target/wp08-verify3'
q=o/'ui-independent.cjs';text=q.read_text(encoding='utf-8');text=text.replace("'기반 시설'","'기반시설'");q.write_text(text,encoding='utf-8')
startup=subprocess.STARTUPINFO();startup.dwFlags=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
with (o/'red/87-init-server.log').open('w') as log:
 p=subprocess.Popen([str(r/'target/wp08-verify3/red/87-build/debug/oh_server.exe'),'--port','19427','--pack-root',str(r/'target/wp08-verify3/red/87/data/packs')],cwd=r,stdout=log,stderr=subprocess.STDOUT,startupinfo=startup)
 try:
  import urllib.request,time
  for _ in range(100):
   try:urllib.request.urlopen('http://127.0.0.1:19427/',timeout=.5);break
   except Exception:time.sleep(.1)
  runs=[]
  for name in ['initialization-red.cjs']:
   with (o/(name+'.log')).open('w',encoding='utf-8') as result:code=subprocess.run(['node',str(o/name)],cwd=r,stdout=result,stderr=subprocess.STDOUT).returncode
   runs.append({'script':name,'exit':code});print(name,code,flush=True);(o/'red/87-init-runner-results.json').write_text(json.dumps(runs,indent=2))
 finally:p.terminate();p.wait(timeout=10)
