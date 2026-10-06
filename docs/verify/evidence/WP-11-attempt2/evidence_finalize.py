import json,pathlib,zipfile,hashlib,datetime,subprocess
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';d=r/'docs/worklog/evidence/WP-11';m=json.loads((d/'raw-output.sha256.json').read_text());raw=(d/'raw-output.zip').read_bytes();assert hashlib.sha256(raw).hexdigest()==m['archive_sha256'];z=zipfile.ZipFile(d/'raw-output.zip');assert all(hashlib.sha256(z.read(n)).hexdigest()==s for n,s in m['members'].items());assert b'unresolved imports' in z.read('req-red.txt') and b'unknown argument --save-out' in z.read('native-red.txt');(o/'initial-redgreen-audit.json').write_text(json.dumps({'members':len(m['members']),'archive_sha':m['archive_sha256'],'SAV01_red':'unresolved API imports compile failure','SAV02_red':'unknown --save-out actual native assertion failure'},indent=2))
commands=[json.loads(line) for line in (o/'commands.jsonl').read_text().splitlines()];rawsha='01689407e75ae80780f9919b5e774c0ec21dac2e13ded4404b56faa7b0417a72';assert all(x['raw_before']==x['raw_after']==rawsha for x in commands)
indexfiles=list(o.glob('*.index'));assert all(hashlib.sha256(p.read_bytes()).hexdigest()==rawsha for p in indexfiles);parent=pathlib.Path('E:/openhoi/.orchestrator/evidence/WP11.verify2.before.index');assert parent.read_bytes()==(o/'start.index').read_bytes()
# Evidence manifest, not a verification report or tracked documentation write.
manifest=[]
for p in o.rglob('*'):
 if not p.is_file() or 'mirror-build' in p.parts or 'probe-build' in p.parts or 'node_modules' in p.parts or 'pack-junction' in p.parts:continue
 manifest.append([p.relative_to(o).as_posix(),hashlib.sha256(p.read_bytes()).hexdigest()])
(o/'evidence-sha-manifest.json').write_text(json.dumps(manifest,indent=2));print(json.dumps({'command_records':len(commands),'raw_checkpoints':len(indexfiles),'rawsha':rawsha,'first_raw_change':None,'artifacts_hashed':len(manifest),'kst':datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=9))).isoformat()}))
