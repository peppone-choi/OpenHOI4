"""Bake explicit art samples and exact procedural model identities, offline."""
from pathlib import Path
import json
import hashlib
import tomllib

root=Path(__file__).resolve().parents[2]
meta=json.loads((root/'assets/preview/units3d/metadata.json').read_text())
definitions=root/'data/preview/world/units/defines.toml'
config=tomllib.loads(definitions.read_text())
models=[]
for m in meta['models']:
    data=(root/m['public_path']).read_bytes()
    if hashlib.sha256(data).hexdigest()!=m['sha256']:
        raise ValueError('unit source hash mismatch')
    models.append(dict(id=m['name'],file=m['name']+'.glb',sha256=m['sha256']))
config.update(schema='world-preview-unit-display-v1',models=models,definitions_sha256=hashlib.sha256(definitions.read_bytes()).hexdigest(),purpose='display samples only; no unit IDs or gameplay state')
(root/'client/public/preview/world/units.json').write_bytes((json.dumps(config,sort_keys=True,separators=(',',':'))+'\n').encode())
