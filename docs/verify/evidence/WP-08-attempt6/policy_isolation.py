import pathlib,subprocess,runpy,sys,json,hashlib
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6';original=subprocess.run;changes=[]
def run(args,*a,**kw):
 if '--manifest-path' in args:
  p=pathlib.Path(args[args.index('--manifest-path')+1]);s=p.read_text(encoding='utf8')
  if p.is_relative_to(OUT/'temp') and 'name="fixture-root"' in s:
   amended=s+'\n[workspace]\n';p.write_text(amended,encoding='utf8');changes.append({'path':str(p),'beforeSHA256':hashlib.sha256(s.encode()).hexdigest(),'afterSHA256':hashlib.sha256(amended.encode()).hexdigest(),'reason':'standalone generated license fixture inside workspace needs empty workspace boundary; dependencies and all original assertions unchanged'})
   print('Generated fixture Cargo workspace isolation:',p,flush=True)
 return original(args,*a,**kw)
subprocess.run=run;sys.argv=['test_wp01.py','-v']
try:runpy.run_path(str(ROOT/'tools/tests/test_wp01.py'),run_name='__main__')
finally:(OUT/'policy-isolation.json').write_text(json.dumps(changes,indent=2),encoding='utf8')
