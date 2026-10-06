import socket,subprocess,pathlib,json,os,hashlib
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';os.environ['GIT_OPTIONAL_LOCKS']='0';os.environ['PATH']='C:/Users/user/.cargo/bin;'+os.environ['PATH']
idx=pathlib.Path(subprocess.check_output(['git','--no-optional-locks','-C',str(r),'rev-parse','--path-format=absolute','--git-path','index']).decode().strip())
for name,cfg in [('m0-e2e-cwd','playwright.config.ts'),('m1-e2e-cwd','playwright.m1.config.ts')]:
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 os.environ['OH_TEST_PORT']=str(port);os.environ['OH_BROWSER_PRODUCTS']='1'
 b=idx.read_bytes();(o/(name+'.before.index')).write_bytes(b)
 cmd=['node','node_modules/@playwright/test/cli.js','test','--config',cfg]
 with (o/(name+'.log')).open('wb') as f:p=subprocess.run(cmd,cwd=r/'client',stdout=f,stderr=subprocess.STDOUT)
 a=idx.read_bytes();(o/(name+'.after.index')).write_bytes(a)
 record=dict(name=name,command=cmd,port=port,exit=p.returncode,raw_before=hashlib.sha256(b).hexdigest(),raw_after=hashlib.sha256(a).hexdigest())
 with (o/'commands.jsonl').open('a') as f:f.write(json.dumps(record)+'\n')
 print(json.dumps(record),flush=True)
