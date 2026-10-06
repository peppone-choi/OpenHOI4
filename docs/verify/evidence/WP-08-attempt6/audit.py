import hashlib,json,subprocess,pathlib,sys,datetime,os
root=pathlib.Path(__file__).resolve().parents[2]
out=root/'target/wp08-verify6'
def git(*args):return subprocess.check_output(['git','--no-optional-locks',*args],cwd=root)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
phase=sys.argv[1]
index=(root/git('rev-parse','--git-path','index').decode().strip()).resolve()
names=git('ls-files','-z'); files={n:sha(root/n) for n in names.decode().split('\0') if n}
r={'time':datetime.datetime.now(datetime.timezone.utc).isoformat(),'root':str(root),'head':git('rev-parse','HEAD').decode().strip(),'raw_index_sha256':sha(index),'semantic_index_sha256':hashlib.sha256(git('ls-files','--stage','-z')).hexdigest(),'tracked_list_sha256':hashlib.sha256(names).hexdigest(),'tracked_count':len(files),'files':files,'status':git('status','--porcelain=v1','--untracked-files=all').decode(),'staged_diff':git('diff','--cached','--binary').decode(),'unstaged_diff':git('diff','--binary').decode()}
(out/f'{phase}.json').write_text(json.dumps(r,ensure_ascii=False,indent=2),encoding='utf8')
print({k:v for k,v in r.items() if k!='files'})
