from run import *
import datetime,struct
PROBE=OUT/'harness-build/debug/wp11_fresh_probe.exe'
def u(n):
 assert 0<=n<1<<64
 a=bytearray()
 while n>127:a.append((n&127)|128);n>>=7
 a.append(n);return bytes(a)
def i(n):return u((n<<1)^(n>>63))
def s(x):b=x.encode('utf-8');return u(len(b))+b
def vec(xs,f):return u(len(xs))+b''.join(map(f,xs))
def opt(x,f):return b'\0' if x is None else b'\1'+f(x)
def date(x):return u(x['year'])+bytes([x['month'],x['day']])
def command(x):return b'\0'+bytes([x['Pause']]) if 'Pause' in x else b'\1'+bytes([x['SetSpeed']])
def modifier(x):return s(x['source'])+s(x['target_stat'])+u(['Add','Mul'].index(x['op']))+i(x['value'])+opt(x['expires'],u)
def entry(x):return opt(x['source'],s)+u(['Base','Add','Mul'].index(x['op']))+i(x['value'])+i(x['accumulated'])
def ledger(x):return s(x['target_stat'])+u(x['tick'])+i(x['value'])+vec(x['entries'],entry)
def nation(x):return u(x['id'])+s(x['tag'])+s(x['government'])+vec(x['support'],lambda p:s(p[0])+i(p[1]))
def stateworld(x):return u(x['id'])+u(x['owner'])+i(x['population'])+vec(x['resources'],lambda p:s(p[0])+i(p[1]))+vec(x['buildings'],lambda p:s(p[0])+i(p[1]))+i(x['base'])+vec(x['modifiers'],modifier)+i(x['infrastructure'])+ledger(x['ledger'])
def province(x):return u(x['id'])+opt(x['state'],u)+opt(x['owner'],u)+opt(x['controller'],u)
def world(x):return u(x['definitions_hash'])+vec(x['inputs']['nations'],nation)+vec(x['inputs']['states'],stateworld)+vec(x['inputs']['provinces'],province)
def dto(x,canonical=False):
 st=x['state'];c=x['config'];b=s(st['scenario'])+u(st['seed'])+u(st['tick'])+date(st['date'])+bytes([st['hour'],st['paused'],st['speed']])+b''.join(u(n) for n in c['speed_ms_per_tick'])+bytes([c['initial_speed']])+vec(x['queue'],lambda q:u(q['tick'])+u(q['nation'])+u(q['sequence'])+command(q['command']))
 return b+(b'' if canonical and x['world'] is None else opt(x['world'],world))
def header(h):return s(h['engine_version'])+u(h['format_version'])+s(h['scenario_id'])+vec(h['packs'],lambda p:s(p['id'])+s(p['version'])+u(p['content_hash']))+date(h['game_date'])+u(h['tick'])+u(h['seed'])+u(h['state_hash'])+vec(h['player_nations'],u)+i(h['saved_at_utc'])+opt(h['definitions_hash'],u)+u(h['effective_defines_hash'])
def fnv(b):
 h=14695981039346656037
 for x in b:h=((h^x)*1099511628211)&((1<<64)-1)
 return f'{h:016x}'
def verifyledgers(x):
 for st in x['world']['inputs']['states']:
  tick=x['state']['tick'];value=st['base'];rows=[dict(source=None,op='Base',value=value,accumulated=value)]
  for m in sorted((m for m in st['modifiers'] if m['expires'] is None or tick<m['expires']),key=lambda m:(['Add','Mul'].index(m['op']),m['source'])):
   value=value+m['value'] if m['op']=='Add' else value*m['value']//(1<<32)
   assert -(1<<63)<=value<(1<<63)
   rows.append(dict(source=m['source'],op=m['op'],value=m['value'],accumulated=value))
  assert st['ledger']==dict(target_stat='infrastructure',tick=tick,value=value,entries=rows)
  assert st['infrastructure']==value
def verifyitem(r):
 b=dto(r['dto'],True);assert b.hex()==r['canonical_hex'];assert fnv(b)==r['hash'];verifyledgers(r['dto'])
def verifyfile(path,h,d):
 b=path.read_bytes();assert b[:4]==b'OHSV';assert struct.unpack('<H',b[4:6])[0]==1
 n=struct.unpack('<I',b[6:10])[0];assert b[10:10+n]==header(h)
 tmp=OUT/'decompressed.bin';subprocess.run([str(PROBE),'decompress',str(path),str(tmp)],cwd=ROOT,env=os.environ,check=True)
 assert tmp.read_bytes()==dto(d);return hashlib.sha256(b).hexdigest()
if __name__=='__main__':
 capture=json.load(open(OUT/'own/capture.json',encoding='utf-8'));resume=json.load(open(OUT/'own/resume.json',encoding='utf-8'));assert capture['pid']!=resume['pid'];assert capture['split']==resume['initial'];assert capture['continuous']==resume['final']
 for name in ['begin','split','paused','expiry','after','continuous']:verifyitem(capture[name])
 verifyitem(resume['initial']);verifyitem(resume['final'])
 split=capture['split']['dto'];assert split['state']['tick']==28 and split['state']['speed']==4 and not split['state']['paused']
 assert split['config']==dict(speed_ms_per_tick=[0,19,13,7,3],initial_speed=2);assert [(p['tick'],p['nation'],p['sequence']) for p in split['queue']]==[(28,0,9),(28,42,2),(28,65535,1),(30,0,4),(60,0,5)]
 assert [n['id'] for n in split['world']['inputs']['nations']]==[0,65535];assert [s['id'] for s in split['world']['inputs']['states']]==[0,901]
 assert [p['id'] for p in split['world']['inputs']['provinces']]==[0,13,50,60,91,65535]
 assert split['world']['inputs']['provinces'][0]['owner']==0;assert split['world']['inputs']['provinces'][1]['controller']==65535
 for p in split['world']['inputs']['provinces']:
  if p['state'] is None:assert p['owner'] is None and p['controller'] is None
 for name,tick,val,rows in [('split',28,19327352833,3),('paused',28,19327352833,3),('expiry',29,12884901889,2),('after',30,12884901889,2),('continuous',61,12884901889,2)]:
  r=capture[name]['dto'];st=r['world']['inputs']['states'][0];assert r['state']['tick']==tick and st['base']==8589934593 and st['infrastructure']==val and len(st['modifiers'])==3 and len(st['ledger']['entries'])==rows
  expected_date=datetime.datetime(2000,1,1)+datetime.timedelta(hours=tick);assert r['state']['date']==dict(year=expected_date.year,month=expected_date.month,day=expected_date.day) and r['state']['hour']==expected_date.hour
 own_sha=verifyfile(OUT/'own/saved.ohsave',capture['header'],split)
 proto=(OUT/'generated/protocol.ts').read_bytes();blob=subprocess.check_output(['git','--no-optional-locks','-C',str(ROOT),'show',BASE['head']+':client/src/proto/protocol.ts'],env=os.environ)
 def checker(b):assert b==blob and b==(ROOT/'client/src/proto/protocol.ts').read_bytes()
 checker(proto);negative=bytearray(proto);negative[len(negative)//2]^=1
 try:checker(bytes(negative));raise RuntimeError('negative checker failed')
 except AssertionError:pass
 # Original three-OS reports, independently encoded byte for byte, display time fixed zero.
 osresults=[]
 for folder in sorted((OUT/'three-os').iterdir()):
  result=json.load(open(folder/'result.json'));assert result['head']==BASE['head'] and not result['dirty']
  for n in [1,2]:
   c=json.load(open(folder/f'capture-{n}.stdout'));r=json.load(open(folder/f'restart-{n}.stdout'));assert c['pid']!=r['pid'];assert r['initial']==c['split'] and r['resumed']==c['continuous']
   for d,hashval in [(c['split'],c['split_hash']),(c['continuous'],c['continuous_hash']),(r['initial'],r['initial_hash']),(r['resumed'],r['resumed_hash'])]:assert fnv(dto(d,True))==hashval;verifyledgers(d)
   assert dto(c['split'],True).hex()==c['canonical_hex'];path=folder/f'run-{n}/saved.ohsave'
   h=dict(engine_version='0.1.0',format_version=1,scenario_id='m1',packs=[c['pack']],game_date=c['split']['state']['date'],tick=c['split']['state']['tick'],seed=c['split']['state']['seed'],state_hash=int(c['split_hash'],16),player_nations=[],saved_at_utc=0,definitions_hash=c['split']['world']['definitions_hash'],effective_defines_hash=None)
   # definitions hash is header's final varint; parse only that last field independently.
   b=path.read_bytes();end=10+struct.unpack('<I',b[6:10])[0];expected_prefix=header({**h,'effective_defines_hash':0})[:-1];assert b[10:end].startswith(expected_prefix)
   at=10+len(expected_prefix);v=0;shift=0
   while True:
    z=b[at];at+=1;v|=(z&127)<<shift;shift+=7
    if z<128:break
   assert at==end;h['effective_defines_hash']=v;sha=verifyfile(path,h,c['split']);assert sha==result['fixture_sha256'];assert path.read_bytes()==(ROOT/'crates/oh_save/tests/fixtures/m1-v1.ohsave').read_bytes()
   assert c['split_hash']==result['split_hash'] and c['continuous_hash']==result['resumed_hash']
  osresults.append(result)
 out=dict(own_fixture_sha256=own_sha,own_hashes={name:capture[name]['hash'] for name in ['split','paused','expiry','after','continuous']},native_pids=[capture['pid'],resume['pid']],protocol_sha256=hashlib.sha256(proto).hexdigest(),protocol_negative_rejected=True,three_os=osresults)
 (OUT/'independent-reference.json').write_text(json.dumps(out,indent=2),encoding='utf-8');print(json.dumps(out))
