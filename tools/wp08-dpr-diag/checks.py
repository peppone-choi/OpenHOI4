"""Read-only guards for the single, temporary WP08 diagnostic experiment."""
from pathlib import Path
import hashlib,os,re,subprocess
from urllib.parse import unquote

SOURCE_HEAD='b73f0ce344138d5fb3909eca5d43c786aac49b15'
STRICT_SHA='c9002e018ad536cc48308973a5d3400d69a8925c9732e936bdef30144ae429ac'
PROBE_SHA='03d5680b91fa1b0140a35a47f89cb33e215ea192af306f9d87e6c936bd48ae34'
ORIGINAL_SHA='c1779231a04ea665b71812d90c6d666577411cdec85cb266513bef63aa0f077a'

def git_read(root,*args):
    env=os.environ.copy();env['GIT_OPTIONAL_LOCKS']='0'
    return subprocess.check_output(['git',*args],cwd=root,env=env)

def tracked_state(root):
    index_path=Path(git_read(root,'rev-parse','--path-format=absolute','--git-path','index').decode().strip())
    raw=index_path.read_bytes();names=git_read(root,'ls-files','-z');index=git_read(root,'ls-files','--stage','-z')
    return {'head':git_read(root,'rev-parse','HEAD').decode().strip(),'raw_index_path':str(index_path),'raw_index_bytes':len(raw),'raw_index_sha256':hashlib.sha256(raw).hexdigest(),'index_entries_sha256':hashlib.sha256(index).hexdigest(),'tracked_names_sha256':hashlib.sha256(names).hexdigest(),'tracked_files':[{'path':name.decode(),'sha256':hashlib.sha256((root/name.decode()).read_bytes()).hexdigest()} for name in names.split(b'\0') if name],'status':git_read(root,'status','--porcelain').decode(),'unstaged_diff_sha256':hashlib.sha256(git_read(root,'diff','--binary')).hexdigest(),'staged_diff_sha256':hashlib.sha256(git_read(root,'diff','--cached','--binary')).hexdigest()}

def parse_enumeration(output):
    cases=[]
    clean=re.sub(r'\x1b\[[0-?]*[ -/]*[@-~]','',output)
    for line in clean.splitlines():
        match=re.match(r'^\s*\[([^]]+)\] › (.+?):\d+:\d+ › (.+)$',line)
        if not match:continue
        project,location,title=match.groups();basename=location.replace('\\','/').rsplit('/',1)[-1]
        cases.append({'project':project,'file':basename,'title':title})
    return cases

def compare_enumerations(enumerations):
    assert len(enumerations)==3,'Require original, probe and candidate lists'
    files=sorted({c['file'] for c in enumerations[0]})
    assert len(files)==8,('Original discovered file set',files)
    for cases in enumerations:
        assert len(cases)==82,('Expected full M1 enumeration',len(cases))
        assert len({(c['project'],c['file'],c['title']) for c in cases})==len(cases),'Duplicate case'
        assert sorted({c['file'] for c in cases})==files,'Discovered file set changed'
        assert cases==enumerations[0],'Ordered project/file/title changed'
    return {'ordered_project_file_titles_equal':True,'count':len(enumerations[0]),'file_count':len(files),'discovered_files':files,'per_project_files':{p:sorted({c['file'] for c in enumerations[0] if c['project']==p}) for p in sorted({c['project'] for c in enumerations[0]})}}

def require_same_bytes(actual,expected,reason):
    proof={'actual_bytes':len(actual),'expected_bytes':len(expected),'actual_sha256':hashlib.sha256(actual).hexdigest(),'expected_sha256':hashlib.sha256(expected).hexdigest(),'equal':actual==expected}
    assert proof['equal'],(reason,proof)
    return proof

def dist_asset_path(dist,asset):
    assert asset.startswith('/assets/'),('Unexpected served asset URL',asset)
    path=(dist/unquote(asset).lstrip('/')).resolve();assert path.is_relative_to(dist.resolve()),('Asset outside source dist',asset)
    return path
