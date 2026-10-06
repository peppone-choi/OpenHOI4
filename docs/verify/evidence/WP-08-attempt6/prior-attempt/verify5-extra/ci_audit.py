import run_checks as m,subprocess,json,pathlib,hashlib,zipfile,io
repo='peppone-choi/OpenHOI4';out=m.OUT/'ci953';out.mkdir(exist_ok=True)
def api(path):return subprocess.check_output(['gh','api','repos/'+repo+'/'+path],cwd=m.ROOT)
def save(name,b):(out/name).write_bytes(b)
summary=[]
for label,rid in [('ci',37504540992),('core',37504541035),('sim',37504540929)]:
 run=api(f'actions/runs/{rid}');save(label+'-run.json',run);r=json.loads(run);assert r['head_sha']=='9531121ee5677e43436be69364fdef4bfce2fe86'
 jobs=api(f'actions/runs/{rid}/jobs?per_page=100');save(label+'-jobs.json',jobs)
 log=api(f'actions/runs/{rid}/logs');save(label+'-full-logs.zip',log)
 with zipfile.ZipFile(io.BytesIO(log)) as z:
  for n in z.namelist():assert '..' not in pathlib.PurePosixPath(n).parts and not pathlib.PurePosixPath(n).is_absolute()
  z.extractall(out/(label+'-logs'))
 artifacts=api(f'actions/runs/{rid}/artifacts?per_page=100');save(label+'-artifacts.json',artifacts);items=[]
 for a in json.loads(artifacts)['artifacts']:
  b=api(f"actions/artifacts/{a['id']}/zip");name=f"{label}-{a['name']}";save(name+'.zip',b);sha=hashlib.sha256(b).hexdigest();assert 'sha256:'+sha==a['digest'],(a['name'],sha,a['digest'])
  with zipfile.ZipFile(io.BytesIO(b)) as z:
   for n in z.namelist():assert '..' not in pathlib.PurePosixPath(n).parts and not pathlib.PurePosixPath(n).is_absolute()
   z.extractall(out/name)
  items.append({'name':a['name'],'id':a['id'],'bytes':len(b),'zipSHA':sha,'digest':a['digest']})
 summary.append({'label':label,'run':rid,'head':r['head_sha'],'status':r['status'],'conclusion':r['conclusion'],'jobs':[(j['name'],j['id'],j['conclusion']) for j in json.loads(jobs)['jobs']],'artifacts':items,'logsZipSHA':hashlib.sha256(log).hexdigest()});print(json.dumps(summary[-1]),flush=True)
 (out/'audit-summary.json').write_text(json.dumps(summary,indent=2))
