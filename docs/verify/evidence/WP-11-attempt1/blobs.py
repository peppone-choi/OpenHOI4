import json,pathlib,copy,struct,tomllib,hashlib
from reference import u,i,st,seq,opt,canon,fnv,p
saved=(p/'saved.ohsave').read_bytes();assert saved[:6]==b'OHSV\1\0';end=10+int.from_bytes(saved[6:10],'little');raw=saved[10:end];at=0
def num():
 global at
 n=shift=0
 while True:
  b=raw[at];at+=1;n|=(b&127)<<shift
  if b<128:return n
  shift+=7
def string():
 global at
 n=num();b=raw[at:at+n];at+=n;return b.decode()
h={'engine_version':string(),'format_version':num(),'scenario_id':string()};h['packs']=[dict(id=string(),version=string(),content_hash=num())for _ in range(num())]
h['game_date']={'year':num(),'month':raw[at],'day':raw[at+1]};at+=2
for f in ['tick','seed','state_hash']:h[f]=num()
h['player_nations']=[num()for _ in range(num())];z=num();h['saved_at_utc']=z//2 if z%2==0 else -(z//2)-1
assert raw[at]==1;at+=1;h['definitions_hash']=num();h['effective_defines_hash']=num();assert at==len(raw)
def eh(h):return st(h['engine_version'])+u(h['format_version'])+st(h['scenario_id'])+seq(h['packs'],lambda v:st(v['id'])+st(v['version'])+u(v['content_hash']))+u(h['game_date']['year'])+bytes([h['game_date']['month'],h['game_date']['day']])+u(h['tick'])+u(h['seed'])+u(h['state_hash'])+seq(h['player_nations'],u)+i(h['saved_at_utc'])+opt(h['definitions_hash'])+u(h['effective_defines_hash'])
assert eh(h)==raw
pack=pathlib.Path('target/wp11-independent/own-pack');files=sorted((f.relative_to(pack).as_posix(),f.read_bytes())for f in pack.rglob('*')if f.is_file());packh=int(fnv(seq(files,lambda kv:st(kv[0])+u(len(kv[1]))+kv[1])),16)
defines=tomllib.loads((pack/'defines.toml').read_text());defines.update(tomllib.loads((pack/'scenarios/m1/defines.toml').read_text()))
def number(x):assert isinstance(x,int);return u(0)+i(x)
def val(x):return u(1)+seq(x,number)if isinstance(x,list)else u(0)+number(x)
defbytes=seq(sorted(defines.items()),lambda kv:st(kv[0])+seq(sorted(kv[1].items()),lambda kv:st(kv[0])+val(kv[1])))
split=json.loads((p/'split.json').read_text())
assert h==dict(engine_version='0.1.0',format_version=1,scenario_id='m1',packs=[dict(id='testland',version='0.1.0',content_hash=packh)],game_date={'year':2000,'month':2,'day':28},tick=23,seed=123,state_hash=int(fnv(canon(split)),16),player_nations=[1,2],saved_at_utc=0,definitions_hash=split['world']['definitions_hash'],effective_defines_hash=int(fnv(defbytes),16))
(p/'independent-header.json').write_text(json.dumps(h,indent=2));print('all header fields and independent pack/effective defines hashes verified',hex(packh),hex(h['effective_defines_hash']))
body=canon(split);cases=[dict(name='independently encoded valid body/header',header=raw.hex(),body=body.hex(),ok=True)]
def headercase(name,path,value):
 v=copy.deepcopy(h);ref=v
 for k in path[:-1]:ref=ref[k]
 ref[path[-1]]=value;cases.append(dict(name=name,header=eh(v).hex(),body=body.hex(),ok=False))
for name,path,value in [('engine',['engine_version'],'0.2.0'),('format',['format_version'],0),('scenario',['scenario_id'],'other'),('tick',['tick'],24),('seed',['seed'],124),('date',['game_date','day'],29),('state hash',['state_hash'],0),('saved_at negative',['saved_at_utc'],-1),('saved_at future',['saved_at_utc'],253402300800),('definitions',['definitions_hash'],None),('effective defines',['effective_defines_hash'],0),('unknown player',['player_nations'],[0]),('duplicate player',['player_nations'],[1,1]),('player order',['player_nations'],[2,1]),('pack id',['packs',0,'id'],'other'),('pack count',['packs'],[])]:headercase(name,path,value)
cases+=[dict(name='noncanonical header varint',header=(bytes([0x85,0])+raw[1:]).hex(),body=body.hex(),ok=False),dict(name='header trailing',header=(raw+b'\0').hex(),body=body.hex(),ok=False),dict(name='body trailing',header=raw.hex(),body=(body+b'\0').hex(),ok=False),dict(name='body noncanonical string',header=raw.hex(),body=(bytes([0x82,0])+body[1:]).hex(),ok=False)]
for length in range(len(saved)):cases.append(dict(name='truncation '+str(length),full=saved[:length].hex(),ok=False))
for version in [0,2,65535]:cases.append(dict(name='prefix version '+str(version),full=(saved[:4]+struct.pack('<H',version)+saved[6:]).hex(),ok=False))
for length in [0,65537,4294967295]:cases.append(dict(name='header length '+str(length),full=(saved[:6]+struct.pack('<I',length)+saved[10:]).hex(),ok=False))
for name,bytes_ in [('magic',b'FAIL'+saved[4:]),('checksum',saved[:-1]+bytes([saved[-1]^1])),('double frame',saved+saved[end:]),('empty second frame',saved+bytes.fromhex('28b52ffd2000010000')),('trailing zero',saved+b'\0'),('skippable frame',saved[:end]+bytes.fromhex('502a4d1800000000')+saved[end:])]:cases.append(dict(name=name,full=bytes_.hex(),ok=False))
for offset,ok in [(-1,False),(0,True),(1,True)]:cases.append(dict(name='body cap '+str(offset),header=raw.hex(),body=body.hex(),body_cap=len(body)+offset,ok=ok))
for offset,ok in [(-1,False),(0,True),(1,True)]:cases.append(dict(name='file cap '+str(offset),full=saved.hex(),file_cap=len(saved)+offset,ok=ok))
# Unknown command enum, bool and world Option tags use independent field lengths.
s=split['state'];c=split['config'];pos=len(st(s['scenario'])+u(s['seed'])+u(s['tick'])+u(s['date']['year']))+5+sum(len(u(x))for x in c['speed_ms_per_tick'])+1+len(u(len(split['queue'])))
q=split['queue'][0];pos+=len(u(q['tick'])+u(q['nation'])+u(q['sequence']))
b=bytearray(body);b[pos]=2;cases.append(dict(name='unknown command enum',header=raw.hex(),body=b.hex(),ok=False))
(p/'blob-cases.json').write_text(json.dumps(cases));print('independent container cases',len(cases))
