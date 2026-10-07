"""Actual registered legacy-nation CLI/host boundaries, separate immutable evidence."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
from pack_startup_native import serve, sha

ROOT = Path(__file__).resolve().parents[3]


def invoke(out, name, args, expected):
    p = subprocess.Popen(args, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = p.communicate()
    (out / f'{name}.stdout').write_bytes(stdout); (out / f'{name}.stderr').write_bytes(stderr)
    r = dict(command=args, cwd=str(ROOT), pid=p.pid, exit=p.returncode, expected_exit=expected,
             passed=p.returncode==expected)
    (out / f'{name}.command.json').write_text(json.dumps(r,indent=2)+'\n')
    return r


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--red', action='store_true')
    parser.add_argument('--red-save', type=Path)
    args=parser.parse_args(); out=args.out.resolve(); out.mkdir(parents=True,exist_ok=False)
    cli=ROOT/'target/debug'/('oh_cli.exe' if sys.platform=='win32' else 'oh_cli')
    normal=(ROOT/'data/packs/testland/scenarios/m1/nations/NTH.toml').read_text(encoding='utf-8')
    cases=[('normal',None),('syntax','invalid [ syntax\n')]
    if not args.red:
        cases += [('type',normal.replace('id = 1', "id = '1'")),
                  ('range',normal.replace('capital = 10','capital = -1')),
                  ('unknown',normal.replace('capital = 10','unknown = 10')),
                  ('tag',normal.replace('tag = "NTH"','tag = "STH"')),
                  ('ideology',normal.replace('"0.75"','"-0.75"')),
                  ('reference',normal.replace('capital = 10','capital = 65535')),
                  ('valid-unselected',normal)]
    receipts=[]
    for name,nation in cases:
        root=out/name; pack=root/'testland'; shutil.copytree(ROOT/'data/packs/testland',pack)
        folder=pack/'scenarios/legacy'; (folder/'nations').mkdir(parents=True)
        (folder/'scenario.toml').write_text("start_date='2000-01-01'\n",encoding='utf-8',newline='\n')
        (folder/'defines.toml').write_text('[time]\ninitial_speed=1\nspeed_ms_per_tick=[500,200,80,25,0]\n',encoding='utf-8',newline='\n')
        save=root/'restore.ohsave'
        if nation is not None:(folder/'nations/NTH.toml').write_text(nation,encoding='utf-8',newline='\n')
        base=[str(cli),'run','--pack',str(pack),'--scenario','m1','--ticks','0','--seed','7']
        if name=='normal' or args.red:
            r=invoke(out,name+'-save-producer',base+['--save-out',str(save)],0); assert r['passed']
        else:
            assert args.red_save and args.red_save.is_file()
            shutil.copyfile(args.red_save,save)
        before={p.relative_to(pack).as_posix():sha(p) for p in pack.rglob('*') if p.is_file()}; save_before=sha(save)
        expected=0 if name=='normal' else 1
        receipts.append(invoke(out,name+'-validate',[str(cli),'validate','--deny-warnings',str(pack)],expected))
        receipts.append(invoke(out,name+'-headless',base+['--hash-out'],expected))
        new=root/'new.ohsave'; existing=root/'existing.ohsave'; shutil.copyfile(save,existing); existing_before=sha(existing)
        receipts.append(invoke(out,name+'-save-new',base+['--save-out',str(new)],expected))
        receipts.append(invoke(out,name+'-save-existing',base+['--save-out',str(existing)],expected))
        for mode,load,force in [('startup',None,False),('restore',save,False),('restore-force',save,True)]:
            receipts.append(serve(root,out,name+'-'+mode,load,force,name=='normal','NTH.toml' if nation else None))
        assert before=={p.relative_to(pack).as_posix():sha(p) for p in pack.rglob('*') if p.is_file()} and sha(save)==save_before
        if name!='normal' and not args.red:
            assert not new.exists() and sha(existing)==existing_before
    (out/'results.json').write_text(json.dumps(receipts,indent=2)+'\n')
    print(json.dumps(dict(total=len(receipts),passed=sum(r['passed'] for r in receipts))))
    return int(any(not r['passed'] for r in receipts))


if __name__=='__main__':raise SystemExit(main())
