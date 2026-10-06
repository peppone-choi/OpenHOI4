import pathlib,subprocess,json,shutil,hashlib
p=pathlib.Path('target/wp11-independent');cli=pathlib.Path('target/debug/oh_cli.exe').resolve();saved=(p/'own-native/saved.ohsave').resolve();root=(p/'own-pack').resolve();old=hashlib.sha256(saved.read_bytes()).hexdigest();cases=[]
def run(name,pack,force,expected,load=saved,out=None,ticks='0'):
 cmd=[str(cli),'resume','--load',str(load),'--pack',str(pack),'--ticks',ticks,'--hash-out']+(['--force']if force else [])+(['--save-out',str(out)]if out else [])
 r=subprocess.run(cmd,text=True,capture_output=True);assert (r.returncode==0)==expected,(name,r.stdout,r.stderr);assert hashlib.sha256(saved.read_bytes()).hexdigest()==old
 cases.append(dict(name=name,command=cmd,exit=r.returncode,stdout=r.stdout,stderr=r.stderr,old_sha=old));print(name,'exit',r.returncode,r.stderr.strip())
def variant(name,file,change):
 pack=(p/('pack-'+name)).resolve();shutil.copytree(root,pack,dirs_exist_ok=True);f=pack/file;f.write_text(change(f.read_text()));return pack
loc=variant('localization','localisation/en/national.ftl',lambda t:t+'\n# Independent localization-only change\n')
ver=variant('version','manifest.toml',lambda t:t.replace('version = "0.1.0"','version = "0.1.1"'))
for name,pack in [('localization',loc),('version',ver)]:run(name,pack,False,False);run(name+' force',pack,True,True)
defs=variant('defines','scenarios/m1/defines.toml',lambda t:t.replace('500, 200','501, 200'))
start=variant('start','scenarios/m1/scenario.toml',lambda t:t.replace('2000-02-28','2000-02-27'))
identity=variant('id','manifest.toml',lambda t:t.replace('id = "testland"','id = "other"'))
map_=variant('definitions','maps/testland/visuals.toml',lambda t:t.replace('185','184'))
for name,pack in [('defines',defs),('start',start),('id',identity),('definitions',map_)]:run(name,pack,True,False)
corrupt=p/'corrupt.ohsave';b=bytearray(saved.read_bytes());b[-1]^=1;corrupt.write_bytes(b)
for force in [False,True]:run('checksum '+str(force),root,force,False,corrupt)
run('output self-reference',root,False,False,out=root/'bad.ohsave');assert not (root/'bad.ohsave').exists()
paused=p/'native/run-1/paused.ohsave';orig=hashlib.sha256(paused.read_bytes()).hexdigest();run('paused advance atomic file',p/'native/run-1/pack',False,False,paused,paused,'24');assert hashlib.sha256(paused.read_bytes()).hexdigest()==orig
run('paused zero advance',p/'native/run-1/pack',False,True,paused)
(p/'force-file-cases.json').write_text(json.dumps(cases,indent=2));print('all force/file preservation cases',len(cases))
