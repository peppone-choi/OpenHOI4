import pathlib,json,hashlib,zipfile,subprocess,sys
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';sys.path.insert(0,str(r/'crates/oh_server/tests'));from lifecycle import ctrl_c
id=json.loads((o/'ui/identity.json').read_text());actual=json.loads((o/'ui/actual-process.json').read_text(encoding='utf-8-sig'));assert actual['ProcessId']==id['pid'] and actual['ExecutablePath']==id['exe'];assert hashlib.sha256(pathlib.Path(id['exe']).read_bytes()).hexdigest()==id['exe_sha'];wire=json.loads((o/'own-ui-wire-saved-session.log').read_text());assert wire['welcome']['packs'][0]['hash']=='7a35613aa8869cab';assert 'LISTENING       32816' in (o/'ui/listener-netstat.txt').read_text(encoding='utf-8-sig')
# Only this session's explicitly recorded server receives Ctrl+C.
class Owned:pid=id['pid']
ctrl_c(Owned());import time,socket
for _ in range(40):
 try:s=socket.create_connection(('127.0.0.1',3800),timeout=.2);s.close();time.sleep(.1)
 except OSError:break
else:raise RuntimeError('own listener did not stop')
assert b'shutdown complete' in (o/'ui/stdout').read_bytes().lower() or b'shut' in (o/'ui/stdout').read_bytes().lower() or not subprocess.run(['tasklist','/FI',f'PID eq {id["pid"]}'],capture_output=True).stdout.decode(errors='replace').find('oh_server.exe')>=0
(o/'ui/cleanup.json').write_text(json.dumps({'own_pid':id['pid'],'CtrlC_sent':True,'listener_gone':True,'stdout':(o/'ui/stdout').read_text()},indent=2))
# Source/canonical fixture/public API and original source prefix remained unchanged.
src=(o/'transaction-original-source.rs').read_bytes();mir=(o/'mirror/crates/oh_save/src/file.rs').read_bytes();assert mir[:len(src)]==src
assert hashlib.sha256((r/'crates/oh_save/tests/fixtures/m1-v1.ohsave').read_bytes()).hexdigest()=='57b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa'
p=subprocess.run(['git','--no-optional-locks','-C',str(r),'diff','--name-only','227c1a2e606ccbb0cbc9712b0dd7a6e15ffadb30','HEAD','--','Cargo.lock','Cargo.toml','tests/golden','client','.github','crates/oh_cli','crates/oh_save/tests','crates/oh_save/src/lib.rs'],capture_output=True);assert p.returncode==0 and p.stdout==b''
# Three exact source servers were real native saved hosts and exited cleanly.
rows=[]
for folder in sorted((o/'ci/three').iterdir()):
 sv=json.loads((folder/'server/result.json').read_text());assert sv['head']=='750c2732b00c164ddc252fa26c4984ababd64f35';assert sv['query_exit']==sv['server_exit']==0;assert '--load-save' in sv['server_command'];assert sv['pack_hash']=='b27b821484f4c43b';assert sv['served_js_sha256']==id['js_sha'];assert sv['query']['snapshot']['state']['tick']=='48';assert sv['query']['snapshot']['state']['paused'] is True;assert sv['query']['state']['state']['speed']==2;assert sv['query']['state']['state']['tick']=='48';rows.append({k:sv[k] for k in ['head','url','server_pid','exe_sha256','served_js_sha256','pack_hash','query_exit','server_exit']})
(o/'ci/three-native-server-audit.json').write_text(json.dumps(rows,indent=2));print(json.dumps({'own_server_cleanup':id['pid'],'same_source_transaction_prefix':True,'public_format_fixture_unchanged':True,'three_os_native_servers':rows}))
