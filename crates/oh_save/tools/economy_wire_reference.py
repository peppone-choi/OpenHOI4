"""Economic wire relations derived from the independently checked paused v5 DTO."""
from decimal import Decimal,localcontext
from fractions import Fraction
import json,re
from pathlib import Path
from economy_reference import verify_report

def number(v,bits):return dict(bits=str(v['bits']),fractional_bits=bits)
def law_values(values,defs):return [dict(category=k,law=v,name_key=defs['laws'][v]['name_key']) for k,v in sorted(values.items())]
def resources(values):return [dict(resource=k,value=str(v)) for k,v in sorted(values.items())]
def dormant(value):return None if value is None else {'NoOwnership':'economy-no-ownership','TargetConflict':'economy-target-conflict','SlotCap':'economy-slot-cap','ZeroCap':'economy-zero-cap','ZeroFactor':'economy-zero-factor'}[value]
def action(c):
    key,v=next(iter(c.items()))
    if key=='Allocate':return dict(type=key,ratios_bits=[str(x['bits']) for x in v['ratios']])
    if key=='Construct':return dict(type=key,project=str(v['project']),state=v['state'],building=v['building'])
    if key=='Cancel':return dict(type=key,project=str(v['project']))
    if key=='Reorder':return dict(type=key,projects=[str(x) for x in v['projects']])
    return dict(type=key,law=v['law'])
def command(c):
    key,v=next(iter(c.items()))
    if key=='Economy':return dict(type=key,command=action(v))
    if key=='Pause':return dict(type=key,paused=v)
    if key=='SetSpeed':return dict(type=key,speed=v)
    return dict(type=key,**v)
def expected_view(report,defs):
    verify_report(report);dto=report['dto'];e=dto['economy'];nations=[]
    for id,n in sorted(e['nations'].items(),key=lambda p:int(p[0])):
        l=n['ledger'];c=n['construction'];available=max(0,n['capacity']-n['committed']-n['reserved']);over=max(0,n['committed']+n['reserved']-n['capacity'])
        ledger=dict(tick=str(l['tick']),laws=law_values(l['laws'],defs),stability=number(l['stability'],32),capacity=str(l['capacity']),contributions=[dict(state=v['state'],building=v['building'],levels=str(v['levels']),unit_ic=number(v['unit_ic'],16),value=number(v['value'],16)) for v in l['contributions']],state_flows=[dict(state=int(state),population=str(population),resources=resources(l['resources_by_state'][state])) for state,population in sorted(l['population_by_state'].items(),key=lambda p:int(p[0]))],population=str(l['population']),resources=resources(l['resources']),multipliers=[dict(source=v['source'],factor=number(v['factor'],32),applied=number(v['applied'],16)) for v in l['multipliers']],total_ic=number(l['total_ic'],16),minimum=number(l['minimum'],32),ratios=[number(v,32) for v in l['ratios']],allocation=[number(v,16) for v in l['allocation']],consumer_residual=number(l['consumer_residual'],16))
        construction=dict(tick=str(c['tick']),budget=number(c['budget'],16),entries=[dict(state=v['state'],building=v['building'],target=str(v['target']),starting_progress=number(v['starting_progress'],16),cost=number(v['cost'],16),daily_cap=number(v['daily_cap'],16),infrastructure=number(v['infrastructure'],32),owner=v['owner'],factor_evaluated=v['factor_evaluated'],project=str(v['project']),factor=number(v['factor'],32),consumed=number(v['consumed'],16),applied=number(v['applied'],16),discarded=number(v['discarded'],16),completed=v['completed'],dormancy=dormant(v['dormancy'])) for v in c['entries']],unused=number(c['unused'],16))
        nations.append(dict(nation=int(id),laws=law_values(n['laws'],defs),political_capital=number(n['political_capital'],16),stability=number(n['stability'],32),mobilization=number(n['mobilization'],32),ratios=[number(v,32) for v in n['allocation']],committed=str(n['committed']),reserved=str(n['reserved']),capacity=str(n['capacity']),available=str(available),overcommitted=str(over),projects=[dict(id=str(p['id']),state=p['state'],building=p['building'],target=str(p['target']),progress=number(p['progress'],16),dormancy=dormant(p['dormancy'])) for p in n['projects']],ledger=ledger,construction=construction))
    scores=None if e['scores'] is None else [dict(nation=int(id),tick=str(s['tick']),input_tick=str(s['input_tick']),weight=number(s['weight'],32),input=number(s['input'],16),term=number(s['term'],16)) for id,s in sorted(e['scores'].items(),key=lambda p:int(p[0]))]
    return dict(state_hash=report['hash'],definitions_hash=f"{e['definitions_hash']:016x}",pending=[dict(tick=str(q['tick']),nation=q['nation'],sequence=str(q['sequence']),command=command(q['command'])) for q in dto['queue']],nations=nations,industrial_scores=scores)
def same(actual,expected):
    if type(expected) is dict:
        assert type(actual) is dict
        if set(expected)=={'bits','fractional_bits'}:
            assert set(actual)=={'bits','fractional_bits','value'} and type(actual['bits']) is str and actual['bits']==expected['bits'] and type(actual['fractional_bits']) is int and actual['fractional_bits']==expected['fractional_bits']
            value=actual['value'];assert type(value) is str and len(value)<=128 and re.fullmatch(r'-?(0|[1-9][0-9]*)(\.[0-9]+)?',value)
            scaled=Fraction(value)*(1<<expected['fractional_bits']);q,r=divmod(scaled.numerator,scaled.denominator);rounded=q+int(r*2>scaled.denominator or (r*2==scaled.denominator and q%2!=0));assert rounded==int(expected['bits']);return
        assert set(actual)==set(expected)
        for key,v in expected.items():same(actual[key],v)
    elif type(expected) is list:
        assert type(actual) is list and len(actual)==len(expected)
        for a,b in zip(actual,expected):same(a,b)
    else:assert type(actual) is type(expected) and actual==expected

def filled(value):
    # Explicit synthetic evidence test control only; never capture server output.
    if type(value) is dict:
        if set(value)=={'bits','fractional_bits'}:
            with localcontext() as context:
                context.prec=100;decimal=format(Decimal(value['bits'])/Decimal(1<<value['fractional_bits']),'f')
                if '.' in decimal:decimal=decimal.rstrip('0').rstrip('.')
                return {**value,'value':decimal}
        return {k:filled(v) for k,v in value.items()}
    return [filled(v) for v in value] if type(value) is list else value

def verify_query(query,record,defs):
    assert type(query['pid']) is int and query['pid']>0
    source=record['paused'];expected=expected_view(source,defs)
    for name in ('before','after'):
        response=query[name];same(response,dict(type='EconomyResult',request=name,supported=True,reason_key=None,economy=expected))
    state=source['dto']['base']['base']['base']['state'];time=dict(date=f"{state['date']['year']:04}-{state['date']['month']:02}-{state['date']['day']:02}",hour=state['hour'],tick=str(state['tick']),paused=state['paused'],speed=state['speed'])
    same(query['snapshot'],dict(type='Snapshot',state=time))
    same(query['accepted'],dict(type='CommandResult',sequence='1',accepted=True,reason_key=None))
    assert len(query['rejected'])==3
    for index,response in enumerate(query['rejected'],2):same(response,dict(type='CommandResult',sequence=str(index),accepted=False,reason_key='invalid-message'))
    for message in [query['welcome'],query['snapshot'],query['before'],query['accepted'],*query['rejected'],query['after']]:assert query['messages'].count(message)>=1
    assert query['welcome']['type']=='Welcome' and query['welcome']['accepted'] is True and query['welcome']['packs']==[dict(id=record['pack']['id'],version=record['pack']['version'],hash=f"{record['pack']['content_hash']:016x}")]
