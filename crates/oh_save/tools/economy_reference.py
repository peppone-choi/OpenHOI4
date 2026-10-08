"""Independent Postcard encoding for the v5 economy native fixture (no movement)."""
from save_reference import varint as _varint, signed as _signed, string, sequence, date, nation, state_world, province, fnv

def varint(n):
    assert type(n) is int and 0<=n<1<<64
    return _varint(n)
def signed(n):
    assert type(n) is int and -(1<<63)<=n<1<<63
    return _signed(n)
def exact(value,keys):assert type(value) is dict and set(value)==set(keys.split())
def numeric_types(value,path=()):
    if type(value) is dict:
        for k,v in value.items():numeric_types(v,(*path,k))
    elif type(value) is list:
        for v in value:numeric_types(v,(*path,'item'))
    elif type(value) is bool:assert path[-1] in ('paused','completed','movement_present','strait_present','factor_evaluated')
    else:assert value is None or type(value) in (int,str)

def fixed(value):
    assert set(value)=={'bits'} and type(value['bits']) is int and -(1<<63)<=value['bits']<(1<<63)
    return signed(value['bits'])
def mapping(value,key,encode):
    return varint(len(value))+b''.join(key(k)+encode(v) for k,v in sorted(value.items(),key=lambda p:int(p[0]) if key is identifier else p[0]))
def identifier(key):
    assert type(key) is str and key==str(int(key)) and 0<=int(key)<=65535
    return varint(int(key))
def resources(value): return mapping(value,string,signed)
def optional(value,encode): return b'\0' if value is None else b'\1'+encode(value)
def dormant(value): return varint(['NoOwnership','TargetConflict','SlotCap','ZeroCap','ZeroFactor'].index(value))
def action(value):
    assert type(value) is dict and len(value)==1
    key,v=next(iter(value.items()));tag=['Allocate','Construct','Cancel','Reorder','ChangeLaw'].index(key)
    if tag==0: data=b''.join(fixed(x) for x in v['ratios']);assert len(v['ratios'])==4
    elif tag==1:data=varint(v['project'])+varint(v['state'])+string(v['building'])
    elif tag==2:data=varint(v['project'])
    elif tag==3:data=sequence(v['projects'],varint)
    else:data=string(v['law'])
    return varint(tag)+data

def command(value):
    assert type(value) is dict and len(value)==1
    key,v=next(iter(value.items()));tag=['Pause','SetSpeed','Move','Stop','Effects','Economy'].index(key)
    if tag==0:data=bytes([int(v)]);assert type(v) is bool
    elif tag==1:data=bytes([v])
    elif tag==2:data=varint(v['unit'])+varint(v['destination'])
    elif tag==3:data=varint(v['unit'])
    elif tag==4:data=string(v['program'])
    else:data=action(v)
    return varint(tag)+data

def queue(value):return varint(value['tick'])+varint(value['nation'])+varint(value['sequence'])+command(value['command'])
def project(p):return varint(p['id'])+varint(p['state'])+string(p['building'])+signed(p['target'])+fixed(p['progress'])+optional(p['dormancy'],dormant)
def contribution(c):return varint(c['state'])+string(c['building'])+signed(c['levels'])+fixed(c['unit_ic'])+fixed(c['value'])
def multiplier(m):return string(m['source'])+fixed(m['factor'])+fixed(m['applied'])
def ledger(l):
    exact(l,"tick laws stability capacity contributions population_by_state resources_by_state population resources multipliers total_ic minimum ratios allocation consumer_residual")
    assert len(l['ratios'])==len(l['allocation'])==4
    return (varint(l['tick'])+mapping(l['laws'],string,string)+fixed(l['stability'])+signed(l['capacity'])+sequence(l['contributions'],contribution)+mapping(l['population_by_state'],identifier,signed)+mapping(l['resources_by_state'],identifier,resources)+signed(l['population'])+resources(l['resources'])+sequence(l['multipliers'],multiplier)+fixed(l['total_ic'])+fixed(l['minimum'])+b''.join(fixed(v) for v in l['ratios'])+b''.join(fixed(v) for v in l['allocation'])+fixed(l['consumer_residual']))
def construction_entry(e):return varint(e['project'])+b''.join(fixed(e[k]) for k in ('factor','consumed','applied','discarded'))+bytes([int(e['completed'])])+optional(e['dormancy'],dormant)+varint(e['state'])+string(e['building'])+signed(e['target'])+b''.join(fixed(e[k]) for k in ('starting_progress','cost','daily_cap','infrastructure'))+varint(e['owner'])+bytes([int(e['factor_evaluated'])])
def construction(c):return varint(c['tick'])+fixed(c['budget'])+sequence(c['entries'],construction_entry)+fixed(c['unused'])
def economy_nation(n):
    exact(n,"laws political_capital stability mobilization allocation committed reserved capacity projects ledger construction")
    assert len(n['allocation'])==4
    return (mapping(n['laws'],string,string)+b''.join(fixed(n[k]) for k in ('political_capital','stability','mobilization'))+b''.join(fixed(v) for v in n['allocation'])+b''.join(signed(n[k]) for k in ('committed','reserved','capacity'))+sequence(n['projects'],project)+ledger(n['ledger'])+construction(n['construction']))
def score(s):return varint(s['tick'])+varint(s['input_tick'])+fixed(s['weight'])+fixed(s['input'])+fixed(s['term'])
def economy(e):
    exact(e,'definitions_hash nations scores')
    return varint(e['definitions_hash'])+mapping(e['nations'],identifier,economy_nation)+optional(e['scores'],lambda v:mapping(v,identifier,score))
def end_cause(c):
    if c=='Date':return b'\0'
    if c=='Condition':return b'\1'
    assert set(c)=={'Explicit'};return b'\2'+string(c['Explicit'])
def checkpoint(e):return varint(e['tick'])+date(e['date'])+bytes([e['hour']])+sequence(e['causes'],end_cause)
def trigger(t):return varint(t['definitions_hash'])+sequence(t['flags'],lambda p:varint(p[0])+sequence(p[1],string))+optional(t['ended'],checkpoint)
def canonical(dto):
    numeric_types(dto);exact(dto,"base movement_present strait_present trigger queue economy")
    assert dto['movement_present'] is False and dto['strait_present'] is False
    base=dto['base']['base']['base'];s,c=base['state'],base['config'];assert not base['queue'] and not dto['base']['base']['queue'] and not dto['base']['base']['units'] and not dto['base']['straits']
    data=(string(s['scenario'])+varint(s['seed'])+varint(s['tick'])+date(s['date'])+bytes([s['hour'],int(s['paused']),s['speed']])+b''.join(varint(v) for v in c['speed_ms_per_tick'])+bytes([c['initial_speed']])+sequence(dto['queue'],queue))
    w=base['world'];assert w is not None;i=w['inputs']
    data+=b'\1'+varint(w['definitions_hash'])+sequence(i['nations'],nation)+sequence(i['states'],state_world)+sequence(i['provinces'],province)
    if dto['trigger'] is not None:data+=b'\1'+trigger(dto['trigger'])
    return data+b'\1'+economy(dto['economy'])
def verify_report(report):
    assert report['format']==5
    encoded=canonical(report['dto'])
    assert report['canonical_hex']==encoded.hex(), 'economy DTO/canonical disagreement'
    assert report['hash']==fnv(encoded), 'economy canonical/hash disagreement'
    assert report['ended'] is (report['dto']['trigger'] is not None and report['dto']['trigger']['ended'] is not None)
    # Independent raw integer expectations for this actual synthetic workload.
    for nation_id,n in report['dto']['economy']['nations'].items():
        l=n['ledger'];total=l['total_ic']['bits'];assert sum(v['bits'] for v in l['allocation'])==total
        population=sum(l['population_by_state'].values());assert population==l['population']
        assert n['capacity']==population//8
        assert n['committed']==60 and n['reserved']==10
        consumed=sum(e['consumed']['bits'] for e in n['construction']['entries']);assert consumed+n['construction']['unused']['bits']==n['construction']['budget']['bits']
        assert n['political_capital']['bits']==min(100,3*(report['dto']['base']['base']['base']['state']['tick']//24))*(1<<16)
    return encoded
