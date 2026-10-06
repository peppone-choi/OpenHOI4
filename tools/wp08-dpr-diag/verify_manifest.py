from pathlib import Path
import argparse,hashlib,json
parser=argparse.ArgumentParser();parser.add_argument('--workflow',required=True);args=parser.parse_args();kit=Path(__file__).resolve().parent
manifest=json.loads((kit/'MANIFEST.json').read_text(encoding='utf-8'))
for entry in manifest['files']:
    path=(kit/entry['path']).resolve();assert path.is_relative_to(kit)
    data=path.read_bytes();assert len(data)==entry['bytes'] and hashlib.sha256(data).hexdigest()==entry['sha256'],entry['path']
workflow=Path(args.workflow).read_bytes();assert hashlib.sha256(workflow).hexdigest()==manifest['workflow']['sha256'] and len(workflow)==manifest['workflow']['bytes']
assert manifest['source_head']=='b73f0ce344138d5fb3909eca5d43c786aac49b15'
assert next(x['sha256'] for x in manifest['files'] if x['path']=='candidate-template.ts')=='c9002e018ad536cc48308973a5d3400d69a8925c9732e936bdef30144ae429ac'
print('All',len(manifest['files']),'toolkit files and exact workflow bytes match MANIFEST.json; strict SHA/source HEAD pinned')
