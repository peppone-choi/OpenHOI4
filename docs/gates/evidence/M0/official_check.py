import json,pathlib,urllib.request,concurrent.futures
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';records=json.loads((root/'docs/verify/evidence/WP-06/official-metadata-summary.json').read_text(encoding='utf8'));results=[]
def check(r):
 req=urllib.request.Request(r['url'],headers={'User-Agent':'OpenHOI-M0-independent-gate-verifier'})
 with urllib.request.urlopen(req,timeout=25) as f:data=json.load(f)
 name=r['name'];version=r['recorded_version'];actual=next(v for v in data['versions'] if v['num']==version) if r['ecosystem']=='cargo' else data
 assert (actual['num'] if r['ecosystem']=='cargo' else actual['version'])==version
 assert actual['license']==r['recorded_license'],(name,actual['license'],r['recorded_license'])
 return {'name':name,'url':r['url'],'version':version,'license':actual['license'],'matches':True}
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
 for r in pool.map(check,records):results.append(r);print(r['name'],'official exact version/license PASS',flush=True)
(ev/'official-version-license.json').write_text(json.dumps(results,indent=2),encoding='utf8')
