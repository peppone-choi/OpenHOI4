import copy,hashlib,json,pathlib,subprocess,sys,os
R=pathlib.Path.cwd(); O=R/'target/verify-ai-policy-r1'; F=O/'independent-fixtures'; F.mkdir(exist_ok=True)
results=[]
def val(x):
 if isinstance(x,dict):return '{ '+', '.join(k+' = '+val(v) for k,v in x.items())+' }'
 if isinstance(x,bool):return str(x).lower()
 return json.dumps(x,ensure_ascii=False)
def test(name,mutate=None,expected=1,diag='',release=True):
 root=F/name; (root/'assets').mkdir(parents=True); (root/'docs').mkdir();
 (root/'assets/p.png').write_bytes(b'Independent synthetic image; no real AI generation')
 (root/'docs/terms.txt').write_bytes(b'Synthetic terms; not service approval')
 (root/'docs/review.txt').write_bytes(b'Synthetic review; no human has approved distribution')
 sha=lambda p:hashlib.sha256((root/p).read_bytes()).hexdigest()
 rec=dict(path='assets/p.png',author='Independent fixture',source='synthetic',license='CC-BY-SA-4.0',modified=False,ai_generated=True,notes='Fixture only',ai_provenance=dict(tool='Invented tool / version 0 (synthetic)',usage_terms_path='docs/terms.txt',usage_terms_sha256=sha('docs/terms.txt')),distribution_review=dict(status='approved',reviewer='Invented reviewer (synthetic)',reviewed_at='2026-10-06T21:00:00+09:00',evidence_path='docs/review.txt',evidence_sha256=sha('docs/review.txt'),asset_sha256=sha('assets/p.png')))
 if mutate:mutate(rec,root)
 (root/'assets/ASSETS.toml').write_text('[[asset]]\n'+'\n'.join(k+' = '+val(v) for k,v in rec.items())+'\n',encoding='utf-8')
 cmd=[sys.executable,str(R/'tools/check_assets.py'),'--root',str(root)]+(['--release'] if release else [])
 p=subprocess.run(cmd,cwd=R,capture_output=True,text=True,encoding='utf-8'); ok=p.returncode==expected and diag in p.stdout+p.stderr
 results.append(dict(name=name,command=cmd,expected=expected,exit=p.returncode,diagnostic=diag,ok=ok,stdout=p.stdout,stderr=p.stderr))
 print(name,'PASS' if ok else 'FAIL','exit',p.returncode,flush=True)
def setfield(t,k,v):return lambda rec,root:rec[t].__setitem__(k,v)
def delfield(t,k):return lambda rec,root:rec[t].pop(k)
test('reviewed-ai',expected=0)
test('nonai-release',lambda r,p:r.update(ai_generated=False),0)
test('unreviewed-release',lambda r,p:(r.pop('distribution_review'),r.pop('ai_provenance')),1,'AI-generated')
test('unreviewed-dev',lambda r,p:(r.pop('distribution_review'),r.pop('ai_provenance')),0,release=False)
fields={'ai_provenance':['tool','usage_terms_path','usage_terms_sha256'],'distribution_review':['status','reviewer','reviewed_at','evidence_path','evidence_sha256','asset_sha256']}
for table,keys in fields.items():
 test(table+'-missing',lambda r,p,t=table:r.pop(t),1,table)
 for n,v in enumerate([False,'wrong',[]]):test(table+'-wrongtable-'+str(n),lambda r,p,t=table,v=v:r.update({t:v}),1,table)
 for k in keys:
  test(k+'-missing',delfield(table,k),1,k)
  for n,v in enumerate([False,123,[],{},'', '   ']):test(k+'-wrongtype-'+str(n),setfield(table,k,v),1,k)
for v in ['pending','rejected','Approved','false']:
 test('status-'+v,setfield('distribution_review','status',v),1,'status')
for n,v in enumerate(['2026-10-06','2026-10-06T09:00:00','2026-02-30T09:00:00Z','2026-10-06T24:00:00Z','2026-10-06T09:00:60Z','2026-10-06T09:00:00+24:00','2026-10-06T09:00:00+09:60','2026-10-06T09:00:00.1Z','0000-01-01T00:00:00Z']):
 test('timestamp-bad-'+str(n),setfield('distribution_review','reviewed_at',v),1,'reviewed_at')
for n,v in enumerate(['2026-10-06T09:00:00Z','2024-02-29T00:00:00-05:30','2099-01-01T00:00:00+09:00']):test('timestamp-good-'+str(n),setfield('distribution_review','reviewed_at',v),0)
for table,key in [('ai_provenance','usage_terms_path'),('distribution_review','evidence_path')]:
 for n,v in enumerate(['docs/missing.txt','docs','../outside.txt','/outside.txt','C:/outside.txt','https://example.com/review','docs\\review.txt','docs/./review.txt','docs//review.txt','docs/../docs/review.txt']):test(key+'-path-'+str(n),setfield(table,key,v),1,key)
 test(key+'-empty',lambda r,p,t=table,k=key:(p/r[t][k]).write_bytes(b''),1,key)
for table,key in [('ai_provenance','usage_terms_sha256'),('distribution_review','evidence_sha256'),('distribution_review','asset_sha256')]:
 for n,v in enumerate(['0'*64,'A'*64,'a'*63,'g'*64]):test(key+'-hash-'+str(n),setfield(table,key,v),1,key)
for path,key in [('assets/p.png','asset_sha256'),('docs/terms.txt','usage_terms_sha256'),('docs/review.txt','evidence_sha256')]:test(key+'-changed',lambda r,p,path=path:(p/path).write_bytes(b'changed independently'),1,key)
test('forbidden-license',lambda r,p:r.update(license='CC-BY-NC-4.0'),1,'unapproved license')
test('missing-asset',lambda r,p:r.update(path='assets/missing.png'),1,'missing file')
test('unregistered',lambda r,p:(p/'assets/unregistered.png').write_bytes(b'fixture'),1,'unregistered')
test('asset-path-traversal',lambda r,p:r.update(path='../outside.png'),1,'relative')
test('nonimage-ai',lambda r,p:((p/'assets/p.png').rename(p/'assets/p.txt'),r.update(path='assets/p.txt')),1,'image')
test('nonai-font-bad',lambda r,p:((p/'assets/p.png').rename(p/'assets/p.woff2'),r.update(path='assets/p.woff2',ai_generated=False,license='CC0-1.0')),1,'font license')
test('nonai-font-ofl',lambda r,p:((p/'assets/p.png').rename(p/'assets/p.woff2'),r.update(path='assets/p.woff2',ai_generated=False,license='OFL-1.1')),0)
# Windows junctions use the same resolved-path boundary check as symbolic links.
outside=F/'outside'; outside.mkdir(); (outside/'outside.txt').write_bytes(b'outside fixture')
linknotes=[]
for table,key in [('ai_provenance','usage_terms_path'),('distribution_review','evidence_path')]:
 def link(r,p,t=table,k=key):
  dest=p/'docs/link';
  try:dest.symlink_to(outside,target_is_directory=True); linknotes.append({'field':k,'kind':'symlink'})
  except OSError as e:
   cp=subprocess.run(['cmd.exe','/c','mklink','/J',str(dest),str(outside)],capture_output=True,text=True)
   if cp.returncode:raise RuntimeError(cp.stdout+cp.stderr)
   linknotes.append({'field':k,'kind':'junction','symlink_error':str(e)})
  r[t][k]='docs/link/outside.txt'
 test(key+'-resolved-outside',link,1,'resolves outside root')
(O/'independent-results.json').write_text(json.dumps({'results':results,'linknotes':linknotes},ensure_ascii=False,indent=2),encoding='utf-8')
print('TOTAL',len(results),'FAILED',sum(not x['ok'] for x in results),'LINKS',linknotes)
sys.exit(int(any(not x['ok'] for x in results)))
