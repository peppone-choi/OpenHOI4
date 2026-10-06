from reference import *
import tomllib,zipfile,urllib.request,re
def packidentity(p):
 files=sorted((x.relative_to(p).as_posix(),x.read_bytes()) for x in p.rglob('*') if x.is_file())
 return int(fnv(vec(files,lambda x:s(x[0])+u(len(x[1]))+x[1])),16)
def defines(p):
 merged={}
 for path in [p/'defines.toml',p/'scenarios/m1/defines.toml']:
  for system,items in tomllib.loads(path.read_text()).items():merged.setdefault(system,{}).update(items)
 def number(x):assert isinstance(x,int);return b'\0'+i(x)
 def value(x):return b'\1'+vec(x,number) if isinstance(x,list) else b'\0'+number(x)
 return int(fnv(vec(sorted(merged.items()),lambda x:s(x[0])+vec(sorted(x[1].items()),lambda kv:s(kv[0])+value(kv[1])))),16)
folders=[OUT/'own/pack',OUT/'ui/packs/testland',OUT/'native/run-1/pack']+[p/'run-1/pack' for p in (OUT/'three-os').iterdir()]
records=[]
for p in folders:
 identity=packidentity(p);dh=defines(p)
 if p==OUT/'own/pack':report=json.load(open(OUT/'own/capture.json'));assert report['pack']['content_hash']==identity and report['header']['effective_defines_hash']==dh
 elif p==OUT/'ui/packs/testland':assert json.load(open(OUT/'ui/wire.json'))['welcome']['packs'][0]['hash']==f'{identity:016x}'
 else:
  folder=p.parent.parent;report=json.load(open(folder/'capture-1.stdout'));assert report['pack']['content_hash']==identity
  b=(p.parent/'saved.ohsave').read_bytes();end=10+struct.unpack('<I',b[6:10])[0];assert b[10:end].endswith(u(dh))
 records.append(dict(root=str(p),pack_hash=f'{identity:016x}',effective_defines_hash=f'{dh:016x}'))
for folder in (OUT/'three-os').iterdir():
 r=json.load(open(folder/'server/result.json'));assert r['head']==BASE['head'] and r['server_exit']==0 and r['query_exit']==0 and r['server_pid']>0;assert r['pack_hash']==f'{packidentity(folder/"server/packs/testland"):016x}'
 for n in [1,2]:
  for stage in ['capture','restart','cli-resume']:
   cmd=json.load(open(folder/f'{stage}-{n}.command.json'));assert cmd['exit']==0 and len(cmd['command'])>2
  assert (folder/f'cli-resume-{n}.stdout').read_text().strip()=='d19d028857bb7485'
run=next(x for x in json.load(open(OUT/'ci-runs-api.json',encoding='utf-8'))['workflow_runs'] if x['name']=='Save determinism')
logs=subprocess.check_output(['gh','api',f'repos/peppone-choi/OpenHOI4/actions/runs/{run["id"]}/logs'],cwd=ROOT,env=os.environ);(OUT/'save-original-ci-logs.zip').write_bytes(logs)
with zipfile.ZipFile(__import__('io').BytesIO(logs)) as z:
 names=z.namelist();assert any('Compare' in n or 'compare' in n for n in names)
 (OUT/'save-original-ci-log-members.json').write_text(json.dumps([dict(name=n,sha256=hashlib.sha256(z.read(n)).hexdigest()) for n in names]))
url='https://raw.githubusercontent.com/facebook/zstd/v1.5.7/LICENSE';license=urllib.request.urlopen(url,timeout=30).read();(OUT/'zstd-official-license.txt').write_bytes(license);assert b'BSD License' in license
shipped=(ROOT/'docs/licenses/WP-11-zstd-1.5.7-BSD-3-Clause.txt').read_text().strip();assert shipped in license.decode()
old=git('show','227c1a2e606ccbb0cbc9712b0dd7a6e15ffadb30:crates/oh_save/src/file.rs');new=(ROOT/'crates/oh_save/src/file.rs').read_text();tests=lambda x:set(re.findall(r'#\[test\]\s+fn\s+(\w+)',x));assert tests(old)<=tests(new)
unchanged=git('diff','--name-only','227c1a2e606ccbb0cbc9712b0dd7a6e15ffadb30',BASE['head'],'--','client','crates/oh_sim','crates/oh_cli','crates/oh_save/tests','crates/oh_save/examples');assert not unchanged
result=dict(identities=records,original_file_tests_preserved=sorted(tests(old)),new_file_tests=sorted(tests(new)-tests(old)),source_227_to_750_unchanged_tests_client_sim_cli=True,official_zstd_license=url)
(OUT/'identity-audit.json').write_text(json.dumps(result,indent=2));print(json.dumps(result))
