import pathlib,json,subprocess,hashlib,struct,copy,datetime
R=pathlib.Path.cwd();O=R/'target/wp11-verify2';P=O/'probe-build/debug/wp11_verify2_probe.exe';PACK=O/'own-pack';C=O/'cases';C.mkdir(exist_ok=True);RESULT=[]
def invoke(args,name):
 p=subprocess.run([str(P),*map(str,args)],cwd=R,capture_output=True);(C/(name+'.stdout')).write_bytes(p.stdout);(C/(name+'.stderr')).write_bytes(p.stderr);assert p.returncode==0,(name,p.stderr.decode());return p.stdout
# Independently encode the positional v1 contract with integer postcard forms.
def u(n):
 assert 0<=n<2**64
 b=[]
 while True:
  b.append((n&127)|(128 if n>127 else 0));n>>=7
  if not n:return bytes(b)
def i(n):return u(n*2 if n>=0 else -n*2-1)
def s(t):b=t.encode();return u(len(b))+b
def opt(x,f=u):return bytes([x is not None])+(f(x) if x is not None else b'')
def vec(xs,f):return u(len(xs))+b''.join(map(f,xs))
def pairs(xs):return vec(xs,lambda p:s(p[0])+i(p[1]))
def date(d):return u(d['year'])+bytes([d['month'],d['day']])
def mod(m):return s(m['source'])+s(m['target_stat'])+u(['Add','Mul'].index(m['op']))+i(m['value'])+opt(m['expires'])
def row(e):return opt(e['source'],s)+u(['Base','Add','Mul'].index(e['op']))+i(e['value'])+i(e['accumulated'])
def ledger(l):return s(l['target_stat'])+u(l['tick'])+i(l['value'])+vec(l['entries'],row)
def nation(n):return u(n['id'])+s(n['tag'])+s(n['government'])+pairs(n['support'])
def state(t):return u(t['id'])+u(t['owner'])+i(t['population'])+pairs(t['resources'])+pairs(t['buildings'])+i(t['base'])+vec(t['modifiers'],mod)+i(t['infrastructure'])+ledger(t['ledger'])
def prov(p):return u(p['id'])+opt(p['state'])+opt(p['owner'])+opt(p['controller'])
def queue(q):k,v=next(iter(q['command'].items()));return u(q['tick'])+u(q['nation'])+u(q['sequence'])+u(['Pause','SetSpeed'].index(k))+bytes([v])
def dto(d,canonical=False):
 st=d['state'];co=d['config'];w=d['world'];b=s(st['scenario'])+u(st['seed'])+u(st['tick'])+date(st['date'])+bytes([st['hour'],st['paused'],st['speed']])+b''.join(u(n) for n in co['speed_ms_per_tick'])+bytes([co['initial_speed']])+vec(d['queue'],queue)
 if w is None:return b+(b'' if canonical else b'\0')
 x=w['inputs'];return b+b'\1'+u(w['definitions_hash'])+vec(x['nations'],nation)+vec(x['states'],state)+vec(x['provinces'],prov)
def fnv(b):
 v=14695981039346656037
 for c in b:v=((v^c)*1099511628211)%2**64
 return f'{v:016x}'
def recompute(d):
 for t in d['world']['inputs']['states']:
  n=t['base'];es=[{'source':None,'op':'Base','value':n,'accumulated':n}]
  for m in sorted([m for m in t['modifiers'] if m['expires'] is None or d['state']['tick']<m['expires']],key=lambda m:(m['op']!='Add',m['source'])):
   n=n+m['value'] if m['op']=='Add' else n*m['value']//4294967296
   assert -(2**63)<=n<2**63
   es.append({'source':m['source'],'op':m['op'],'value':m['value'],'accumulated':n})
  t['infrastructure']=n;t['ledger']={'target_stat':'infrastructure','tick':d['state']['tick'],'value':n,'entries':es}
 return d
def verify(rep):
 d=rep['dto'];expected=copy.deepcopy(d);recompute(expected);assert expected==d
 assert dto(d,True).hex()==rep['canonical'];assert fnv(dto(d,True))==rep['hash']
 for t in d['world']['inputs']['states']:assert t['ledger']['tick']==d['state']['tick']
a=json.loads((O/'own-capture.log').read_text());b=json.loads((O/'own-resume.log').read_text());end=json.loads((O/'own-native/continuous.json').read_text())
for rep in [*a['points'],a['paused'],b['loaded'],b['resumed'],end]:verify(rep)
assert a['split']==b['loaded']|{'pid':a['split']['pid']};assert a['split']['pid']!=b['loaded']['pid'];assert b['resumed']['dto']==end['dto'];assert b['resumed']['hash']==end['hash']==(O/'own-cli-resume.log').read_text().strip()
assert a['split']['dto']['state']['tick']==23;assert end['dto']['state']['tick']==60
assert [x['dto']['world']['inputs']['states'][0]['infrastructure'] for x in a['points']]==[6442450945,6442450944,6442450944]
assert a['paused']['dto']['state']['tick']==23 and a['paused']['dto']['state']['paused'];assert len(a['paused']['dto']['queue'])==2;assert a['points'][1]['dto']['state']['speed']==5 and a['points'][2]['dto']['state']['speed']==5 and a['points'][2]['dto']['queue'][0]['tick']==25 and end['dto']['state']['speed']==4
# Independent byte equality to the tracked Git blob AND original worktree; negative checker is identical.
blob=subprocess.check_output(['git','--no-optional-locks','-C',str(R),'show','HEAD:client/src/proto/protocol.ts']);original=(R/'client/src/proto/protocol.ts').read_bytes();generated=(O/'generated/protocol.ts').read_bytes()
def same(x):return x==blob==original
assert same(generated);mut=bytearray(generated);mut[len(mut)//2]^=1;(O/'generated/changed-protocol.ts').write_bytes(mut);assert not same(mut)
(C/'proto-check.json').write_text(json.dumps({'git':hashlib.sha256(blob).hexdigest(),'worktree':hashlib.sha256(original).hexdigest(),'generated':hashlib.sha256(generated).hexdigest(),'negative_rejected':not same(mut)},indent=2))
# Decode header independently. Field lengths cannot be silently normalized.
raw=(O/'own-native/saved.ohsave').read_bytes();assert raw[:6]==b'OHSV\1\0';hlen=struct.unpack('<I',raw[6:10])[0]
class Reader:
 def __init__(self,b):self.b=b;self.at=0
 def u(self):
  n=shift=0
  while True:
   v=self.b[self.at];self.at+=1;n|=(v&127)<<shift
   if not v&128:return n
   shift+=7
 def text(self):n=self.u();b=self.b[self.at:self.at+n];self.at+=n;return b.decode()
 def byte(self):v=self.b[self.at];self.at+=1;return v
 def date(self):return {'year':self.u(),'month':self.byte(),'day':self.byte()}
 def option(self):return self.u() if self.byte() else None
rr=Reader(raw[10:10+hlen]);h={'engine_version':rr.text(),'format_version':rr.u(),'scenario_id':rr.text()};h['packs']=[{'id':rr.text(),'version':rr.text(),'content_hash':rr.u()} for _ in range(rr.u())];h['game_date']=rr.date()
for k in ['tick','seed','state_hash']:h[k]=rr.u()
h['player_nations']=[rr.u() for _ in range(rr.u())];zz=rr.u();h['saved_at_utc']=-(zz//2)-1 if zz%2 else zz//2;h['definitions_hash']=rr.option();h['effective_defines_hash']=rr.u();assert rr.at==hlen
assert h['engine_version']=='0.1.0' and h['format_version']==1 and h['scenario_id']=='m1';assert h['packs']==[a['pack']];assert h['game_date']==a['split']['dto']['state']['date'];assert h['seed']==987654321 and h['tick']==23;assert f"{h['state_hash']:016x}"==a['split']['hash'];assert h['player_nations']==[1,2] and h['saved_at_utc']==0;assert h['definitions_hash']==a['split']['dto']['world']['definitions_hash'];assert h['effective_defines_hash']==a['defines']
def header(h):return s(h['engine_version'])+u(h['format_version'])+s(h['scenario_id'])+vec(h['packs'],lambda p:s(p['id'])+s(p['version'])+u(p['content_hash']))+date(h['game_date'])+u(h['tick'])+u(h['seed'])+u(h['state_hash'])+vec(h['player_nations'],u)+i(h['saved_at_utc'])+opt(h['definitions_hash'])+u(h['effective_defines_hash'])
assert header(h)==raw[10:10+hlen]
(C/'compressed.zst').write_bytes(raw[10+hlen:]);invoke(['uncompress',C/'compressed.zst',C/'body.raw'],'body-uncompress');assert (C/'body.raw').read_bytes()==dto(a['split']['dto'])
def pack_hash(p):
 entries=sorted((f.relative_to(p).as_posix(),f.read_bytes()) for f in p.rglob('*') if f.is_file());return int(fnv(vec(entries,lambda x:s(x[0])+u(len(x[1]))+x[1])),16)
assert pack_hash(PACK)==h['packs'][0]['content_hash']
(C/'field-evidence.json').write_text(json.dumps({'header':h,'split_hash':a['split']['hash'],'loaded_hash':b['loaded']['hash'],'resumed_hash':end['hash'],'capture_pid':a['split']['pid'],'resume_pid':b['loaded']['pid'],'expiry':[x['hash'] for x in a['points']],'fixture_sha':hashlib.sha256(raw).hexdigest()},indent=2))
def trial(name,d,ok=False,pack=PACK):
 f=C/(name+'.json');f.write_text(json.dumps(d));res=json.loads(invoke(['dto',pack,f],name));RESULT.append({'name':name,'expected':ok,'actual':res['ok'],'error':res.get('error')});assert res['ok']==ok,(name,res)
 if ok:verify(res['report']);assert res['report']['dto']==d
base=a['split']['dto']
def mutate(name,fn,ok=False):d=copy.deepcopy(base);fn(d);trial(name,d,ok)
for speed in range(1,6):mutate(f'speed{speed}',lambda d,v=speed:d['state'].update(speed=v),True)
mutate('nondefault-time',lambda d:d['config'].update(speed_ms_per_tick=[0,1,2,999999,2**64-1],initial_speed=5),True)
for field,v in [('speed',0),('speed',6),('hour',24),('scenario',''),('scenario','../m1'),('tick',22)]:mutate('badstate-'+field+str(v).replace('/','_'),lambda d,k=field,v=v:d['state'].update({k:v}))
for value in [0,6]:mutate('badconfig'+str(value),lambda d,v=value:d['config'].update(initial_speed=v))
for yy,mm,dd in [(0,1,1),(1900,2,29),(2000,2,30),(2000,13,1)]:mutate(f'baddate{yy}-{mm}-{dd}',lambda d,yy=yy,mm=mm,dd=dd:d['state'].update(date={'year':yy,'month':mm,'day':dd}))
mutate('queue-duplicate',lambda d:d['queue'].insert(0,copy.deepcopy(d['queue'][0])));mutate('queue-unsorted',lambda d:d['queue'].reverse());mutate('queue-past',lambda d:d['queue'][0].update(tick=22))
for v in [0,6]:mutate('queue-badspeed'+str(v),lambda d,v=v:d['queue'][0].update(command={'SetSpeed':v}))
mutate('world-none',lambda d:d.update(world=None));mutate('definitions',lambda d:d['world'].update(definitions_hash=0))
for kind in ['nations','states','provinces']:
 mutate(kind+'-duplicate',lambda d,k=kind:d['world']['inputs'][k].insert(0,copy.deepcopy(d['world']['inputs'][k][0])));mutate(kind+'-missing',lambda d,k=kind:d['world']['inputs'][k].pop());mutate(kind+'-unknown',lambda d,k=kind:d['world']['inputs'][k][0].update(id=60000))
for field,value in [('tag','XXX'),('government',''),('support',[]),('support',[['x',4294967295]]),('support',[['x',4294967297]]),('support',[['x',-1],['y',4294967297]]),('support',[['x',2147483648],['x',2147483648]])]:mutate('nation-'+field+str(len(RESULT)),lambda d,k=field,v=value:d['world']['inputs']['nations'][0].update({k:v}))
for field,value in [('owner',60000),('population',-1),('base',-1),('resources',[['unknown',1]]),('resources',[['steel',-1]]),('resources',[['steel',1],['steel',1]]),('buildings',[['unknown',1]]),('buildings',[['industry',-1]])]:mutate('state-'+field+str(len(RESULT)),lambda d,k=field,v=value:d['world']['inputs']['states'][0].update({k:v}))
for field,value in [('state',None),('state',2),('owner',2),('controller',60000),('controller',None)]:mutate('province-'+field+str(len(RESULT)),lambda d,k=field,v=value:d['world']['inputs']['provinces'][0].update({k:v}))
for field in ['state','owner','controller']:mutate('water-'+field,lambda d,k=field:d['world']['inputs']['provinces'][-1].update({k:1}))
for field,value in [('source',''),('target_stat','other'),('value',2**63-1)]:mutate('modifier-'+field,lambda d,k=field,v=value:d['world']['inputs']['states'][0]['modifiers'][0].update({k:v}))
mutate('modifier-duplicate',lambda d:d['world']['inputs']['states'][0]['modifiers'].insert(0,copy.deepcopy(d['world']['inputs']['states'][0]['modifiers'][0])))
mutate('ledger-bit',lambda d:d['world']['inputs']['states'][0]['ledger']['entries'][1].update(accumulated=4294967298));mutate('applied-bit',lambda d:d['world']['inputs']['states'][0].update(infrastructure=6442450946));mutate('ledger-tick',lambda d:d['world']['inputs']['states'][0]['ledger'].update(tick=22))
# Deliberately fractional base with independent Add then Mul flooring and expiry.
for tick,expected in [(23,19327352833),(24,12884901889),(25,12884901889)]:
 d=copy.deepcopy(base);d['state'].update(tick=tick,hour=tick%24,date={'year':2000,'month':2,'day':28+tick//24});d['queue']=[q for q in d['queue'] if q['tick']>=tick];t=d['world']['inputs']['states'][0];t['base']=2*4294967296+1;t['modifiers']=[{'source':'a.fraction','target_stat':'infrastructure','op':'Add','value':4294967296,'expires':24},{'source':'b.fraction','target_stat':'infrastructure','op':'Mul','value':6442450944,'expires':None}];recompute(d);assert t['infrastructure']==expected;trial('fraction-'+str(tick),d,True)
# Expired duplicate inputs remain, source registry intentionally nonempty-only current contract.
d=copy.deepcopy(base);t=d['world']['inputs']['states'][0];t['modifiers']=[{'source':'expired','target_stat':'infrastructure','op':'Add','value':1,'expires':0}]*2;recompute(d);trial('expired-duplicates-retained',d,True)
(C/'dto-results.json').write_text(json.dumps(RESULT,indent=2));print(json.dumps({'field_checks':'all header/body bytes and independent canonical/hash/ledger verified','dto_count':len(RESULT),'native_pids':[a['split']['pid'],b['loaded']['pid']],'split_hash':a['split']['hash'],'end_hash':end['hash']}))
