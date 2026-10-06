import json, subprocess, pathlib, hashlib, tomllib
from fractions import Fraction
from jsonschema import Draft202012Validator
base=pathlib.Path(__file__).resolve().parent
probe=base/'build/debug/wp03_verify_probe.exe'
manifest='id = "verify_pack"\nname_key = "verify_name"\nversion = "0.1.0"\nengine = ">=0.1, <0.2"\n'
results=[]

def run(name, text, expected=None, error=None, manifest_text=manifest, location=None):
    root=base/'cases'/name
    root.mkdir(parents=True, exist_ok=True)
    (root/'manifest.toml').write_bytes(manifest_text.encode())
    (root/'defines.toml').write_bytes(text.encode())
    p=subprocess.run([str(probe),str(root)],capture_output=True,text=True,encoding='utf-8',check=True)
    data=json.loads(p.stdout)
    if error:
        assert data['ok'] is False and data['kind']==error,(name,data)
        if location: assert (data['line'],data['column'])==location,(name,data)
    else:
        assert data['ok'] is True,(name,data)
        if expected is not None: assert data['values']['verify.x']==expected,(name,data,expected)
    results.append({'name':name,'result':data})
    return data

for i,lexeme in enumerate(['0.0','-0.0','+0.0','0.1','0.2','-2.5','+2.5','1e2','1E-2','1_000.2_5','1.0e+3','1.0e-30','2147483647.99999999976716935634613037109375','-2147483648.0','1.000000000116415321826934814453125','1.000000000116415321826934814453126','1.000000000116415321826934814453124','1.000000000349245965480804443359375','-1.000000000116415321826934814453125','-1.000000000116415321826934814453126','0.000000000116415321826934814453125','0.000000000349245965480804443359375']):
    bits=round(Fraction(lexeme.replace('_',''))*(1<<32))
    run(f'fixed-{i}',f'[verify]\nx = {lexeme}\n',{'bits':bits})
for i,lexeme in enumerate(['2147483648.0','-2147483649.0','1e100','nan','-nan','+inf','-inf']):
    run(f'fixed-error-{i}',f'[verify]\nx = {lexeme}\n',error='Schema',location=(2,5))
for i,(lexeme,integer) in enumerate([('9223372036854775807',2**63-1),('-9223372036854775808',-2**63),('+42',42),('1_000_000',1000000),('0xff_ff',65535),('0o7_7',63),('0b10_10',10)]):
    run(f'integer-{i}',f'[verify]\nx = {lexeme}\n',{'integer':integer})
for i,lexeme in enumerate(['9223372036854775808','-9223372036854775809']):
    run(f'integer-error-{i}',f'[verify]\nx = {lexeme}\n',error='Schema',location=(2,5))
run('crlf-unicode','[verify]\r\n"수치😀" = 0.25\r\nx = [1, "오류"]\r\n',error='Schema',location=(3,9))
run('unicode-column','[verify]\r\n"😀수치" = false\r\n',error='Schema',location=(2,9))
run('array','[verify]\nx = [1, 0.5, -2]\n',[{'integer':1},{'bits':1<<31},{'integer':-2}])
run('empty-array','[verify]\nx = []\n',[])
first=run('mutated','[verify]\nx = 0.1\n',{'bits':round(Fraction(1,10)*(1<<32))})
second=run('mutated','[verify]\nx = 0.2\n',{'bits':round(Fraction(1,5)*(1<<32))})
assert first['values']!=second['values']
for i,text in enumerate(['[verify]\nx = true\n','[verify]\nx = "0.1"\n','[verify]\nx = 2026-10-06\n','x = 1\n','[verify]\nx = { inner = 1 }\n','[verify]\nx = [[1]]\n']):
    run(f'shape-error-{i}',text,error='Schema')
for i,text in enumerate(['[verify]\nx = [1,\n','[verify]\nx = 1\nx = 2\n']):
    run(f'syntax-{i}',text,error='Syntax')
for i,extra in enumerate(['depends = [{ id = "ok", version = ">=0.1", typo = 1 }]\n','depends = "bad"\n','load_after = [1]\n','conflicts = [false]\n']):
    run(f'manifest-error-{i}','',manifest_text=manifest+extra,error='Schema')

subprocess.run([str(probe),'schema',str(base/'generated')],check=True)
subprocess.run([str(probe),'schema',str(base/'generated-second')],check=True)
repo=base.parents[2]
for name in ['manifest','defines']:
    actual=(base/'generated'/f'{name}.schema.json').read_bytes()
    again=(base/'generated-second'/f'{name}.schema.json').read_bytes()
    original=(repo/'crates/oh_data/schema'/f'{name}.schema.json').read_bytes()
    assert actual==again==original,(name,'schema bytes changed')
    print(name,'schema SHA256',hashlib.sha256(actual).hexdigest())
    Draft202012Validator.check_schema(json.loads(actual))
validators={name:Draft202012Validator(json.loads((base/'generated'/f'{name}.schema.json').read_text())) for name in ['manifest','defines']}
# Schema covers structural types; fixed ranges and VersionReq grammar are extra semantic checks as documented.
shape_cases=[('[verify]\nx=1\n',True),('[verify]\nx=0.1\n',True),('[verify]\nx=[]\n',True),('[verify]\nx=[1,0.1]\n',True),('[verify]\nx=true\n',False),('[verify]\nx="1"\n',False),('x=1\n',False),('[verify]\nx=[[1]]\n',False),('[verify]\nx={ a=1 }\n',False)]
for i,(text,ok) in enumerate(shape_cases):
    obj=tomllib.loads(text)
    schema_ok=validators['defines'].is_valid(obj)
    loader=run(f'schema-shape-{i}',text,error=None if ok else 'Schema')
    assert schema_ok==ok==loader['ok'],(i,obj,schema_ok,loader)
for i,extra in enumerate(['','depends = [{ id="base", version=">=0.1" }]\n','conflicts=["other"]\n','load_after=["base"]\n','unexpected=1\n','depends=[{ id="base" }]\n','engine_typo=1\n']):
    obj=tomllib.loads(manifest+extra)
    ok=i<4
    assert validators['manifest'].is_valid(obj)==ok,(i,obj)
    run(f'schema-manifest-{i}','',manifest_text=manifest+extra,error=None if ok else 'Schema')
(base/'independent-results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
print(f'PASS {len(results)} independent loads; rational rounding, range, mutation, CRLF/Unicode, structural schema and exact regeneration checked')
