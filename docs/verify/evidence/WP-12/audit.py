import pathlib,subprocess,json,hashlib,re,urllib.request
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');o=r/'target/wp12-verify2'
sums=json.loads((r/'docs/worklog/evidence/WP-12/SHA256SUMS.json').read_text())
errors=[]
for p,h in sums['files'].items():
 b=subprocess.check_output(['git','-C',str(r),'show',':docs/worklog/evidence/WP-12/'+p]);a=hashlib.sha256(b).hexdigest()
 if a!=h:errors.append(p)
print('Evidence SHA index verified',len(sums['files']),'errors',errors)
old='1bfea48511c6b75efb0dd26ee6e980a90d77da04'
paths=subprocess.check_output(['git','-C',str(r),'diff','--name-only',old]).decode().splitlines()
for p in ['client/src/App.test.tsx','client/src/network.test.ts','client/e2e/network.spec.ts','client/src/proto/protocol.ts']:
 print('Preserved',p,p not in paths)
print('Simulation/save/proto/golden modifications',[p for p in paths if p.startswith(('crates/oh_sim','crates/oh_core','crates/oh_save','crates/oh_proto','tests/golden'))])
for name,url in [('font','https://raw.githubusercontent.com/google/fonts/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf'),('ofl','https://raw.githubusercontent.com/google/fonts/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/OFL.txt'),('fluent','https://registry.npmjs.org/@fluent%2fbundle/0.19.1')]:
 try:
  b=urllib.request.urlopen(url,timeout=30).read();(o/(name+'-upstream.bin')).write_bytes(b)
  if name=='fluent':print(name,json.loads(b)['version'],json.loads(b)['license'])
  else:
   p=r/('client/public/fonts/'+('NotoSansKR.ttf' if name=='font' else 'OFL.txt'));idx=subprocess.check_output(['git','-C',str(r),'show',':'+p.relative_to(r).as_posix()])
   print(name,'bytes',len(b),'SHA',hashlib.sha256(b).hexdigest(),'index-original-equal',idx==b,'working-original-equal',p.read_bytes()==b)
 except Exception as e:print('FETCH ERROR',name,repr(e))
