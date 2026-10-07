"""Same-job native comparison of two separately built library sources."""
import argparse, hashlib, io, json, os, platform, subprocess, sys, tarfile, tomllib, stat
from pathlib import Path
from compare import compare_samples

ROOT=Path(__file__).resolve().parents[1]
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def prepare_metadata(label,tree,driver,command,policy):
    source_lock=(tree/'Cargo.lock').read_bytes()
    # Metadata needs archives for all resolved platforms, including dependencies
    # absent from a host-only build cache. Fill that cache using the immutable
    # source workspace lock before normalizing the new external workspace lock.
    command(label+'-fetch',['cargo','fetch','--locked','--manifest-path',str(tree/'Cargo.toml')],policy['build_timeout_seconds'])
    if (tree/'Cargo.lock').read_bytes()!=source_lock:
        raise ValueError('source lock changed during locked cache preparation')
    command(label+'-metadata',['cargo','metadata','--offline','--format-version','1','--manifest-path',str(driver/'Cargo.toml')],policy['build_timeout_seconds'])
    if (tree/'Cargo.lock').read_bytes()!=source_lock:
        raise ValueError('source lock changed during external metadata preparation')
    metadata=json.loads((driver.parent/(label+'-metadata.stdout')).read_bytes()) if (driver.parent/(label+'-metadata.stdout')).exists() else None
    if metadata is not None:
        for package in metadata['packages']:
            if package.get('source') is None:
                path=Path(package['manifest_path']).resolve()
                if not path.is_relative_to(tree.resolve()) and path != (driver/'Cargo.toml').resolve():
                    raise ValueError('external driver escaped source path dependency: '+str(path))
        for name in ('oh_cli','oh_core'):
            linked=[p for p in metadata['packages'] if p['name']==name]
            if len(linked)!=1 or Path(linked[0]['manifest_path']).resolve()!=(tree/'crates'/name/'Cargo.toml').resolve():
                raise ValueError('external driver library linkage changed: '+name)
    original={(p['name'],p['version'],p.get('source')):p.get('checksum') for p in tomllib.loads(source_lock.decode())['package'] if p.get('source')}
    resolved=tomllib.loads((driver/'Cargo.lock').read_text(encoding='utf-8'))['package']
    for package in resolved:
        if package.get('source') and original.get((package['name'],package['version'],package['source'])) != package.get('checksum'):
            raise ValueError('external driver changed source registry resolution: '+package['name'])
def regular_path(path):
    # lstat on every ancestor rejects Windows junction/reparse links as well.
    for item in [path, *path.parents]:
        info=item.lstat()
        if stat.S_ISLNK(info.st_mode) or getattr(info,'st_file_attributes',0) & getattr(stat,'FILE_ATTRIBUTE_REPARSE_POINT',0x400):
            raise ValueError('nonregular benchmark path: '+str(item))
    return path.resolve(strict=True)

def pack_inventory(pack):
    root=regular_path(pack)
    if not root.is_dir(): raise ValueError('nonregular benchmark pack root')
    files=[]; directories=[]
    for path in sorted(pack.rglob('*')):
        resolved=regular_path(path)
        if not resolved.is_relative_to(root): raise ValueError('benchmark pack reference escape')
        info=path.lstat(); relative=path.relative_to(pack).as_posix()
        if stat.S_ISDIR(info.st_mode): directories.append(relative)
        elif stat.S_ISREG(info.st_mode): files.append({'path':relative,'bytes':info.st_size,'sha256':sha(path)})
        else: raise ValueError('nonregular benchmark pack member: '+relative)
    if not files: raise ValueError('empty benchmark pack')
    return {'path':str(root),'directories':directories,'files':files}

def verify_pack(pack,expected):
    if pack_inventory(pack)!=expected: raise ValueError('benchmark pack identity changed')

def native_sample(name,binary,pack,expected,policy,steps,command):
    verify_pack(pack,expected)
    expected_binary=sha(binary)
    result=command(name,[str(binary),str(pack),policy['scenario'],str(policy['seed']),str(steps)])
    verify_pack(pack,expected)
    if sha(binary)!=expected_binary: raise ValueError('benchmark binary identity changed')
    return result

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--out',type=Path,required=True)
    parser.add_argument('--local-evidence',action='store_true',help='allow non-Linux; never CI gate evidence')
    args=parser.parse_args()
    if platform.system()!='Linux' and not args.local_evidence: raise ValueError('Linux same-runner gate required')
    out=args.out.resolve()
    out.mkdir(parents=True,exist_ok=True)
    if not out.is_relative_to(ROOT/'target'): raise ValueError('benchmark output must be in this worktree target')
    policy=tomllib.loads((ROOT/'bench/defines.toml').read_text(encoding='utf-8'))
    if policy['pairs']!=2 or policy['threshold_percent']!=15: raise ValueError('approved policy changed')
    def command(name,cmd,timeout=policy['native_timeout_seconds'],cwd=ROOT):
        p=subprocess.run(cmd,cwd=cwd,capture_output=True,timeout=timeout,env={**os.environ,'GIT_OPTIONAL_LOCKS':'0'})
        (out/(name+'.stdout')).write_bytes(p.stdout)
        (out/(name+'.stderr')).write_bytes(p.stderr)
        (out/(name+'.json')).write_text(json.dumps({'command':cmd,'cwd':str(cwd),'native_exit':p.returncode}),encoding='utf-8')
        if p.returncode: raise ValueError(f'{name}: native exit {p.returncode}, see stderr')
        return p.stdout
    current_sha=command('current-head',['git','rev-parse','HEAD']).decode().strip()
    dirty=command('current-diff',['git','status','--porcelain']).decode().strip()
    if dirty and not args.local_evidence: raise ValueError('CI requires clean committed current source')
    baseline=policy['baseline_sha']
    if command('baseline-sha',['git','rev-parse',baseline+'^{commit}']).decode().strip()!=baseline: raise ValueError('baseline ref is not fixed exact commit')
    if current_sha==baseline and not args.local_evidence: raise ValueError('current must not be its own baseline')
    source=out/'baseline-source'
    if source.exists(): raise ValueError('use a fresh evidence directory; do not mutate prior source/evidence')
    source.mkdir()
    archive=command('baseline-archive',['git','archive','--format=tar',baseline])
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        for member in tar.getmembers():
            destination=source/member.name
            if member.issym() or member.islnk() or not destination.resolve().is_relative_to(source): raise ValueError('unsafe baseline source member')
        # Python 3.11 also lacks extractall(filter=...) on older patch releases.
        for member in tar.getmembers():
            destination=source/member.name
            if member.isdir(): destination.mkdir(parents=True,exist_ok=True)
            elif member.isfile():
                destination.parent.mkdir(parents=True,exist_ok=True)
                reader=tar.extractfile(member)
                if reader is None: raise ValueError('missing archived baseline member')
                destination.write_bytes(reader.read())
            else: raise ValueError('nonregular baseline source member')
    # Separate immutable baseline input tree; build products go into sibling driver targets.
    for p in source.rglob('*'):
        if p.is_file(): p.chmod(0o444)
    env={k:os.environ.get(k) for k in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','CARGO_BUILD_TARGET','CARGO_PROFILE_RELEASE_OPT_LEVEL','CARGO_PROFILE_RELEASE_LTO','CARGO_PROFILE_RELEASE_DEBUG','RUSTUP_TOOLCHAIN')}
    rustc=command('rustc-version',['rustc','-vV']).decode()
    cargo=command('cargo-version',['cargo','-V']).decode()
    machine={'system':platform.system(),'platform':platform.platform(),'machine':platform.machine(),'processor':platform.processor(),'python':sys.version,'cpu_count':os.cpu_count(),'environment':env,'rustc':rustc,'cargo':cargo,'profile':'release','features':'default library features','policy':policy,'current_sha':current_sha,'baseline_sha':baseline,'current_dirty':dirty,'gate_evidence':platform.system()=='Linux' and not args.local_evidence,'driver_sha256':sha(ROOT/'bench/driver.rs')}
    if Path('/proc/cpuinfo').exists(): (out/'cpuinfo.txt').write_bytes(Path('/proc/cpuinfo').read_bytes())
    (out/'environment.json').write_text(json.dumps(machine,indent=2),encoding='utf-8')
    binaries={}
    pack_root=source/policy['pack']
    frozen_pack=pack_inventory(pack_root)
    (out/'measurement-pack.json').write_text(json.dumps({'source_sha':baseline,'archive_sha256':hashlib.sha256(archive).hexdigest(),'pack':frozen_pack},indent=2),encoding='utf-8')
    machine['measurement_pack_path']=str(pack_root.resolve())
    machine['measurement_pack_source_sha']=baseline
    for label,tree in [('baseline',source),('current',ROOT)]:
        verify_pack(pack_root,frozen_pack)
        (out/(label+'-pack.json')).write_text(json.dumps(frozen_pack,indent=2),encoding='utf-8')
        driver=out/(label+'-driver'); (driver/'src').mkdir(parents=True)
        (driver/'src/main.rs').write_bytes((ROOT/'bench/driver.rs').read_bytes())
        manifest='[package]\nname="oh_bench_driver"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
        for name in ('oh_cli','oh_core'):
            manifest+=name+' = { path = '+json.dumps((tree/'crates'/name).as_posix())+' }\n'
        manifest+='serde_json = "=1.0.151"\n'
        (driver/'Cargo.toml').write_text(manifest,encoding='utf-8')
        lock=(tree/'Cargo.lock').read_text(encoding='utf-8')
        (driver/'Cargo.lock').write_text(lock,encoding='utf-8')
        # This is a new external workspace, so Cargo must add its root and prune
        # unrelated source workspace packages. Seed all resolutions from source
        # and audit that no registry package/version/checksum changed.
        (driver/'source.lock').write_text(lock,encoding='utf-8')
        prepare_metadata(label,tree,driver,command,policy)
        command(label+'-build',['cargo','build','--release','--locked','--manifest-path',str(driver/'Cargo.toml')],policy['build_timeout_seconds'])
        binary=driver/'target/release'/('oh_bench_driver.exe' if os.name=='nt' else 'oh_bench_driver')
        binaries[label]=binary
        machine[label+'_binary_sha256']=sha(binary)
        verify_pack(pack_root,frozen_pack)
    verify_pack(pack_root,frozen_pack)
    (out/'environment.json').write_text(json.dumps(machine,indent=2),encoding='utf-8')
    samples={'baseline':[],'current':[]}
    for label,tree in [('baseline',source),('current',ROOT)]:
        native_sample(label+'-warmup',binaries[label],pack_root,frozen_pack,policy,policy['warmup_steps'],command)
    for pair in range(policy['pairs']):
        for label,tree in [('baseline',source),('current',ROOT)]:
            data=native_sample(f'{label}-{pair}',binaries[label],pack_root,frozen_pack,policy,policy['steps'],command)
            samples[label].append(json.loads(data))
    failed=compare_samples(samples['baseline'],samples['current'],policy)
    result={'regression':failed,'gate_evidence':machine['gate_evidence'],'baseline_ns':[s['elapsed_ns'] for s in samples['baseline']],'current_ns':[s['elapsed_ns'] for s in samples['current']],'hash':samples['baseline'][0]['hash']}
    (out/'comparison.json').write_text(json.dumps(result,indent=2),encoding='utf-8')
    print(json.dumps(result))
    return 1 if failed else 0
if __name__=='__main__':
    try: sys.exit(main())
    except (ValueError,OSError,KeyError,subprocess.TimeoutExpired) as error:
        print('benchmark error: '+str(error),file=sys.stderr); sys.exit(2)
