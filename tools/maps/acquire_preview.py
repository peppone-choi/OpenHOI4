"""Explicit online acquisition only; offline bake never imports this module."""
import argparse
from pathlib import Path
import json
import hashlib
import urllib.request

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--manifest',type=Path,default=Path('data/preview/world/source-manifest.json'))
args=parser.parse_args();root=args.manifest.parent
for source in json.loads(args.manifest.read_text())['sources']:
    path=root/source['file']
    if not path.resolve().is_relative_to(root.resolve()):
        raise ValueError('source path escape')
    if path.exists():
        data=path.read_bytes()
    else:
        data=urllib.request.urlopen(source['url'],timeout=60).read()
    if hashlib.sha256(data).hexdigest()!=source['sha256']:
        raise ValueError('source hash mismatch: '+source['file'])
    if not path.exists():
        path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
    print(source['file'],len(data),source['sha256'])
