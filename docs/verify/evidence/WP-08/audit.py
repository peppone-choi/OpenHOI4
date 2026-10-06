import pathlib,json,hashlib,subprocess,sys,re,urllib.request
from run import ROOT,OUT,run,CARGO,ENV
def sha(b):return hashlib.sha256(b).hexdigest()
results={}
for suffix in ['', '/p06-redirect']:
 folder=ROOT/('docs/worklog/evidence/WP-08'+suffix);manifest=json.loads((folder/'SHA256SUMS.json').read_text(encoding='utf-8'))
 raw=[];crlf=[];final_crlf=[];mismatch=[]
 for name,expected in manifest.items():
  b=(folder/name).read_bytes()
  if sha(b)==expected:raw.append(name)
  elif sha(b.replace(b'\r\n',b'\n').replace(b'\n',b'\r\n'))==expected:crlf.append(name)
  elif b.endswith(b'\n') and sha(b[:-1]+b'\r\n')==expected:final_crlf.append(name)
  else:mismatch.append(name)
 results[suffix or 'original']={'count':len(manifest),'rawMatch':len(raw),'LFtoCRLFMatch':len(crlf),'onlyFinalLFtoCRLFMatch':final_crlf,'unexplained':mismatch}
 assert not mismatch,(suffix,mismatch)
first=ROOT/'docs/worklog/evidence/WP-08/first-map'
results['first-map']={p.name:p.read_bytes()==subprocess.check_output(['git','show','445b935:docs/worklog/evidence/WP-08/first-map/'+p.name],cwd=ROOT) for p in first.iterdir() if p.is_file()};assert all(results['first-map'].values())
results['baseline']=sha((ROOT/'client/e2e-m1/baselines/display-fixture.png').read_bytes())
violations={}
for crate in ['oh_sim','oh_ai']:
 for p in (ROOT/'crates'/crate/'src').rglob('*.rs'):
  matches=[(i,line) for i,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1) if re.search(r'\bf32\b|\bf64\b|\bHashMap\b|thread_rng|SystemTime|Instant::|Utc::now',line)]
  if matches:violations[str(p.relative_to(ROOT))]=matches
results['forbidden']=violations;assert not violations
results['unchangedGolden']=subprocess.check_output(['git','diff','--name-only','b1da8f4','HEAD','--','tests/golden','tests/repro'],cwd=ROOT).decode();assert not results['unchangedGolden']
results['p06ProductDiff']=subprocess.check_output(['git','diff','--name-only','f58c0c6','HEAD','--','crates','data','client/src'],cwd=ROOT).decode()
base='http://127.0.0.1:19415';html=urllib.request.urlopen(base).read();js=re.search(rb'src="([^"]+\.js)"',html).group(1).decode();serverjs=urllib.request.urlopen(base+js).read()
assert serverjs==(ROOT/'client/dist'/js.lstrip('/')).read_bytes()
results['servedIdentity']={'sourceCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip(),'exeSHA256':sha((ROOT/'target/debug/oh_server.exe').read_bytes()),'js':js,'jsSHA256':sha(serverjs),'servedEqualsBuilt':True,'htmlSHA256':sha(html),'packHash':json.load(urllib.request.urlopen(base+'/maps/testland/metadata'))['pack_hash'],'port':19415}
(OUT/'audit.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(results,ensure_ascii=False,indent=2))
