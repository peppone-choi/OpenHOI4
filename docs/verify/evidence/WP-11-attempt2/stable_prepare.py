import pathlib,shutil,csv,re,json
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';p=o/'stable-pack';shutil.copytree(o/'own-pack',p,dirs_exist_ok=True);mp={10:0,20:13,30:65535,40:411,50:80,60:65534}
f=p/'maps/testland/provinces.csv';rows=list(csv.DictReader(f.read_text().splitlines()));fields=list(rows[0]);
for row in rows:row['id']=str(mp[int(row['id'])])
with f.open('w',newline='') as out:w=csv.DictWriter(out,fieldnames=fields,lineterminator='\n');w.writeheader();w.writerows(rows)
f=p/'maps/testland/adjacency_overrides.csv';rows=list(csv.DictReader(f.read_text().splitlines()));
for row in rows:
 for k in ['a','b']:row[k]=str(mp[int(row[k])])
with f.open('w',newline='') as out:w=csv.DictWriter(out,fieldnames=['a','b','kind'],lineterminator='\n');w.writeheader();w.writerows(rows)
f=p/'maps/testland/states.toml';t=f.read_text();t=t.replace('id = 1','id = 0').replace('id = 2','id = 901').replace('[10, 20]','[0, 13]').replace('[30, 40]','[65535, 411]').replace('province = 10','province = 0').replace('province = 30','province = 65535');f.write_text(t)
for nation,nid,capital in [('NTH',0,0),('STH',65535,65535)]:
 f=p/f'scenarios/m1/nations/{nation}.toml';t=f.read_text();t=re.sub(r'(?m)^id = \d+',f'id = {nid}',t);t=re.sub(r'(?m)^capital = \d+',f'capital = {capital}',t);f.write_text(t)
f=p/'maps/testland/visuals.toml';f.write_text(f.read_text().replace('1 = [','0 = [').replace('2 = [','901 = ['))
f=p/'scenarios/m1/scenario.toml';f.write_text(f.read_text().replace('1 = "NTH"','0 = "NTH"').replace('2 = "STH"','901 = "STH"').replace('20 = "STH"','13 = "STH"').replace('1=[','0=['))
# Copies for pack-only force, defines mismatch and defs mismatch.
for name in ['content-changed','defines-changed','definitions-changed']:
 dst=o/name;shutil.copytree(o/'own-pack',dst,dirs_exist_ok=True)
 if name=='content-changed':f=dst/'SOURCES.md';f.write_text(f.read_text()+'\n# verifier-only metadata comment\n') if f.exists() else f.write_text('# verifier-only metadata comment\n')
 if name=='defines-changed':f=dst/'defines.toml';f.write_text(f.read_text()+'\n[probe]\nidentity = 1\n')
 if name=='definitions-changed':f=dst/'maps/testland/provinces.csv';f.write_text(f.read_text().replace('land,plains,true','land,hills,true'))
probe=str(o/'probe-build/debug/wp11_verify2_probe.exe');(o/'extended-steps.json').write_text(json.dumps([['stable-initial',[probe,'initial',str(p)]],['public-filecases',[probe,'filecases',str(o/'own-pack'),str(o/'public-file')]]]))
