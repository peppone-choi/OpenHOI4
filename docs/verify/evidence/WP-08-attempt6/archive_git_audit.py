import pathlib,json,hashlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6';results=[]
for name in ['red62','red953']:
 identity=json.loads((OUT/'prior-attempt'/(name+'-identity.json')).read_text(encoding='utf-8-sig'));head=identity['source_commit'];paths=subprocess.check_output(['git','--no-optional-locks','ls-tree','-r','--name-only','-z',head],cwd=ROOT).decode('utf8').split('\0');paths=[p for p in paths if p and not any(p.startswith(prefix) for prefix in identity['omitted'])];listed=[f['path'] for f in identity['files']];assert sorted(paths)==sorted(listed),(name,set(paths)-set(listed),set(listed)-set(paths))
 data=subprocess.check_output(['git','--no-optional-locks','cat-file','--batch'],input=''.join(head+':'+p+'\n' for p in paths).encode('utf8'),cwd=ROOT);pos=0;mismatches=[]
 for p in paths:
  newline=data.index(b'\n',pos);header=data[pos:newline].split();assert header[1]==b'blob',header;size=int(header[2]);body=data[newline+1:newline+1+size];pos=newline+size+2;assert data[pos-1:pos]==b'\n';actual=(OUT/(name+'-source')/p).read_bytes()
  if body!=actual:mismatches.append(p)
 assert not mismatches
 results.append({'name':name,'source_commit':head,'allIncludedGitBlobsMatchBuildInputAfterBuild':True,'selected_files':len(paths),'onlyOmissions':identity['omitted'],'mismatches':mismatches})
(OUT/'archive-git-audit.json').write_text(json.dumps(results,indent=2));print(results)
