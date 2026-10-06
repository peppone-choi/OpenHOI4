import pathlib,subprocess,json,hashlib,urllib.request
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');o=r/'target/wp12-verify2'
def git(*a):return subprocess.check_output(['git','-C',str(r),*a])
old='a91d324e3c644cb6cb8e44247181c21fd40b3f0e';base='docs/worklog/evidence/WP-12/'
paths=git('ls-tree','-r','--name-only',old,'--',base).decode().splitlines()
changes=[];hashes={}
for p in paths:
 before=git('show',old+':'+p);now=git('show',':'+p)
 if now!=before:changes.append(p)
 hashes[p]=hashlib.sha256(now).hexdigest()
data={'original_commit':old,'count_including_manifest':len(paths),'count_excluding_manifest':len(paths)-1,'changed_paths':changes,'index_SHA256':hashes}
(o/'original-58-integrity.json').write_text(json.dumps(data,indent=2));print('Original paths',len(paths),'including SHA256SUMS.json; changes',changes);assert not changes and len(paths)==58
for name,url in [('gpl','https://raw.githubusercontent.com/spdx/license-list-data/main/text/GPL-3.0-or-later.txt'),('cc','https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt')]:
 try:
  b=urllib.request.urlopen(url,timeout=20).read();(o/(name+'-legal-original.txt')).write_bytes(b)
  local=r/('client/licenses/'+('GPL-3.0.txt' if name=='gpl' else 'CC-BY-SA-4.0.txt'))
  print(name,'upstream bytes',len(b),'equal-index',b==git('show',':'+local.relative_to(r).as_posix()),'equal-LF-normalized',b.replace(b'\r\n',b'\n')==git('show',':'+local.relative_to(r).as_posix()))
 except Exception as e:print(name,'download limitation',repr(e))
