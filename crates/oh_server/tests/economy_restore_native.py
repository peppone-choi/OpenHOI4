"""F-01 exact original bytes through actual CLI and isolated server normal/force."""
import argparse, hashlib, json, shutil, subprocess, sys
from pathlib import Path
from pack_startup_native import serve
ROOT=Path(__file__).resolve().parents[3]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def files(root):return {p.relative_to(root).as_posix():sha(p) for p in sorted(root.rglob('*')) if p.is_file()}
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True)
    out=parser.parse_args().out.resolve();assert out.is_relative_to(ROOT/'target/evidence/WP-14-M2-r3-P06');out.mkdir(parents=True,exist_ok=False)
    source=ROOT/'tests/repro/WP-14-M2-r3-restore-stage';shutil.copytree(source/'packs',out/'packs');shutil.copyfile(source/'industry-level4.ohsave',out/'industry-level4.ohsave')
    save=out/'industry-level4.ohsave';before=files(out/'packs');digest=sha(save)
    assert save.stat().st_size==458 and digest=='c67f5cefbba865be78643b920536678f789b8375b418b9034ddd2cd2b4db1e53'
    cli=ROOT/('target/debug/oh_cli.exe' if sys.platform=='win32' else 'target/debug/oh_cli');results=[]
    for force in (False,True):
        label='force' if force else 'normal';target=out/f'{label}-must-not-exist.ohsave'
        argv=[str(cli),'resume','--load',str(save),'--pack',str(out/'packs/testland'),'--ticks','0','--hash-out','--save-out',str(target)]+(['--force'] if force else [])
        p=subprocess.Popen(argv,cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE);stdout,stderr=p.communicate(timeout=120)
        receipt=dict(argv=argv,cwd=str(ROOT),pid=p.pid,exit=p.returncode,exe_sha256=sha(cli),input_sha256=digest,output_exists=target.exists(),expected_exit=1)
        (out/f'cli-{label}.stdout').write_bytes(stdout);(out/f'cli-{label}.stderr').write_bytes(stderr);(out/f'cli-{label}.receipt.json').write_text(json.dumps(receipt,indent=2),encoding='utf-8')
        results.append(p.returncode==1 and not target.exists())
        receipt=serve(out/'packs',out,f'server-{label}',save,force,False)
        results.append(receipt['passed'])
        assert files(out/'packs')==before and sha(save)==digest
    record=dict(head=subprocess.check_output(['git','--no-optional-locks','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),dirty=bool(subprocess.check_output(['git','--no-optional-locks','status','--porcelain'],cwd=ROOT).strip()),cwd=str(ROOT),input_sha256=digest,input_preserved=True,pack_preserved=True,checks=results,passed=all(results))
    (out/'result.json').write_text(json.dumps(record,indent=2),encoding='utf-8');print(json.dumps(record));return 0 if all(results) else 1
if __name__=='__main__':raise SystemExit(main())
