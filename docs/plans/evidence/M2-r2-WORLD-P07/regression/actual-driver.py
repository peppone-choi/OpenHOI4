"""Task-local P07 commands; run only after independent exact PASS integration."""
import argparse,datetime,hashlib,json,os,pathlib,re,subprocess,sys
p=argparse.ArgumentParser();p.add_argument('head');p.add_argument('label');a=p.parse_args()
r=pathlib.Path('E:/openhoi'); out=r/'.orchestrator/evidence'/a.label
assert subprocess.check_output(['git','-C',str(r),'rev-parse','HEAD'],text=True).strip()==a.head
assert not subprocess.check_output(['git','--no-optional-locks','-C',str(r),'status','--porcelain'],text=True), 'P07 starts from the exact clean merged main; custody archives come after capture.'
assert not out.exists(), 'Keep earlier evidence; choose a fresh label.'
out.mkdir(parents=True); env=dict(os.environ);env['PATH']='C:/Users/user/.cargo/bin;'+env['PATH'];env['PYTHONUTF8']='1';env['GIT_OPTIONAL_LOCKS']='0';env['TRIGGER_TEST_CAPTURE']=str(out/'trigger-current');env['TRIGGER_PUBLIC_EVIDENCE']=str(out/'trigger-public')
(out/'actual-driver.py').write_bytes(pathlib.Path(__file__).read_bytes())
cargo='C:/Users/user/.cargo/bin/cargo.exe';py=sys.executable
commands=[('npm-ci',['npm.cmd','--prefix','client','ci'],0),('client-test',['npm.cmd','--prefix','client','test'],0),('client-build',['npm.cmd','--prefix','client','run','build'],0),('fmt',[cargo,'fmt','--check'],0),('clippy',[cargo,'clippy','--workspace','--all-targets','--locked','--','-D','warnings'],0),('workspace-tests',[cargo,'test','--workspace','--locked'],0),('typecheck',['npm.cmd','--prefix','client','run','typecheck'],0),('client-licenses',['npm.cmd','--prefix','client','run','licenses'],0),('assets',[py,'-X','utf8','tools/check_assets.py','--release'],0),('policy-tests',[py,'-X','utf8','-m','unittest','discover','-s','tools/tests','-p','test_wp01.py','-v'],0),('docs',[py,'-X','utf8','tools/check_docs.py'],0),('localisation',[py,'-X','utf8','tools/check_localisation.py'],0),('locale-regressions',[py,'-X','utf8','-m','unittest','discover','-s','tools/tests','-p','test_localisation.py','-v'],0),('current-validate',[cargo,'run','-p','oh_cli','--locked','--','validate','--deny-warnings','data/packs/testland'],0),('legacy-strict-negative',[cargo,'run','-p','oh_cli','--locked','--','validate','data/packs/examples/m0/testland'],1)]
commands.extend([('architecture',[py,'-X','utf8','tools/check_architecture.py'],0),('rust-licenses',[cargo,'deny','check','licenses','bans','advisories'],0)])
for name,pack in [('m0',None),('m1','data/packs/testland')]:
 for i in (1,2):
  cmd=[cargo,'run','-p','oh_cli','--locked','--','run']
  if pack:cmd.extend(['--pack',pack])
  cmd.extend(['--scenario','testland' if name=='m0' else 'm1','--days','365','--seed','1','--hash-out'])
  commands.append((f'{name}-hash-{i}',cmd,0))
commands.append(('native-build',[cargo,'build','--workspace','--locked'],0))
commands.append(('v1-original-capture',[py,'-X','utf8','crates/oh_save/tools/check_save_determinism.py','capture','--out',str(out/'v1')],0))
commands.append(('native-server-guards',[py,'-X','utf8','crates/oh_server/tests/pack_startup_native.py','--out',str(out/'native-server')],0))
commands.append(('native-legacy-nations',[py,'-X','utf8','crates/oh_server/tests/legacy_nations_native.py','--out',str(out/'native-legacy-nations'),'--red-save',str(out/'native-server/normal/start.ohsave')],0))
commands.append(('original-save-evidence-tests',[py,'-X','utf8','-m','unittest','discover','-s','crates/oh_save/tests','-p','test_save_evidence.py','-v'],0))
commands.append(('current-save-evidence-tests',[py,'-X','utf8','-m','unittest','discover','-s','crates/oh_save/tests','-p','test_save_current_evidence.py','-v'],0))
commands.append(('current-save-capture',[py,'-X','utf8','crates/oh_save/tools/check_save_current.py','capture','--out',str(out/'current-save')],0))
for force,name in [(False,'server'),(True,'server-force')]:
 cmd=[py,'-X','utf8','crates/oh_server/tests/save_native.py','--capture',str(out/'current-save'),'--out',str(out/'current-save'/name)]
 if force:cmd.append('--force')
 commands.append(('current-'+name,cmd,0))
commands.append(('historical-server-negative',[py,'-X','utf8','crates/oh_server/tests/save_historical_native.py','--capture',str(out/'v1'),'--out',str(out/'v1/server-rejected')],0))
local_checker="import importlib.util,json,pathlib,sys; sys.path.insert(0,str(pathlib.Path('crates/oh_save/tools').resolve())); s=importlib.util.spec_from_file_location('actual_current','crates/oh_save/tools/check_save_current.py'); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); folder=pathlib.Path(sys.argv[1]); result=m.verify_folder(folder); m.verify_servers(folder,result); print(json.dumps({'head':result['head'],'scope':'one actual local platform; 3OS comparison remains CI-only','mode':result['evidence_mode'],'runs':result['runs']}))"
commands.append(('current-local-evidence-check',[py,'-X','utf8','-c',local_checker,str(out/'current-save')],0))
for name,version in [('movement',2),('strait',3)]:
 for i in (1,2):commands.append((f'{name}-capture-{i}',[cargo,'run','--quiet','-p','oh_save','--locked','--example',f'{name}_fixture','--','capture',str(out/f'{name}-{i}')],0))
 commands.append((f'{name}-resume',[cargo,'run','--quiet','-p','oh_save','--locked','--example',f'{name}_fixture','--','resume','data/packs/testland',str(out/f'{name}-1/{name}-v{version}.ohsave')],0))
commands.append(('trigger-current-capture',[py,'-X','utf8','crates/oh_sim/tools/check_trigger_determinism.py','capture','--out',str(out/'trigger-current')],0))
commands.append(('trigger-evidence-gates',[py,'-X','utf8','-m','unittest','discover','-s','crates/oh_sim/tests','-p','test_trigger_evidence.py','-v'],0))
rows=[];outputs={};passed=True
for name,cmd,expected in commands:
 t=datetime.datetime.now(datetime.timezone.utc).isoformat();proc=subprocess.run(cmd,cwd=r,env=env,capture_output=True,encoding='utf-8',errors='replace');outputs[name]=proc.stdout
 (out/f'{name}.stdout').write_text(proc.stdout,encoding='utf-8');(out/f'{name}.stderr').write_text(proc.stderr,encoding='utf-8')
 row=dict(name=name,cmd=cmd,cwd=str(r),started_utc=t,native_exit=proc.returncode,expected_exit=expected,passed=proc.returncode==expected)
 rows.append(row);(out/'commands.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(name,proc.returncode,flush=True)
 if not row['passed']:passed=False;break
checks={}
try:
 if passed:
  for name in ('m0','m1'):
   vals=[re.findall(r'^[0-9a-f]{16}$',outputs[f'{name}-hash-{i}'],re.M) for i in (1,2)]
   assert len(vals[0])==1 and vals[0]==vals[1], vals
   checks[name+'_hash']=vals[0][0]
  assert checks['m0_hash']=='b039d35666b77fc2';assert checks['m1_hash']=='b595dc2a1e5b4f8c'
  for name,version in [('movement',2),('strait',3)]:
   vals=[json.loads((out/f'{name}-{i}/capture.json').read_text(encoding='utf-8')) for i in (1,2)];resume=json.loads(outputs[name+'-resume']);assert vals[0]==vals[1]==resume
   src=(out/f'{name}-1/{name}-v{version}.ohsave').read_bytes();assert src==(out/f'{name}-2/{name}-v{version}.ohsave').read_bytes()
   checks[name]=dict(full_dto_canonical_hash_equal=True,save_sha256=hashlib.sha256(src).hexdigest(),split_hash=vals[0]['split']['hash'],continuous_hash=vals[0]['continuous']['hash'])
  (out/'semantic-checks.json').write_text(json.dumps(checks,indent=2)+'\n',encoding='utf-8')
except Exception as exc:
 passed=False;checks['error']=repr(exc);(out/'semantic-failure.json').write_text(json.dumps(checks,indent=2)+'\n',encoding='utf-8')
assert subprocess.check_output(['git','-C',str(r),'rev-parse','HEAD'],text=True).strip()==a.head
summary=dict(head=a.head,passed=passed,commands=len(rows),planned=len(commands),semantic_checks=checks,scope='WP13 integration after exact independent PASS; old None/v1/v2/v3 + new actual flags/checkpoint v4; exact main all old+Trigger CI and actual3OS remain separate')
(out/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(summary));sys.exit(0 if passed else 1)
