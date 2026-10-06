from pathlib import Path
from PIL import Image
import json
p=Path('target/wp08-verify4/resize-results.json');j=json.loads(p.read_text(encoding='utf-8'))
for r in j['results']:
 im=Image.open(r['file']).convert('RGB');r['screenshotRGB']=list(im.getpixel((int(r['p']['x']),int(r['p']['y']))));r['rgbPass']=all(abs(a-b)<=1 for a,b in zip(r['screenshotRGB'],[40,100,180]));print(r['label'],r['screenshotRGB'],r['rgbPass'])
p.with_name('resize-oracle.json').write_text(json.dumps(j,indent=2),encoding='utf-8')
raise SystemExit(0 if all(r['rgbPass'] for r in j['results']) else 1)
