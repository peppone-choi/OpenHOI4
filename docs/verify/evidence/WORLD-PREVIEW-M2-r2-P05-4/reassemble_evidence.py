import pathlib,json,hashlib,argparse
p=argparse.ArgumentParser();p.add_argument('output');a=p.parse_args();base=pathlib.Path(__file__).resolve().parent;j=json.loads((base/'evidence-storage.json').read_text(encoding='utf-8'));out=pathlib.Path(a.output).resolve();assert not out.exists(),'Do not replace an existing original';out.parent.mkdir(parents=True,exist_ok=True);whole=hashlib.sha256();count=0
with out.open('xb') as f:
 for part in j['parts']:
  b=(base/part['path']).read_bytes();assert len(b)==part['bytes'] and hashlib.sha256(b).hexdigest()==part['sha256'];f.write(b);whole.update(b);count+=len(b)
assert count==j['original_bytes'] and whole.hexdigest()==j['original_sha256'];print(whole.hexdigest())
