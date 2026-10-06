import run_checks as m,hashlib,json,subprocess,pathlib,re,socket,urllib.request
out=m.OUT
copy=json.loads((out/'prior-attempt/copied-source-SHA.json').read_text(encoding='utf-8'));bad=[]
for x in copy:
 p=out/'prior-attempt'/x['path']
 if hashlib.sha256(p.read_bytes()).hexdigest()!=x['sha256']:bad.append(x['path'])
assert not bad,bad
index=subprocess.check_output(['git','ls-files','--stage','-z'],cwd=m.ROOT);head=subprocess.check_output(['git','ls-tree','-rz','HEAD'],cwd=m.ROOT)
stages={};trees={}
for entry in index.decode().split('\0'):
 if entry:
  info,path=entry.split('\t');mode,sha,stage=info.split();assert stage=='0';stages[path]=(mode,sha)
for entry in head.decode().split('\0'):
 if entry:
  info,path=entry.split('\t');mode,kind,sha=info.split();trees[path]=(mode,sha)
assert stages==trees
(out/'semantic-index-stage.txt').write_bytes(index);(out/'committed-tree.txt').write_bytes(head)
tcp=subprocess.check_output(['netstat','-ano','-p','tcp']).decode();listen=[l for l in tcp.splitlines() if 'LISTENING' in l and re.search(r':1944[34]\s',l)];assert not listen,listen
for port in [19443,19444]:
 with socket.socket() as s:s.bind(('127.0.0.1',port))
cmds=[json.loads(l) for l in (out/'commands.jsonl').read_text().splitlines()];tests=(out/'workspace-test.log').read_text(encoding='utf-8');total=sum(int(n) for n in re.findall(r'test result: ok\. (\d+) passed;',tests))
metadata=[]
for name,url in [('three','https://registry.npmjs.org/three/0.186.1'),('types-three','https://registry.npmjs.org/@types%2fthree/0.186.0'),('playwright','https://registry.npmjs.org/@playwright%2ftest/1.63.0')]:
 try:
  with urllib.request.urlopen(url,timeout=15) as response:b=response.read();status=response.status
  (out/(name+'-registry-original.json')).write_bytes(b);doc=json.loads(b);metadata.append({'url':url,'status':status,'version':doc['version'],'license':doc['license'],'SHA256':hashlib.sha256(b).hexdigest()})
 except Exception as e:metadata.append({'url':url,'error':str(e)})
b=json.loads((out/'before.json').read_text());a=json.loads((out/'after.json').read_text())
summary={'head':a['head'],'files':len(a['tracked']),'trackedListsAndSHAsEqual':a['tracked']==b['tracked'],'stagedAndUnstagedDiffsEqual':a['staged']==b['staged'] and a['unstaged']==b['unstaged'],'statusesEqual':a['status']==b['status']=='','rawIndexSHAequal':a['indexSHA']==b['indexSHA'],'rawIndexBefore':b['indexSHA'],'rawIndexAfter':a['indexSHA'],'semanticIndexAllEntriesEqualCommittedHEAD':stages==trees,'semanticIndexSHA':hashlib.sha256(index).hexdigest(),'committedTree':m.git('rev-parse','HEAD^{tree}'),'priorManifestItemsChecked':len(copy),'priorManifestMismatch':bad,'ownPortsReleased':[19443,19444],'listeners':listen,'workspaceTestCount':total,'commandsRecorded':len(cmds),'registryChecks':metadata,'noEarlierFAILorCurrentLinuxFAILSuperseded':True,'exact953Verdict':'FAIL'}
(out/'final-audit.json').write_text(json.dumps(summary,indent=2));print(json.dumps(summary,indent=2))
# Exclude rebuild source/dependencies/build cache; the original archive and initial
# extracted source digest document them. Preserve all independent evidence bytes.
files={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in out.rglob('*') if p.is_file() and 'red62-source' not in p.relative_to(out).parts and '__pycache__' not in p.relative_to(out).parts and p.name!='SHA256SUMS.json'}
(out/'SHA256SUMS.json').write_text(json.dumps(files,indent=2))
