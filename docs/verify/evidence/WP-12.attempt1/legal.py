from pathlib import Path
import urllib.request,hashlib,subprocess,json
r=Path('E:/openhoi/.orchestrator/wt/WP-12-verify');result=[]
for p,url in [('client/licenses/GPL-3.0.txt','https://raw.githubusercontent.com/spdx/license-list-data/main/text/GPL-3.0-or-later.txt'),('client/licenses/CC-BY-SA-4.0.txt','https://creativecommons.org/licenses/by-sa/4.0/legalcode.txt')]:
 b=urllib.request.urlopen(url,timeout=30).read();i=subprocess.check_output(['git','-C',str(r),'show',':'+p]);w=(r/p).read_bytes()
 v={'path':p,'url':url,'upstreamSHA256':hashlib.sha256(b).hexdigest(),'indexSHA256':hashlib.sha256(i).hexdigest(),'workingSHA256':hashlib.sha256(w).hexdigest(),'indexExact':i==b,'workingExact':w==b,'indexLFEqual':i==b.replace(b'\r\n',b'\n')};result.append(v)
print(json.dumps(result,indent=2));(r/'target/wp12-verify/license-originals.json').write_text(json.dumps(result,indent=2))
