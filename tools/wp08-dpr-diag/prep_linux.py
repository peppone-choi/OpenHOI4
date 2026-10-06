from pathlib import Path
import argparse,hashlib,json,os,re,shutil,subprocess
from checks import SOURCE_HEAD,STRICT_SHA,PROBE_SHA,ORIGINAL_SHA,git_read,tracked_state,parse_enumeration,compare_enumerations
parser=argparse.ArgumentParser();parser.add_argument('--root');parser.add_argument('--output');args=parser.parse_args();kit=Path(__file__).resolve().parent;root=Path(args.root).resolve() if args.root else kit.parents[1];out=Path(args.output).resolve() if args.output else kit
assert out.is_relative_to(root/'target'),out
before=None
try:
    before=tracked_state(root);head=before['head'];status=before['status'];assert head==SOURCE_HEAD and not status,(head,status)
    out.mkdir(parents=True,exist_ok=True);(out/'preparation-product-state-before.json').write_text(json.dumps(before,indent=2)+'\n',encoding='utf-8')
    paths=['client/src','client/e2e-m1','client/playwright.config.ts','client/playwright.m1.config.ts','client/package-lock.json','crates/oh_server','data/packs/testland','.github/workflows/ci.yml']
    diff=git_read(root,'diff','9fbc05a1e29062f93bb362450aaddc2068aa41e9',head,'--name-only','--',*paths);assert not diff,diff.decode()
    source=(root/'client/e2e-m1/map-dpr.spec.ts').read_bytes();assert hashlib.sha256(source).hexdigest()==ORIGINAL_SHA
    assert hashlib.sha256((kit/'candidate-template.ts').read_bytes()).hexdigest()==STRICT_SHA
    assert hashlib.sha256((kit/'probe-template.ts').read_bytes()).hexdigest()==PROBE_SHA
    out.mkdir(parents=True,exist_ok=True)
    def rel(path,base):return os.path.relpath(path,base).replace(os.sep,'/')
    profile=(kit/'profile-template.ts').read_text(encoding='utf-8').replace('../../client/node_modules/@playwright/test',rel(root/'client/node_modules/@playwright/test',out));(out/'profile.ts').write_text(profile,encoding='utf-8')
    identity={'source_head':head,'tracked_status_before':status,'original_fixture_sha256':hashlib.sha256(source).hexdigest(),'preparation_adaptations':[]}
    for mode in ['probe','candidate']:
        folder=out/('full-'+mode);assert not folder.exists();shutil.copytree(root/'client/e2e-m1',folder)
        for file in folder.glob('*.ts'):
            text=file.read_text(encoding='utf-8');text=text.replace("from '@playwright/test'",f"from '{rel(root/'client/node_modules/@playwright/test',folder)}'").replace("from '@msgpack/msgpack'",f"from '{rel(root/'client/node_modules/@msgpack/msgpack',folder)}'").replace("from '../src/proto/protocol'",f"from '{rel(root/'client/src/proto/protocol',folder)}'").replace('import.meta.dirname',json.dumps(str(folder)));file.write_text(text,encoding='utf-8')
        text=(kit/(mode+'-template.ts')).read_text(encoding='utf-8').replace('../../../client/',rel(root/'client',folder)+'/').replace("from '../profile'",f"from '{rel(out/'profile',folder)}'")
        # Observe the same worker instead of extending a worker fixture, which could restart it.
        text=text.replace('begin(info,force);','begin(info,force,browser);').replace('await context.close();finish(info);',"phase('context-close-start');await context.close();phase('context-close-end');")
        (folder/'map-dpr.spec.ts').write_text(text,encoding='utf-8')
        config="const path=require('node:path');const base=require(BASE).default;module.exports={...base,testDir:DIR,outputDir:path.join(process.env.OH_DPR_PROFILE,'test-results'),reporter:[['line'],['json',{outputFile:path.join(process.env.OH_DPR_PROFILE,'report.json')}]]};\n".replace('BASE',json.dumps(str(root/'client/playwright.m1.config.ts'))).replace('DIR',json.dumps(str(folder)))
        (out/('full-'+mode+'.config.cjs')).write_text(config,encoding='utf-8')
        identity['preparation_adaptations'].append({'mode':mode,'fixture_sha256':hashlib.sha256(text.encode()).hexdigest(),'scope':'Copy all original test files/baselines; change only import paths, DPR Node timing hooks and ignored candidate diff. Original eight testMatch entries, 41 cases per browser, order, base worker and timeouts retained. Context.close remains inside original test function.'})
    (out/'linux-preparation-identity.json').write_text(json.dumps(identity,indent=2)+'\n',encoding='utf-8')
    enumerations=[]
    for mode,config in [('original',str(root/'client/playwright.m1.config.ts')),('probe',str(out/'full-probe.config.cjs')),('candidate',str(out/'full-candidate.config.cjs'))]:
        env=os.environ.copy();env.pop('OH_BROWSER_PRODUCTS',None);env.update(OH_TEST_PORT='19471',OH_DPR_PROFILE=str(out/('list-'+mode)))
        command=['node','node_modules/@playwright/test/cli.js','test','--config',config,'--project=chromium','--project=webkit','--list'];result=subprocess.run(command,cwd=root/'client',env=env,capture_output=True,text=True,encoding='utf-8');(out/(mode+'-list.log')).write_text(result.stdout+result.stderr,encoding='utf-8');assert result.returncode==0,result.stderr
        names=parse_enumeration(result.stdout);enumerations.append({'mode':mode,'command':command,'real_exit_code':result.returncode,'ordered_project_file_titles':names,'discovered_files':sorted({c['file'] for c in names})})
    proof=compare_enumerations([e['ordered_project_file_titles'] for e in enumerations]);proof['enumerations']=enumerations
    (out/'enumeration-parity.json').write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8');print(json.dumps(identity,indent=2));print('original/probe/candidate: same82 ordered project/file/title entries and discovered eight-file set',proof['discovered_files'])
finally:
    active_error=__import__('sys').exc_info()[1];after=None;after_error=None
    try:
        after=tracked_state(root);out.mkdir(parents=True,exist_ok=True)
        (out/'preparation-product-state-after.json').write_text(json.dumps(after,indent=2)+'\n',encoding='utf-8')
    except BaseException as failure:after_error=repr(failure)
    unchanged=before is not None and before==after
    (out/'preparation-result.json').write_text(json.dumps({'source_state_unchanged':unchanged,'preparation_error':repr(active_error) if active_error else None,'source_after_error':after_error},indent=2)+'\n',encoding='utf-8')
    if active_error is None:assert unchanged,'Source changed during preparation/collection, including raw index'
