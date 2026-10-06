from run import *
import zipfile,io,tomllib,urllib.request
def api(endpoint,name,binary=False):
 b=subprocess.check_output(['gh','api',endpoint],cwd=ROOT,env=os.environ);(OUT/name).write_bytes(b)
 return b if binary else json.loads(b)
runs=api('repos/peppone-choi/OpenHOI4/actions/runs?head_sha='+BASE['head']+'&per_page=100','ci-runs-api.json')['workflow_runs']
assert len(runs)==4
for r in runs:
 assert r['head_sha']==BASE['head'] and r['conclusion']=='success' and r['status']=='completed'
 api('repos/peppone-choi/OpenHOI4/actions/runs/'+str(r['id'])+'/jobs?per_page=100',str(r['id'])+'-jobs-api.json')
save=next(r for r in runs if r['name']=='Save determinism')
arts=api('repos/peppone-choi/OpenHOI4/actions/runs/'+str(save['id'])+'/artifacts','save-artifacts-api.json')['artifacts']
downloads=[]
for a in arts:
 if not a['name'].startswith('save-'):continue
 assert a['workflow_run']['head_sha']==BASE['head'] and not a['expired']
 b=api('repos/peppone-choi/OpenHOI4/actions/artifacts/'+str(a['id'])+'/zip',a['name']+'.zip',True)
 sha=hashlib.sha256(b).hexdigest();assert a['digest']=='sha256:'+sha
 with zipfile.ZipFile(io.BytesIO(b)) as z:
  folder=OUT/'three-os'/a['name'];folder.mkdir(parents=True,exist_ok=True)
  for member in z.infolist():
   p=(folder/member.filename).resolve();assert folder.resolve()==p or folder.resolve() in p.parents
  z.extractall(folder)
  members=[dict(path=n,sha256=hashlib.sha256(z.read(n)).hexdigest()) for n in z.namelist() if not n.endswith('/')]
 downloads.append(dict(id=a['id'],name=a['name'],head=a['workflow_run']['head_sha'],api_digest=a['digest'],zip_sha256=sha,members=members))
(OUT/'three-os-downloads.json').write_text(json.dumps(downloads,indent=2),encoding='utf-8')
old=tomllib.loads(git('show','bbf8762922cf74c3b54f6274c1bff83b0977da62:Cargo.lock'))['package']
new=tomllib.loads((ROOT/'Cargo.lock').read_text())['package'];oldset={(p['name'],p['version']) for p in old};results=[]
for p in new:
 if (p['name'],p['version']) in oldset or 'checksum' not in p:continue
 url='https://crates.io/api/v1/crates/'+p['name']+'/'+p['version']
 raw=urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'OpenHOI4 independent license verification'}),timeout=30).read();j=json.loads(raw)
 (OUT/(p['name']+'-official-api.json')).write_bytes(raw)
 assert j['version']['checksum']==p['checksum'];results.append(dict(name=p['name'],version=p['version'],license=j['version']['license'],url=url,checksum=p['checksum']))
(OUT/'dependencies-official.json').write_text(json.dumps(results,indent=2),encoding='utf-8')
print(json.dumps(dict(runs=[(r['id'],r['name'],r['conclusion']) for r in runs],artifacts=[(a['name'],a['zip_sha256']) for a in downloads],new_dependencies=results)))
