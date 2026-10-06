import pathlib,subprocess,json,hashlib,zipfile,sys,re
from oracle import decode
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6';dest=OUT/'independent-ci';dest.mkdir(exist_ok=True)
GH=r'C:\Program Files\GitHub CLI\gh.exe';REPO='repos/peppone-choi/OpenHOI4';HEAD='9fbc05a1e29062f93bb362450aaddc2068aa41e9'
def api(endpoint,name):
 data=subprocess.check_output([GH,'api',REPO+'/'+endpoint],cwd=ROOT);(dest/(name+'.json')).write_bytes(data);return json.loads(data)
result={'runs':[],'artifacts':[]}
for run in [37513851532,37513851556,37513851414]:
 r=api('actions/runs/'+str(run),'run'+str(run));j=api('actions/runs/'+str(run)+'/jobs?per_page=100','jobs'+str(run));assert r['head_sha']==HEAD and r['conclusion']=='success';assert all(x['conclusion']=='success' for x in j['jobs']);result['runs'].append({'id':run,'head':r['head_sha'],'event':r['event'],'head_branch':r['head_branch'],'jobs':[(x['name'],x['id'],x['conclusion']) for x in j['jobs']]})
log=subprocess.check_output([GH,'api',REPO+'/actions/jobs/112441600818/logs'],cwd=ROOT);(dest/'client-native-job.log').write_bytes(log);text=log.decode('utf8');result['case_summary']=re.findall(r'(?:Running \d+ tests? using \d+ workers?|\d+ passed \([^)]+\))',text)
for id in [11437106228,11437065536,11436935567,11436596439]:
 a=api('actions/artifacts/'+str(id),'artifact'+str(id));z=dest/(a['name']+'.zip')
 with z.open('wb') as f:subprocess.run([GH,'api',REPO+'/actions/artifacts/'+str(id)+'/zip'],cwd=ROOT,stdout=f,check=True)
 digest=hashlib.sha256(z.read_bytes()).hexdigest();assert a['digest']=='sha256:'+digest and a['workflow_run']['head_sha']==HEAD
 parent=OUT/'prior-attempt/current9fbc-ci'/('actual-client/original-artifact.zip' if id==11437106228 else 'three-CI-and-M1/'+a['name']+'.zip');assert parent.read_bytes()==z.read_bytes()
 with zipfile.ZipFile(z) as archive:
  for member in archive.infolist():
   p=pathlib.PurePosixPath(member.filename);assert not p.is_absolute() and '..' not in p.parts and not ((member.external_attr>>16)&0o170000)==0o120000
  extracted=dest/a['name'];archive.extractall(extracted)
 result['artifacts'].append({'id':id,'name':a['name'],'ZIP_SHA256':digest,'bytes':z.stat().st_size,'members':len(archive.namelist()),'same_parent_original':True})
 (dest/'audit.json').write_text(json.dumps(result,indent=2),encoding='utf8')
print(json.dumps(result,indent=2))
