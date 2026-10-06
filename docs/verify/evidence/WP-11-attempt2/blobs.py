import runpy,pathlib,json,struct,copy,hashlib,subprocess
v=runpy.run_path('target/wp11-verify2/reference.py');globals().update({k:v[k] for k in ['R','O','P','PACK','C','u','i','s','opt','vec','date','dto','fnv','header','invoke','h','raw','base','pack_hash','verify']})
B=C/'blobs';B.mkdir(exist_ok=True);rows=[]
def compress(data,name):
 f=B/(name+'.raw');out=B/(name+'.zst');f.write_bytes(data);invoke(['compress',f,out],name+'-compress');return out.read_bytes()
def container(hh,body=None,compressed=None):
 hb=header(hh);cb=raw[10+struct.unpack('<I',raw[6:10])[0]:] if compressed is None and body is None else (compressed if compressed is not None else compress(body,'temp'+str(len(rows))))
 return b'OHSV\1\0'+struct.pack('<I',len(hb))+hb+cb
original=raw
for force in [False,True]:
 for field,val in [('engine_version','0.2.0'),('engine_version','bad'),('format_version',0),('scenario_id','../m1'),('tick',24),('seed',123),('state_hash',0),('player_nations',[2,1]),('player_nations',[1,1]),('player_nations',[0]),('saved_at_utc',-1),('saved_at_utc',253402300800),('definitions_hash',0),('effective_defines_hash',0)]:
  hh=copy.deepcopy(h);hh[field]=val
  name='header-'+field+str(len(rows));f=B/(name+'.ohsave');f.write_bytes(container(hh));res=json.loads(invoke(['blob',PACK,f,'force' if force else 'normal'],name));assert not res['ok'],(name,res);rows.append({'name':name,'force':force,'expected':False,'error':res['error']})
 for field in ['version','content_hash']:
  hh=copy.deepcopy(h);hh['packs'][0][field]='9.0.0' if field=='version' else 0;name='pack-'+field+str(force);f=B/(name+'.ohsave');f.write_bytes(container(hh));res=json.loads(invoke(['blob',PACK,f,'force' if force else 'normal'],name));assert res['ok']==force,(name,res)
  if force:assert len(res['warnings'])==1;verify(res['report']);assert res['report']['hash']==f"{h['state_hash']:016x}"
  rows.append({'name':name,'force':force,'expected':force,'error':res.get('error'),'warnings':res.get('warnings')})
def check(name,b,ok=False,lim=None):
 f=B/(name+'.ohsave');f.write_bytes(b)
 for force in [False,True]:
  args=['blob',PACK,f,'force' if force else 'normal']
  if lim is not None:lp=B/(name+'.limits.json');lp.write_text(json.dumps(lim));args.append(lp)
  res=json.loads(invoke(args,name+str(force)));assert res['ok']==ok,(name,force,res);rows.append({'name':name,'force':force,'expected':ok,'error':res.get('error')})
check('valid-normal',raw,True)
for ts in [1,253402300799]:
 hh=copy.deepcopy(h);hh['saved_at_utc']=ts;check('metadata-time'+str(ts),container(hh),True)
for ver in [0,2,65535]:b=bytearray(raw);b[4:6]=struct.pack('<H',ver);check('format'+str(ver),b)
for leng in [0,65537,2**32-1]:b=bytearray(raw);b[6:10]=struct.pack('<I',leng);check('length'+str(leng),b)
b=bytearray(raw);b[0]^=32;check('magic',b)
for n in range(len(raw)):check('truncated-'+str(n),raw[:n])
end=10+struct.unpack('<I',raw[6:10])[0];comp=raw[end:]
for extra in [b'\0',b'x',comp,bytes.fromhex('502a4d1800000000')]:check('trailing'+str(len(rows)),raw+extra)
for at in [0,1,2,3,20,len(comp)//2,*range(len(comp)-4,len(comp))]:
 b=bytearray(comp);b[at]^=1;check('zstd-corruption'+str(at),container(h,compressed=b))
check('postcard-body-trailing',container(h,body=dto(base)+b'\0'))
hb=header(h);check('postcard-header-trailing',b'OHSV\1\0'+struct.pack('<I',len(hb)+1)+hb+b'\0'+comp)
# Nonminimal varint engine string length, otherwise exactly valid bytes.
check('overlong-varint',b'OHSV\1\0'+struct.pack('<I',len(hb)+1)+bytes([hb[0]|128,0])+hb[1:]+comp)
check('body-expands-over-limit',raw,lim={'body_max_bytes':len(dto(base))-1})
check('body-exact-limit',raw,True,lim={'body_max_bytes':len(dto(base))})
check('file-over-limit',raw,lim={'file_max_bytes':len(raw)-1});check('header-over-limit',raw,lim={'header_max_bytes':len(hb)-1});check('budget-1',raw,lim={'allocation_budget_bytes':1})
# Semantic forged bytes with hash recomputed by the independent encoder still cannot bypass references or ledger.
for field,change in [('owner',lambda d:d['world']['inputs']['states'][0].update(owner=65000)),('ledger',lambda d:d['world']['inputs']['states'][0]['ledger'].update(value=1)),('queue',lambda d:d['queue'][0].update(command={'SetSpeed':0}))]:
 d=copy.deepcopy(base);change(d);hh=copy.deepcopy(h);hh['state_hash']=int(fnv(dto(d,True)),16);check('forged-'+field,container(hh,body=dto(d)))
assert (O/'own-native/saved.ohsave').read_bytes()==original
(B/'results.json').write_text(json.dumps(rows,indent=2));print(json.dumps({'blob_trials':len(rows),'stored_file_unchanged_sha':hashlib.sha256(original).hexdigest()}))
# Independently compare exact bytes/header/hash/full structures for every OS artifact, without product Python reference.
ci=[]
for folder in sorted((O/'ci/three').iterdir()):
 result=json.loads((folder/'result.json').read_text());assert result['head']=='750c2732b00c164ddc252fa26c4984ababd64f35';assert not result['dirty'];local=(R/'crates/oh_save/tests/fixtures/m1-v1.ohsave').read_bytes()
 for n in [1,2]:
  cp=json.loads((folder/f'capture-{n}.stdout').read_text());rp=json.loads((folder/f'restart-{n}.stdout').read_text());assert cp['pid']!=rp['pid'];assert cp['split']==rp['initial'];assert cp['continuous']==rp['resumed'];assert (folder/f'run-{n}/saved.ohsave').read_bytes()==local
  for stage,key in [('split','split_hash'),('continuous','continuous_hash')]:verify({'dto':cp[stage],'hash':cp[key],'canonical':cp['canonical_hex'] if stage=='split' else dto(cp[stage],True).hex()})
  assert int(fnv(dto(cp['split'],True)),16)==int(rp['initial_hash'],16);assert fnv(dto(rp['resumed'],True))==rp['resumed_hash'];assert pack_hash(folder/f'run-{n}/pack')==cp['pack']['content_hash'];assert (folder/f'cli-resume-{n}.stdout').read_text().strip()==rp['resumed_hash']
  for stage in ['capture','restart','cli-resume']:
   cmd=json.loads((folder/f'{stage}-{n}.command.json').read_text());assert cmd['exit']==0;assert (folder/f'{stage}-{n}.stderr').is_file()
  dat=(folder/f'run-{n}/saved.ohsave').read_bytes();ln=struct.unpack('<I',dat[6:10])[0];(B/'ci-compressed').write_bytes(dat[10+ln:]);invoke(['uncompress',B/'ci-compressed',B/'ci-body'],folder.name+str(n)+'-uncompress');assert (B/'ci-body').read_bytes()==dto(cp['split'])
  assert cp['split']['state']['tick']==48 and cp['continuous']['state']['tick']==96;assert cp['split']['world']['inputs']['states'][0]['infrastructure']==12884901890;assert cp['continuous']['world']['inputs']['states'][0]['infrastructure']==12884901888
  ci.append({'os':folder.name,'capture_pid':cp['pid'],'restart_pid':rp['pid'],'file_sha':hashlib.sha256(dat).hexdigest(),'split':cp['split_hash'],'resume':rp['resumed_hash']})
assert len(ci)==6;(O/'ci/independent-os-compare.json').write_text(json.dumps(ci,indent=2));print(json.dumps({'independent_3os_runs':ci}))
