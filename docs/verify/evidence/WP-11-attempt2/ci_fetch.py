import subprocess,json,pathlib,hashlib,zipfile,io,urllib.request,tomllib
r=pathlib.Path.cwd();o=r/'target/wp11-verify2/ci';o.mkdir(exist_ok=True);repo='repos/peppone-choi/OpenHOI4'
def api(url,name):
 p=subprocess.run(['gh','api',url],cwd=r,capture_output=True);(o/name).write_bytes(p.stdout);assert p.returncode==0,p.stderr.decode();return p.stdout
run=json.loads(api(repo+'/actions/runs/37536767684','run.json'));assert run['head_sha']=='750c2732b00c164ddc252fa26c4984ababd64f35';assert run['conclusion']=='success'
art=json.loads(api(repo+'/actions/runs/37536767684/artifacts?per_page=100','artifacts.json'));reports=[]
for a in art['artifacts']:
 if not a['name'].startswith('save-'):continue
 raw=api(repo+f'/actions/artifacts/{a["id"]}/zip',a['name']+'.zip');sha=hashlib.sha256(raw).hexdigest();assert a['digest']=='sha256:'+sha,(a['name'],a['digest'],sha)
 dest=o/'three'/a['name'];dest.mkdir(parents=True,exist_ok=True)
 z=zipfile.ZipFile(io.BytesIO(raw));z.extractall(dest)
 # v7 archive option encloses the original directory in an inner ZIP
 nested=list(dest.glob('*.zip'))
 if len(nested)==1:
  zz=zipfile.ZipFile(nested[0]);zz.extractall(dest);nested[0].unlink()
 reports.append({'name':a['name'],'id':a['id'],'digest':a['digest'],'actual_sha':sha,'source_head':run['head_sha'],'result':json.loads((dest/'result.json').read_text())})
assert len(reports)==3
(o/'digests.json').write_text(json.dumps(reports,indent=2));api(repo+'/actions/runs/37536767681','general-ci.json');api(repo+'/actions/runs/37536767681/jobs?per_page=100','general-jobs.json')
print(json.dumps({'save_run':run['id'],'source_head':run['head_sha'],'artifacts':[{'name':x['name'],'id':x['id'],'digest':x['digest']} for x in reports]}))
# New dependencies compared with pre-WP11 baseline, official immutable version metadata.
old=tomllib.loads(subprocess.check_output(['git','--no-optional-locks','-C',str(r),'show','bbf8762922cf74c3b54f6274c1bff83b0977da62:Cargo.lock']).decode());new=tomllib.loads((r/'Cargo.lock').read_text());oldkeys={(p['name'],p['version']) for p in old['package']};deplist=[]
for p in new['package']:
 if (p['name'],p['version']) in oldkeys or 'checksum' not in p:continue
 url=f'https://crates.io/api/v1/crates/{p["name"]}/{p["version"]}'
 req=urllib.request.Request(url,headers={'User-Agent':'OpenHOI4 independent verification'});raw=urllib.request.urlopen(req,timeout=30).read();v=json.loads(raw)['version'];assert v['checksum']==p['checksum'];(o/f'dep-{p["name"]}.json').write_bytes(raw);deplist.append({'url':url,'name':p['name'],'version':p['version'],'license':v['license'],'checksum':v['checksum']})
(o/'dependency-direct.json').write_text(json.dumps(deplist,indent=2));print(json.dumps(deplist))
