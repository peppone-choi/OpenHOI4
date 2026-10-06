import subprocess,pathlib,json,hashlib,os,re
root=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-10-verify');out=root/'target/verify/WP-10';parent='1bfea48511c6b75efb0dd26ee6e980a90d77da04'
def git(*args):return subprocess.check_output(['git',*args],cwd=root).decode('utf8')
changed=git('diff','--name-only',parent,'HEAD').splitlines()
policy={'parent':parent,'head':git('rev-parse','HEAD').strip(),'changed_paths':changed,'existing_tests_changed':[p for p in changed if p.startswith(('tests/','crates/')) and '/tests/' in p and git('ls-tree',parent,p).strip()], 'client_changed':[p for p in changed if p.startswith('client/')], 'rules_docs_changed':[p for p in changed if p in ['docs/01-game-design.md','docs/02-production-spec.md']], 'golden_changed':[p for p in changed if p.startswith('tests/golden/')], 'asset_data_changed':[p for p in changed if p.startswith(('assets/','data/'))], 'runtime_files':{}, 'official_sources':[{'url':'https://docs.rs/crate/proptest/1.11.0','verified':'package/version 1.11.0'},{'url':'https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html','verified':'cases/rng_seed/failure_persistence API'},{'url':'https://docs.rs/fixed/1.31.0/fixed/struct.FixedI64.html#method.checked_mul','verified':'checked multiplication API'}]}
for p in ['crates/oh_sim/src/ledger.rs','crates/oh_sim/src/formula.rs']:
 src=(root/p).read_text(encoding='utf8');lines=[line for line in src.splitlines() if not line.strip().startswith('//')]
 policy['runtime_files'][p]={'forbidden_tokens':[(i+1,l) for i,l in enumerate(lines) if re.search(r'\b(f32|f64|HashMap|HashSet|thread_rng|SystemTime)\b|std::(fs|io|time)|rand::',l)],'sha256':hashlib.sha256((root/p).read_bytes()).hexdigest()}
policy['dependency_delta']=git('diff',parent,'HEAD','--','Cargo.lock','crates/oh_sim/Cargo.toml')
(out/'source-audit.json').write_text(json.dumps(policy,ensure_ascii=False,indent=2),encoding='utf8')
fixture=out/'red-fixture';(fixture/'src').mkdir(parents=True,exist_ok=True);(fixture/'tests').mkdir(exist_ok=True)
for p in git('ls-tree','-r','--name-only',parent,'crates/oh_sim/src').splitlines():
 dest=fixture/pathlib.Path(p).relative_to('crates/oh_sim');dest.parent.mkdir(parents=True,exist_ok=True);dest.write_bytes(subprocess.check_output(['git','show',parent+':'+p],cwd=root))
(fixture/'tests/ledger.rs').write_bytes((root/'crates/oh_sim/tests/ledger.rs').read_bytes())
(fixture/'Cargo.toml').write_text('''[package]
name="oh_sim"
version="0.0.0"
edition="2024"
[workspace]
[dependencies]
oh_core={path="../../../../crates/oh_core"}
oh_data={path="../../../../crates/oh_data"}
serde={version="=1.0.229",features=["derive"]}
[dev-dependencies]
proptest={version="=1.11.0",default-features=false,features=["std"]}
''',encoding='utf8')
os.environ['PATH']=r'C:/Users/user/.cargo/bin;'+os.environ['PATH'];os.environ['CARGO_TARGET_DIR']=str(root/'target')
cmd=['cargo','test','--offline','--manifest-path','target/verify/WP-10/red-fixture/Cargo.toml','--test','ledger']
with (out/'red-reproduction.log').open('w',encoding='utf8') as f:
 f.write('Independent parent-source copy under ignored target only; current tests unmodified.\nparent='+parent+'\ncommand: '+subprocess.list2cmdline(cmd)+'\n');f.flush();p=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT);f.write('\nexit: '+str(p.returncode)+'\n')
text=(out/'red-reproduction.log').read_text(encoding='utf8')
assert p.returncode==101 and 'E0432' in text and 'E0425' in text
print('Independent RED reproduction exit 101: missing ledger/stat_value confirmed. Source audit recorded.')
