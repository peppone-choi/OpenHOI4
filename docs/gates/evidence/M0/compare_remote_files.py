import json,pathlib,hashlib,subprocess
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';results=[]
for remote,stored in [('remote-core','main-core'),('remote-sim','main-sim'),('remote-client/wp05-protocol-browser-evidence','main-client')]:
 rp=ev/remote;sp=root/'docs/verify/evidence/WP-05'/stored
 for p in sp.rglob('*'):
  if not p.is_file():continue
  rel=p.relative_to(sp); actual=rp/rel; assert actual.is_file(),str(actual)
  raw=subprocess.check_output(['git','show','HEAD:'+p.relative_to(root).as_posix()]); received=actual.read_bytes(); checkout=p.read_bytes()
  ok=raw==received
  normalized=raw.replace(b'\r\n',b'\n')==received.replace(b'\r\n',b'\n')
  assert ok or normalized,str(rel)
  results.append({'path':str(p.relative_to(root)),'git_blob_exact':ok,'newline_normalized_equal':normalized,'checkout_exact':checkout==received,'remote_sha256':hashlib.sha256(received).hexdigest()})
for family in ['remote-core','remote-sim']:
 for p in (ev/family).rglob('*.txt'):print(p.relative_to(ev),p.read_text().strip())
(ev/'remote-file-comparison.json').write_text(json.dumps(results,indent=2),encoding='utf8')
print('comparison PASS',len(results),'exact git blobs',sum(x['git_blob_exact'] for x in results),'newline-only',sum(not x['git_blob_exact'] for x in results))
