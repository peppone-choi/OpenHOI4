"""Exact MIN/F01 contexts, semantic causes and valid control on pinned native bytes."""
import argparse,hashlib,json,shutil,socket,subprocess,sys,time,urllib.request
from pathlib import Path
from lifecycle import ctrl_c
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'crates/oh_save/tools'))
from economy_current import header
from save_reference import pack_hash
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inventory(p):return [dict(path=f.relative_to(p).as_posix(),bytes=f.stat().st_size,sha256=sha(f)) for f in sorted(p.rglob('*')) if f.is_file()]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True,type=Path);parser.add_argument('--bin-dir',required=True,type=Path);parser.add_argument('--fixture-root',type=Path,default=ROOT/'tests/repro');args=parser.parse_args()
    out=args.out.resolve();assert out.is_relative_to(ROOT/'target/evidence/WP-14-M2-r3-P06-2');out.mkdir(parents=True,exist_ok=False)
    bins=args.bin_dir.resolve();cli=bins/('oh_cli.exe' if sys.platform=='win32' else 'oh_cli');server=bins/('oh_server.exe' if sys.platform=='win32' else 'oh_server');digests={str(p):sha(p) for p in (cli,server)};checks=[]
    for case,fixture,file,cause in [('min','WP-14-M2-r3-target-range','project-target-min.ohsave','economy:InvalidValue'),('valid','WP-14-M2-r3-target-range','control-valid-target2.ohsave',None),('f01','WP-14-M2-r3-restore-stage','industry-level4.ohsave','economy:TargetConflict')]:
        folder=out/case;folder.mkdir();source=args.fixture_root/fixture;shutil.copytree(source/'packs',folder/'packs');shutil.copyfile(source/file,folder/file);save=folder/file;pack=folder/'packs/testland';before=inventory(pack);expected=json.loads((source/'INPUT_MANIFEST.json').read_text())['files'];assert before==sorted(expected,key=lambda v:v['path'])
        h=header(save);assert h['packs'][0]['content_hash']==pack_hash(pack),'wrong exact context; not a semantic proof'
        for force in (False,True):
            mode='force' if force else 'normal';target=folder/(mode+'.out.ohsave');argv=[str(cli),'resume','--load',str(save),'--pack',str(pack),'--ticks','0','--hash-out','--save-out',str(target)]+(['--force'] if force else [])
            p=subprocess.Popen(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE);stdout,stderr=p.communicate(timeout=120)
            ok=(p.returncode==0 and target.exists() and stdout.decode().strip()==f"{h['state_hash']:016x}") if cause is None else (p.returncode==1 and cause.encode() in stderr and b'PackMismatch' not in stderr and b'panicked' not in stderr and not target.exists())
            receipt=dict(argv=argv,cwd=str(ROOT),pid=p.pid,exit=p.returncode,exe_sha256=sha(cli),input_sha256=sha(save),pack_hash=f"{pack_hash(pack):016x}",expected_cause=cause,passed=ok)
            (folder/f'cli-{mode}.stdout').write_bytes(stdout);(folder/f'cli-{mode}.stderr').write_bytes(stderr);(folder/f'cli-{mode}.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8');checks.append(ok)
            with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
            argv=[str(server),'--port',str(port),'--pack-root',str(folder/'packs'),'--load-save',str(save)]+(['--force'] if force else [])
            kwargs={}
            if sys.platform=='win32':
                info=subprocess.STARTUPINFO();info.dwFlags=subprocess.STARTF_USESHOWWINDOW;info.wShowWindow=0;kwargs=dict(creationflags=subprocess.CREATE_NEW_CONSOLE,startupinfo=info)
            process=subprocess.Popen(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,**kwargs);http=None;query_exit=None;stopped=False
            try:
                deadline=time.monotonic()+15
                while process.poll() is None and time.monotonic()<deadline:
                    try:
                        with urllib.request.urlopen(f'http://127.0.0.1:{port}/',timeout=.3) as response:http=response.status
                        break
                    except OSError:time.sleep(.03)
                if http is not None:
                    if cause is None:
                        query=subprocess.run(['node','crates/oh_server/tests/economy_target_query.cjs',f'ws://127.0.0.1:{port}/ws'],cwd=ROOT,capture_output=True,timeout=20);query_exit=query.returncode;(folder/f'query-{mode}.stdout').write_bytes(query.stdout);(folder/f'query-{mode}.stderr').write_bytes(query.stderr)
                    ctrl_c(process);stopped=True
                stdout,stderr=process.communicate(timeout=10)
            finally:
                if process.poll() is None:ctrl_c(process);process.communicate(timeout=10)
            ok=(process.returncode==0 and http==200 and query_exit==0) if cause is None else (process.returncode==1 and http is None and not stopped and cause.encode() in stderr and b'PackMismatch' not in stderr and b'panicked' not in stderr)
            receipt=dict(argv=argv,cwd=str(ROOT),pid=process.pid,exit=process.returncode,exe_sha256=sha(server),input_sha256=sha(save),http=http,query_exit=query_exit,expected_cause=cause,passed=ok)
            (folder/f'server-{mode}.stdout').write_bytes(stdout);(folder/f'server-{mode}.stderr').write_bytes(stderr);(folder/f'server-{mode}.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8');checks.append(ok)
            assert before==inventory(pack) and sha(save)==sha(source/file)
    assert all(sha(Path(path))==digest for path,digest in digests.items())
    result=dict(head=subprocess.check_output(['git','--no-optional-locks','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),binary_sha256=digests,checks=checks,passed=all(checks));(out/'result.json').write_text(json.dumps(result,indent=2),encoding='utf-8');print(json.dumps(result));return 0 if all(checks) else 1
if __name__=='__main__':raise SystemExit(main())
