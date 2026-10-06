from pathlib import Path
import argparse,hashlib,json,sys
from checks import SOURCE_HEAD,git_read,tracked_state
parser=argparse.ArgumentParser();parser.add_argument('--checkpoint',choices=['before','after']);args=parser.parse_args()
root=Path.cwd().resolve();out=root/'target/wp08-dpr-ci2';out.mkdir(parents=True,exist_ok=True);dest=out/'fixture-inputs';dest.mkdir(exist_ok=True)
if args.checkpoint:
    state=None;failure=None
    try:
        state=tracked_state(root);(out/('source-job-'+args.checkpoint+'.json')).write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8')
    except BaseException as error:failure=repr(error)
    previous=json.loads((out/'source-job-before.json').read_text(encoding='utf-8')) if (out/'source-job-before.json').is_file() else None
    valid=state is not None and state['head']==SOURCE_HEAD and not state['status'] and (args.checkpoint=='before' or previous==state)
    (out/('source-job-'+args.checkpoint+'-result.json')).write_text(json.dumps({'state_valid':valid,'observation_error':failure,'before_after_equal':previous==state if args.checkpoint=='after' and previous else None},indent=2)+'\n',encoding='utf-8')
    sys.exit(0 if valid else 1)
before=tracked_state(root);assert before['head']==SOURCE_HEAD and not before['status'];records=[]
for language in ['ko','en']:
    path=root/'target/wp12'/('ledger-'+language+'.html');assert path.is_file(),('Original npm test fixture missing',str(path));data=path.read_bytes();assert data
    (dest/path.name).write_bytes(data);records.append({'path':str(path),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'copied_to':str(dest/path.name)})
after=tracked_state(root);assert before==after
(out/'fixture-inputs.json').write_text(json.dumps({'source_head':SOURCE_HEAD,'producer':'original npm --prefix client test; client/src/i18n.test.tsx','files':records,'source_state_equal_during_recording':True},indent=2)+'\n',encoding='utf-8')
print(json.dumps(records,indent=2))
