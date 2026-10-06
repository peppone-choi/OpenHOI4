from pathlib import Path
import hashlib,json,zipfile
root=Path.cwd().resolve()
assert root.as_posix().lower()=='e:/openhoi/.orchestrator/wt/wp-08'
out=root/'docs/worklog/evidence/WP-08/p06-f04'
out.mkdir(parents=True,exist_ok=True)
inputs=[]
for folder in ['target/wp08-f04','target/wp08-dpr','target/wp08-f02-f03/prior-verify5-dpr']:
    inputs += [p for p in (root/folder).rglob('*') if p.is_file() and p.suffix in ['.log','.json','.png','.cjs','.py','.md']]
inputs += list((root/'target/wp08-f02-f03').glob('dpr*.log'))
prior=root/'target/wp08-f02-f03/prior-linux953-failure'
inputs += [p for p in prior.iterdir() if p.is_file()]
inputs=sorted(set(inputs))
archive=root/'target/wp08-f04/raw-originals.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED) as z:
    for p in inputs:z.writestr(p.relative_to(root).as_posix(),p.read_bytes())
    for p in (root/'target/wp08-f04/linux-phase/selected-artifact').rglob('trace.zip'):z.writestr(p.relative_to(root).as_posix(),p.read_bytes())
raw={};normalized={};encodings={}
for p in inputs:
    data=p.read_bytes();key=p.relative_to(root/'target').as_posix()
    raw[key]=hashlib.sha256(data).hexdigest()
    if p.suffix not in ['.png','.zip']:
        for encoding in (['utf-16'] if data[:2] in [b'\xff\xfe',b'\xfe\xff'] else ['utf-8-sig','cp949']):
            try:text=data.decode(encoding);break
            except UnicodeDecodeError:continue
        else:raise ValueError(f'Unknown text encoding: {p}')
        encodings[key]=encoding
        data=('\n'.join(line.rstrip() for line in text.replace('\r\n','\n').replace('\r','\n').splitlines()).rstrip()+'\n').encode('utf-8')
    dest=out/key;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(data)
    normalized[key]=hashlib.sha256(data).hexdigest()
provenance={'base_product':'9531121ee5677e43436be69364fdef4bfce2fe86','diagnostic_commit':'a15f64f7b241f7d64cce5b39ab9b95c0142e9b86','scope':'Self P06 evidence and prior CI/verifier failure originals; not fresh independent PASS or main CI success. Preview953 remains on old exact source. Pure953 Linux probe full77/78 failed; isolated2 passed. New-product Linux whole CI not executed here.','raw_archive':archive.relative_to(root).as_posix(),'raw_archive_sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),'raw_file_sha256':raw,'raw_trace_sha256':{p.relative_to(root).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'target/wp08-f04/linux-phase/selected-artifact').rglob('trace.zip')},'text_encoding':encodings,'normalization':'Text: strict declared decode, LF, trailing whitespace trim, single final LF. PNG and ZIP bytes unchanged. No expected value or error content edits. Four raw traces included only in ignored archive; 459MB original archive separately preserved with API identity/member hashes.'}
(out/'PROVENANCE.json').write_bytes((json.dumps(provenance,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))
normalized['PROVENANCE.json']=hashlib.sha256((out/'PROVENANCE.json').read_bytes()).hexdigest()
(out/'SHA256SUMS.json').write_bytes((json.dumps(normalized,ensure_ascii=False,indent=2)+'\n').encode('utf-8'))
print(json.dumps({'review_files':len(normalized),'archive_bytes':archive.stat().st_size,'archive_sha256':provenance['raw_archive_sha256']},indent=2))
