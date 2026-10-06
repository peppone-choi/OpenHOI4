import os,sys,pathlib,subprocess,json,hashlib,time
root=pathlib.Path(__file__).resolve().parents[2]; out=root/'target/wp11-verify2'
os.environ['GIT_OPTIONAL_LOCKS']='0';os.environ['PATH']='C:/Users/user/.cargo/bin;'+os.environ['PATH']
def git(*a): return subprocess.check_output(['git','--no-optional-locks','-C',str(root),*a])
index=pathlib.Path(git('rev-parse','--path-format=absolute','--git-path','index').decode().strip())
def snapshot(label):
 b=index.read_bytes();(out/(label+'.index')).write_bytes(b);return hashlib.sha256(b).hexdigest()
steps=json.loads(pathlib.Path(sys.argv[1]).read_text())
for name,command in steps:
 before=snapshot(name+'.before');started=time.time()
 with (out/(name+'.log')).open('wb') as f:
  p=subprocess.run(command,cwd=root,stdout=f,stderr=subprocess.STDOUT)
 after=snapshot(name+'.after')
 result=dict(name=name,command=command,cwd=str(root),exit=p.returncode,seconds=time.time()-started,raw_before=before,raw_after=after)
 with (out/'commands.jsonl').open('a') as f:f.write(json.dumps(result)+'\n')
 print(json.dumps(result),flush=True)
