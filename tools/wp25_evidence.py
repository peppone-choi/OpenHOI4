"""Package selected original WP-25 evidence and exact committed source bytes."""
import hashlib, json, os, shutil, subprocess, sys, zipfile, zlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'target/evidence/WP-25-M2-r2'
BASE='81deb803944cf6297f13b3f14ca454774cea141b'
def digest(path):
    h=hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda:stream.read(1024*1024),b''): h.update(chunk)
    return h.hexdigest()
def git(label,args):
    p=subprocess.run(['git',*args],cwd=ROOT,capture_output=True,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'},timeout=120)
    (OUT/(label+'.stdout')).write_bytes(p.stdout); (OUT/(label+'.stderr')).write_bytes(p.stderr)
    (OUT/(label+'.json')).write_text(json.dumps({'cwd':str(ROOT),'command':['git',*args],'native_exit':p.returncode}),encoding='utf-8')
    if p.returncode: raise ValueError('git '+label+' failed')
    return p.stdout
def inventory(paths): return [{'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'sha256':digest(p)} for p in paths]
def selected(tree):
    for directory,dirs,files in os.walk(tree,followlinks=False):
        directory=Path(directory)
        dirs[:]=[d for d in dirs if d not in ('target','baseline-source','__pycache__') and not (directory/d).is_symlink() and not (getattr((directory/d).lstat(),'st_file_attributes',0)&0x400)]
        for f in files:
            p=directory/f
            if f in ('evidence.zip','ZIP-IDENTITY.json','SHA256SUMS.json'): continue
            if p.is_symlink() or (getattr(p.lstat(),'st_file_attributes',0)&0x400): continue
            yield p
def main():
    OUT.mkdir(parents=True,exist_ok=True)
    if (OUT/'evidence.zip').exists(): raise ValueError('do not replace an existing evidence archive')
    head=git('final-head',['rev-parse','HEAD']).decode().strip()
    status=git('final-status',['status','--porcelain=v1']).decode()
    if status.strip(): raise ValueError('exact source must be committed and clean')
    changed=git('final-changed',['diff','--name-only',BASE,'HEAD','-z']).decode().split('\0')
    tracked=git('final-tracked',['ls-files','-z']).decode().split('\0')
    tracked=[ROOT/p for p in tracked if p]
    (OUT/'tracked-files.json').write_text(json.dumps({'source_commit':head,'files':inventory(tracked)},ensure_ascii=False,indent=2),encoding='utf-8')
    git('final-index',['ls-files','--stage','-z'])
    git('final-diff',['diff',BASE,'HEAD','--'])
    source=OUT/'source-files'
    for name in changed:
        if not name: continue
        original=ROOT/name
        if not original.is_file(): raise ValueError('unexpected deleted changed source')
        destination=source/name
        destination.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(original,destination)
    (OUT/'changed-files.json').write_text(json.dumps({'source_commit':head,'files':inventory([ROOT/p for p in changed if p])},ensure_ascii=False,indent=2),encoding='utf-8')
    files=[]
    prior=ROOT/'target/evidence/WP-25'
    if prior.exists(): files += [(p,'prior/'+p.relative_to(prior).as_posix()) for p in selected(prior)]
    files += [(p,p.relative_to(OUT).as_posix()) for p in selected(OUT)]
    files.sort(key=lambda x:x[1])
    rows=[{'path':name,'bytes':p.stat().st_size,'sha256':digest(p)} for p,name in files]
    sums=OUT/'SHA256SUMS.json'; sums.write_text(json.dumps({'source_commit':head,'files':rows},ensure_ascii=False,indent=2),encoding='utf-8')
    files.append((sums,'SHA256SUMS.json'))
    artifact=OUT/'evidence.zip'
    with zipfile.ZipFile(artifact,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as archive:
        for p,name in files: archive.write(p,name)
    identities=[]; original_zips=[]
    with zipfile.ZipFile(artifact) as archive:
        if len(archive.infolist())!=len(files): raise ValueError('member count mismatch')
        for p,name in files:
            raw=p.read_bytes(); member=archive.getinfo(name); actual=archive.read(name)
            h=hashlib.sha256(raw).hexdigest(); crc=zlib.crc32(raw)&0xffffffff
            if actual!=raw or member.CRC!=crc or member.file_size!=len(raw): raise ValueError('raw member mismatch '+name)
            identities.append({'path':name,'bytes':len(raw),'sha256':h,'crc32':f'{crc:08x}'})
            # Includes native recorded ZIP bytes kept in target-after.bin snapshots.
            if p.suffix=='.zip' or raw.startswith(b'PK\x03\x04'):
                original_zips.append({'path':name,'bytes':len(raw),'sha256':h})
    result={'source_commit':head,'baseline_sha':BASE,'archive':str(artifact),'bytes':artifact.stat().st_size,'sha256':digest(artifact),'member_count':len(identities),'verified_raw_bytes_sha_crc':True,'original_zip_count':len(original_zips),'original_zips':original_zips,'members':identities}
    (OUT/'ZIP-IDENTITY.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
    print(json.dumps({k:v for k,v in result.items() if k not in ('members','original_zips')},ensure_ascii=False))
if __name__=='__main__':
    try: main()
    except (ValueError,OSError,subprocess.TimeoutExpired) as error: print(str(error),file=sys.stderr); sys.exit(1)
