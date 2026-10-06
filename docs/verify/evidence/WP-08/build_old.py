import sys,tarfile,pathlib,subprocess,json
from run import ROOT,OUT,ENV,NPM,CARGO,run
commit='f58c0c61322287f48318bce47ca51deb7daebcc9'
archive=OUT/'f58-source.tar';old=OUT/'f58-source';old.mkdir(exist_ok=True)
args=['git','archive','--format=tar','--output',str(archive),commit,'Cargo.toml','Cargo.lock','rust-toolchain.toml','deny.toml','crates','client','data','assets','tools']
if run('old-archive',args):sys.exit(1)
with tarfile.open(archive) as tar:
 for member in tar.getmembers():
  path=(old/member.name).resolve()
  if not path.is_relative_to(old.resolve()) or member.issym() or member.islnk():raise RuntimeError('unsafe member')
 tar.extractall(old)
ENV['CARGO_TARGET_DIR']=str(OUT/'f58-build')
for name,args in [('old-npm-ci',NPM+['--prefix','client','ci']),('old-client-build',NPM+['--prefix','client','run','build']),('old-server-build',[CARGO,'build','--locked','-p','oh_server'])]:
 if run(name,args,cwd=old):sys.exit(1)
(OUT/'old-build-identity.json').write_text(json.dumps({'commit':commit,'source':'git archive exact commit, no source mutation','archive':str(archive),'exe':str(OUT/'f58-build/debug/oh_server.exe')},indent=2),encoding='utf-8')
