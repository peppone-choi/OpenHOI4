import runpy,json,pathlib,copy,hashlib,subprocess
v=runpy.run_path('target/wp11-verify2/reference.py');globals().update({k:v[k] for k in ['R','O','P','PACK','C','u','i','s','opt','vec','date','dto','fnv','header','invoke','h','raw','base','pack_hash','verify','trial']})
out=O/'extended';out.mkdir(exist_ok=True);stable=O/'stable-pack';rep=json.loads((O/'stable-initial.log').read_text());verify(rep);d=rep['dto'];assert [n['id'] for n in d['world']['inputs']['nations']]==[0,65535];assert [t['id'] for t in d['world']['inputs']['states']]==[0,901];ps=d['world']['inputs']['provinces'];assert ps[0]=={'id':0,'state':0,'owner':0,'controller':0};assert ps[1]['owner']==0 and ps[1]['controller']==65535;assert all(ps[j]['owner'] is None and ps[j]['state'] is None and ps[j]['controller'] is None for j in [2,4]);trial('stable-valid',d,True,stable)
hh=copy.deepcopy(h);hh.update(game_date=d['state']['date'],tick=0,seed=77,state_hash=int(rep['hash'],16),player_nations=[0,65535],definitions_hash=d['world']['definitions_hash']);hh['packs'][0]['content_hash']=pack_hash(stable);body=out/'stable.body';body.write_bytes(dto(d));invoke(['compress',body,out/'stable.zst'],'stable-compress');hb=header(hh);save=out/'stable.ohsave';save.write_bytes(b'OHSV\1\0'+len(hb).to_bytes(4,'little')+hb+(out/'stable.zst').read_bytes());res=json.loads(invoke(['blob',stable,save],'stable-file'));assert res['ok'];assert res['report']['dto']==d;verify(res['report'])
for fld,val in [('state',None),('owner',None),('owner',65535),('controller',1)]:
 invalid=copy.deepcopy(d);invalid['world']['inputs']['provinces'][0][fld]=val;trial('stable-invalid-'+fld+str(val),invalid,False,stable)
# Native CLI force contract with full before/after file digests, including failed save replacement.
cli=R/'target/debug/oh_cli.exe';src=O/'own-native/saved.ohsave';rows=[]
def call(name,args,expect):
 old=src.read_bytes();f=out/(name+'.stdout');p=subprocess.run([str(cli),*map(str,args)],cwd=R,capture_output=True);f.write_bytes(p.stdout);(out/(name+'.stderr')).write_bytes(p.stderr);assert p.returncode==expect,(name,p.stderr);assert src.read_bytes()==old;rows.append({'name':name,'command':[str(cli),*map(str,args)],'exit':p.returncode,'stdout':p.stdout.decode(),'stderr':p.stderr.decode(),'old_sha':hashlib.sha256(old).hexdigest(),'after_sha':hashlib.sha256(src.read_bytes()).hexdigest()})
for packname in ['content-changed','defines-changed','definitions-changed']:
 for force in [False,True]:
  args=['resume','--load',src,'--pack',O/packname,'--ticks','0','--hash-out']+(['--force'] if force else []);expected=0 if packname=='content-changed' and force else 1;call(packname+str(force),args,expected)
call('junction-pack',['resume','--load',src,'--pack',O/'pack-junction','--ticks','0','--force','--hash-out'],1)
call('save-inside-junction',['run','--pack',PACK,'--scenario','m1','--ticks','0','--save-out',O/'pack-junction/forbidden.ohsave'],1);assert not (PACK/'forbidden.ohsave').exists()
# Failed save-out must preserve an existing file, with no useful old data deletion.
missing=out/'does-not-exist.ohsave';before=src.read_bytes();call('missing-input-saveout',['resume','--load',missing,'--pack',PACK,'--ticks','0','--save-out',src],1)
paused=O/'own-native/paused.ohsave';call('paused-resume-saveout',['resume','--load',paused,'--pack',PACK,'--ticks','1','--save-out',src],1)
# Tampered and unsupported format force CLI stay rejected.
for name in ['format0','zstd-corruption20','forged-ledger']:
 f=C/'blobs'/f'{name}.ohsave';call('corrupt-cli-'+name,['resume','--load',f,'--pack',PACK,'--ticks','0','--force','--save-out',src],1)
(out/'native-force-files.json').write_text(json.dumps(rows,indent=2))
# Independent SHA256 of transaction test artifacts (test implementation's SHA helper is not the oracle).
t=O/'transaction-independent';old=(t/'old.ohsave').read_bytes();new=(t/'new.ohsave').read_bytes();cases=json.loads((t/'cases.json').read_text());assert len(cases)==12;assert old!=new
for case in cases:assert case['cleanup'];assert case['calls']==list(range(min(case['fault'],4)+1));assert case['success']==(case['fault']>=4)
(t/'sha-evidence.json').write_text(json.dumps({'old_sha256':hashlib.sha256(old).hexdigest(),'new_sha256':hashlib.sha256(new).hexdigest(),'cases':len(cases),'public_old_sha':hashlib.sha256((O/'public-file/old.ohsave').read_bytes()).hexdigest(),'public_new_sha':hashlib.sha256((O/'public-file/new.ohsave').read_bytes()).hexdigest(),'public_file_sha':hashlib.sha256((O/'public-file/file.ohsave').read_bytes()).hexdigest()},indent=2))
assert (O/'public-file/file.ohsave').read_bytes()==(O/'public-file/new.ohsave').read_bytes();print(json.dumps({'stable_ids':'0/65535 nations and provinces, state0/901, null distinct, native container fields verified','nativeforce_count':len(rows),'transaction_cases':len(cases),'file_digest':hashlib.sha256(src.read_bytes()).hexdigest()}))
