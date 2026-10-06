from run import *
import re
sources=list((ROOT/'target/debug/build').glob('oh_server-*/out/assets.rs'));assert sources;records=[]
for p in sources:
 text=p.read_text();paths=[json.loads(x) for x in re.findall(r'include_bytes!\(("(?:[^"\\]|\\.)*")\)',text)];assert paths
 assert all('WP-11-verify3' in x and 'client' in x and pathlib.Path(x).is_file() for x in paths)
 records.append(dict(source=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),count=len(paths),all_verify3=True,assets=[dict(path=x,sha256=hashlib.sha256(pathlib.Path(x).read_bytes()).hexdigest()) for x in paths]))
id=json.load(open(OUT/'ui/identity.json'));wire=json.load(open(OUT/'ui/wire.json'));id['pack_hash']=wire['welcome']['packs'][0]['hash'];(OUT/'ui/identity-with-pack.json').write_text(json.dumps(id,indent=2));(OUT/'runtime-assets.json').write_text(json.dumps(records,indent=2));print(json.dumps([(r['source'],r['count'],r['all_verify3']) for r in records]))
