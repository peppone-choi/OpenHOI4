import run_checks as m,zipfile,hashlib,json,pathlib
archive=m.OUT/'prior-attempt/red62-source.zip'; identity=json.loads((m.OUT/'prior-attempt/red62-source-identity.json').read_text())
assert hashlib.sha256(archive.read_bytes()).hexdigest()==identity['sha256']
old=m.OUT/'red62-source';old.mkdir(exist_ok=True)
with zipfile.ZipFile(archive) as z:
 for n in z.namelist(): assert pathlib.PurePosixPath(n).is_absolute()==False and '..' not in pathlib.PurePosixPath(n).parts
 z.extractall(old)
(m.OUT/'red62-extracted-sha.json').write_text(json.dumps({str(p.relative_to(old)):hashlib.sha256(p.read_bytes()).hexdigest() for p in old.rglob('*') if p.is_file()}))
for name,args in [('red62-npm-ci',['npm.cmd','--prefix','client','ci']),('red62-client-build',['npm.cmd','--prefix','client','run','build']),('red62-server-build',['cargo','build','-p','oh_server','--locked'])]:m.run(name,args,old)
