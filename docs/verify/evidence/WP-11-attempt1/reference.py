import pathlib,json,copy,hashlib,datetime
p=pathlib.Path('target/wp11-independent/own-native');d=json.loads((p/'initial.json').read_text())
def u(n):
 assert 0<=n<2**64
 out=bytearray()
 while n>127:out.append((n%128)|128);n//=128
 out.append(n);return bytes(out)
def i(n):return u(2*n if n>=0 else -2*n-1)
def st(s):b=s.encode();return u(len(b))+b
def seq(a,f):return u(len(a))+b''.join(map(f,a))
def opt(v,f=u):return b'\0' if v is None else b'\1'+f(v)
def mp(a):return seq(a,lambda kv:st(kv[0])+i(kv[1]))
def mod(m):return st(m['source'])+st(m['target_stat'])+u(['Add','Mul'].index(m['op']))+i(m['value'])+opt(m['expires'])
def row(r):return opt(r['source'],st)+u(['Base','Add','Mul'].index(r['op']))+i(r['value'])+i(r['accumulated'])
def led(l):return st(l['target_stat'])+u(l['tick'])+i(l['value'])+seq(l['entries'],row)
def state(s):return u(s['id'])+u(s['owner'])+i(s['population'])+mp(s['resources'])+mp(s['buildings'])+i(s['base'])+seq(s['modifiers'],mod)+i(s['infrastructure'])+led(s['ledger'])
def nation(n):return u(n['id'])+st(n['tag'])+st(n['government'])+mp(n['support'])
def prov(v):return u(v['id'])+opt(v['state'])+opt(v['owner'])+opt(v['controller'])
def queue(q):
 name,arg=next(iter(q['command'].items()));return u(q['tick'])+u(q['nation'])+u(q['sequence'])+u(['Pause','SetSpeed'].index(name))+bytes([int(arg)])
def canon(d):
 s=d['state'];c=d['config'];w=d['world'];dt=s['date']
 b=st(s['scenario'])+u(s['seed'])+u(s['tick'])+u(dt['year'])+bytes([dt['month'],dt['day'],s['hour'],s['paused'],s['speed']])+b''.join(u(x) for x in c['speed_ms_per_tick'])+bytes([c['initial_speed']])+seq(d['queue'],queue)
 if w:b+=b'\1'+u(w['definitions_hash'])+seq(w['inputs']['nations'],nation)+seq(w['inputs']['states'],state)+seq(w['inputs']['provinces'],prov)
 return b
def fnv(b):
 h=14695981039346656037
 for x in b:h=((h^x)*1099511628211)%2**64
 return f'{h:016x}'
def recalc(v):
 for s in v['world']['inputs']['states']:
  x=s['base'];rs=[dict(source=None,op='Base',value=x,accumulated=x)]
  for m in sorted(s['modifiers'],key=lambda m:(['Add','Mul'].index(m['op']),m['source'])):
   if m['expires'] is not None and v['state']['tick']>=m['expires']:continue
   x=x+m['value'] if m['op']=='Add' else (x*m['value'])//2**32
   assert -2**63<=x<2**63
   rs.append(dict(source=m['source'],op=m['op'],value=m['value'],accumulated=x))
  s['ledger']=dict(target_stat='infrastructure',tick=v['state']['tick'],value=x,entries=rs);s['infrastructure']=x

def expected(tick):
 v=copy.deepcopy(d);v['state']['tick']=tick
 day=datetime.date(2000,2,28)+datetime.timedelta(days=tick//24);v['state']['date']=dict(year=day.year,month=day.month,day=day.day);v['state']['hour']=tick%24
 v['state']['speed']=1 if tick<=23 or tick>49 else 4
 v['queue']=[q for q in v['queue'] if q['tick']>=tick];recalc(v);return v
for name,t in [('initial',0),('split',23),('loaded',23),('expiry',24),('continuous',54),('resumed',54)]:
 a=json.loads((p/(name+'.json')).read_text());e=expected(t);assert a==e,name
 b=canon(e);assert b==(p/(name+'.canonical')).read_bytes(),name
 print(name,'tick',t,'FNV',fnv(b),'SHA',hashlib.sha256(b).hexdigest(),'rawbits',e['world']['inputs']['states'][0]['infrastructure'])
 (p/(name+'.expected.canonical')).write_bytes(b)
assert (p/'resumed.canonical').read_bytes()==(p/'continuous.canonical').read_bytes()
assert pathlib.Path('target/wp11-independent/own-cli-resume.log').read_text().strip()==fnv(canon(expected(54)))
cases=[]
def add(name,change,ok=False,base=None):
 v=copy.deepcopy(base or d);change(v);cases.append(dict(name=name,ok=ok,dto=v))
def setpath(v,path,x):
 for k in path[:-1]:v=v[k]
 v[path[-1]]=x
W=['world','inputs'];S=W+['states',0];L=S+['ledger'];E=L+['entries',1];N=W+['nations',0];P=W+['provinces',0]
for path,val in [(['state','date','year'],0),(['state','date','month'],13),(['state','date','day'],30),(['state','hour'],24),(['state','tick'],1),(['state','speed'],0),(['state','speed'],6),(['config','initial_speed'],0),(['world'],None),(['world','definitions_hash'],0),(N+['government'],''),(N+['tag'],'BAD'),(N+['support',0,1],2**32),(N+['support',0,0],''),(S+['owner'],0),(S+['population'],-1),(S+['base'],-1),(S+['resources',0,0],'unknown'),(S+['buildings',0,0],'unknown'),(S+['buildings',0,1],-1),(S+['modifiers',0,'source'],''),(S+['modifiers',0,'target_stat'],'other'),(S+['infrastructure'],1),(L+['tick'],1),(L+['value'],1),(L+['target_stat'],'other'),(E+['source'],'forged'),(E+['op'],'Mul'),(E+['value'],2),(E+['accumulated'],1),(P+['state'],None),(P+['owner'],2),(P+['controller'],0),(W+['provinces',4,'owner'],1)]:add('/'.join(map(str,path)),lambda v,path=path,val=val:setpath(v,path,val))
for collection in [W+['nations'],W+['states'],W+['provinces'],N+['support'],S+['resources'],S+['buildings'],['queue']]:
 def dup(v,p=collection):
  for k in p:v=v[k]
  v.insert(0,copy.deepcopy(v[0]))
 add('duplicate '+str(collection),dup)
for path in [W+['nations'],W+['states'],W+['provinces'],N+['support'],S+['modifiers'],L+['entries'],['queue']]:
 def rev(v,p=path):
  for k in p:v=v[k]
  v.reverse()
 add('reverse '+str(path),rev)
for path in [W+['nations'],W+['states'],W+['provinces']]:
 def drop(v,p=path):
  for k in p:v=v[k]
  v.pop()
 add('missing '+str(path),drop)
for speed in [0,6]:add('queue invalid speed '+str(speed),lambda v,speed=speed:setpath(v,['queue',0,'command'],{'SetSpeed':speed}))
add('past queue',lambda v:setpath(v,['queue',0,'tick'],22),base=expected(23))
for speed in range(1,6):
 def valid(v,speed=speed):v['state']['speed']=speed;v['config']={'speed_ms_per_tick':[0,7,11,13,17],'initial_speed':2}
 add('valid speed '+str(speed),valid,True)
add('government not registered nonempty',lambda v:setpath(v,N+['government'],'government.unregistered'),True)
for bits in [-2**63,2**63-1,-1,0,1]:
 def extreme(v,bits=bits):
  s=v['world']['inputs']['states'][1];s['modifiers']=[dict(source='raw',target_stat='infrastructure',op='Add',value=bits,expires=2**64-1)];recalc(v)
 add('signed bits '+str(bits),extreme,True)
for tick in [23,24,25]:
 v=expected(tick);s=v['world']['inputs']['states'][0];s['base']=2*2**32+1;s['modifiers']=[dict(source='a',target_stat='infrastructure',op='Add',value=2**32,expires=24),dict(source='b',target_stat='infrastructure',op='Mul',value=3*2**31,expires=None)];recalc(v)
 assert s['infrastructure']==(19327352833 if tick==23 else 12884901889)
 cases.append(dict(name='fractional DTO boundary '+str(tick),ok=True,dto=v))
def expired(v):
 s=v['world']['inputs']['states'][0];s['modifiers']=[dict(source='a',target_stat='infrastructure',op='Add',value=1,expires=0)]*2;recalc(v)
add('expired duplicates retained',expired,True)
def same_source(v):
 s=v['world']['inputs']['states'][0];s['modifiers']=[dict(source='a',target_stat='infrastructure',op='Add',value=1,expires=None),dict(source='a',target_stat='infrastructure',op='Mul',value=-2**32,expires=None)];recalc(v)
add('same source Add Mul signed',same_source,True)
add('active duplicate',lambda v:v['world']['inputs']['states'][0]['modifiers'].insert(0,copy.deepcopy(v['world']['inputs']['states'][0]['modifiers'][0])))
add('overflow before later cancellation',lambda v:setpath(v,S+['modifiers',0,'value'],2**63-1))
(p/'cases.json').write_text(json.dumps(cases));print('independent DTO cases',len(cases))
