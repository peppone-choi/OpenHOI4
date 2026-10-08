"""Required additive v5 evidence within the existing current-save artifact."""
import copy,hashlib,json,platform,re,tomllib
from pathlib import Path
from economy_reference import verify_report
from save_reference import pack_hash,fnv,varint
ROOT=Path(__file__).resolve().parents[3]
MODE='economy-native-v5'

def load(path):
    def unique(pairs):
        result={}
        for key,value in pairs:assert key not in result,'duplicate economy JSON field';result[key]=value
        return result
    return json.loads(Path(path).read_text(encoding='utf-8'),object_pairs_hook=unique)

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def header(path):
    raw=path.read_bytes();assert raw[:4]==b'OHSV' and int.from_bytes(raw[4:6],'little')==5
    length=int.from_bytes(raw[6:10],'little');assert 0<length<=tomllib.loads((ROOT/'crates/oh_save/defines.toml').read_text())['save']['header_max_bytes']
    data=raw[10:10+length];at=0
    def byte():
        nonlocal at
        assert at<len(data);v=data[at];at+=1;return v
    def uint(bits=64):
        start=at;v=0
        for shift in range(0,bits,7):
            b=byte();v|=(b&127)<<shift
            if b<128:assert v<1<<bits and data[start:at]==varint(v);return v
        raise AssertionError('invalid header varint')
    def text():
        nonlocal at
        n=uint();assert at+n<=len(data);v=data[at:at+n].decode();at+=n;return v
    engine=text();version=uint(16);scenario=text();count=uint();assert count==1
    packs=[dict(id=text(),version=text(),content_hash=uint()) for _ in range(count)]
    date=dict(year=uint(32),month=byte(),day=byte());tick=uint();seed=uint();h=uint();count=uint();assert count<=65536
    players=[uint(16) for _ in range(count)];saved=uint();saved=(saved>>1)^-(saved&1);tag=byte();assert tag in (0,1);definition=uint() if tag else None;defines=uint();assert at==len(data) and version==5
    return dict(engine_version=engine,format_version=version,scenario_id=scenario,packs=packs,game_date=date,tick=tick,seed=seed,state_hash=h,player_nations=players,saved_at_utc=saved,definitions_hash=definition,effective_defines_hash=defines)

def native(folder,parent,stage,index,executable,argv):
    receipt=load(folder/f'{stage}-{index}.command.json')
    assert type(receipt['exit']) is int and receipt['exit']==0, 'economy native exit'
    assert type(receipt['pid']) is int and receipt['pid']>0
    assert receipt['head']==parent['head'] and receipt['cwd']==parent['checkout_root']
    actual=[str(v).replace('\\','/') for v in receipt['command']]
    expected=[str(v).replace('\\','/') for v in argv]
    assert actual==expected, 'economy native command mismatch'
    assert receipt['exe_sha256']==parent[executable+'_sha256']
    (folder/f'{stage}-{index}.stderr').read_bytes()
    return receipt,load(folder/f'{stage}-{index}.stdout')

def capture(folder,boundary,run,files):
    folder.mkdir(exist_ok=False)
    run(['cargo','build','-p','oh_save','--example','economy_fixture','--locked'],folder,'build-economy')
    suffix='.exe' if platform.system()=='Windows' else ''
    helper=ROOT/f'target/debug/examples/economy_fixture{suffix}';cli=ROOT/f'target/debug/oh_cli{suffix}'
    result=dict(schema=1,mode=MODE,head=boundary['head'],dirty=bool(boundary['status']),checkout_root=str(ROOT),platform=platform.system(),evidence_root=str(folder),helper_path=str(helper),cli_path=str(cli),helper_sha256=sha(helper),cli_sha256=sha(cli),runs=[])
    for index in (1,2):
        out=folder/f'run-{index}';origin=out
        report=json.loads(run([str(helper),'capture',str(out)],folder,f'capture-{index}',True))
        source=files(out/'pack')
        run([str(helper),'resume',str(out/'pack'),str(origin/'saved.ohsave'),'48'],folder,f'restart-{index}',True)
        run([str(helper),'resume',str(out/'pack'),str(origin/'paused.ohsave'),'0'],folder,f'paused-{index}',True)
        run([str(cli),'resume','--load',str(origin/'saved.ohsave'),'--pack',str(out/'pack'),'--ticks','48','--hash-out','--save-out',str(origin/'cli-end.ohsave')],folder,f'cli-resume-{index}',True)
        run([str(helper),'resume',str(out/'pack'),str(origin/'cli-end.ohsave'),'0'],folder,f'cli-full-{index}',True)
        run([str(helper),'replay',str(out/'pack'),str(origin/'journal.zip')],folder,f'replay-{index}',True)
        run([str(cli),'repro','run',str(origin/'journal.zip'),'--pack',str(out/'pack')],folder,f'cli-replay-{index}',True)
        assert source==files(out/'pack'),'economy measurement input mutated'
        result['runs'].append(dict(pack=report['pack'],defines_hash=report['defines_hash'],source_files=source,split=report['split'],paused=report['paused'],continued=report['continued'],files={name:dict(bytes=(out/name).stat().st_size,sha256=sha(out/name)) for name in ('saved.ohsave','repeat.ohsave','paused.ohsave','journal.zip','cli-end.ohsave')}))
    (folder/'result.json').write_text(json.dumps(result,indent=2)+'\n')
    verify(folder,result['head'],result['dirty'],files)
    return result

def verify(folder,head,dirty,files):
    result=load(folder/'result.json');assert result['schema']==1 and result['mode']==MODE
    assert result['head']==head and type(result['dirty']) is bool and result['dirty']==dirty
    assert len(result['runs'])==2
    build=load(folder/'build-economy.command.json');assert type(build['exit']) is int and build['exit']==0 and build['head']==head
    assert build['command']==['cargo','build','-p','oh_save','--example','economy_fixture','--locked'];assert build['cwd']==result['checkout_root']
    (folder/'build-economy.stdout').read_bytes();(folder/'build-economy.stderr').read_bytes()
    pids=[];summaries=[]
    for index,record in enumerate(result['runs'],1):
        out=folder/f'run-{index}';origin=Path(result['evidence_root'])/f'run-{index}';helper=result['helper_path'];cli=result['cli_path'];pack=str(origin/'pack')
        receipt,report=native(folder,result,'capture',index,'helper',[helper,'capture',str(origin)]);pids.append(receipt['pid']);assert report['pid']==receipt['pid']
        assert load(out/'capture.json')==report
        assert record['pack']==report['pack'] and record['pack']['content_hash']==pack_hash(out/'pack')
        assert record['source_files']==files(out/'pack')
        for name in ('split','paused','continued'):assert record[name]==report[name];verify_report(report[name])
        assert report['split']['dto']['base']['base']['base']['state']['tick']==24
        assert report['continued']['dto']['base']['base']['base']['state']['tick']==72
        assert len(report['split']['dto']['queue'])==2 and not report['continued']['dto']['queue']
        assert (out/'saved.ohsave').read_bytes()==(out/'repeat.ohsave').read_bytes()
        for name,identity in record['files'].items():assert identity==dict(bytes=(out/name).stat().st_size,sha256=sha(out/name))
        assert set(record['files'])=={'saved.ohsave','repeat.ohsave','paused.ohsave','journal.zip','cli-end.ohsave'}
        for name,state in [('saved.ohsave','split'),('paused.ohsave','paused'),('cli-end.ohsave','continued')]:
            h=header(out/name);base=report[state]['dto']['base']['base']['base'];assert h['scenario_id']==base['state']['scenario']=='wp14_native'
            assert h['packs']==[record['pack']] and h['effective_defines_hash']==record['defines_hash']
            assert h['tick']==base['state']['tick'] and h['game_date']==base['state']['date'] and h['seed']==base['state']['seed']==1000
            assert f"{h['state_hash']:016x}"==report[state]['hash'] and h['definitions_hash']==base['world']['definitions_hash']
            if name!='cli-end.ohsave':assert h['saved_at_utc']==0 and h['player_nations']==[1]
        receipt,restart=native(folder,result,'restart',index,'helper',[helper,'resume',pack,str(origin/'saved.ohsave'),'48']);pids.append(receipt['pid']);assert restart['pid']==receipt['pid'] and restart['initial']==report['split'] and restart['resumed']==report['continued'];verify_report(restart['initial']);verify_report(restart['resumed'])
        receipt,paused=native(folder,result,'paused',index,'helper',[helper,'resume',pack,str(origin/'paused.ohsave'),'0']);pids.append(receipt['pid']);assert paused['pid']==receipt['pid'] and paused['initial']==paused['resumed']==report['paused'];verify_report(paused['resumed'])
        receipt=load(folder/f'cli-resume-{index}.command.json');assert type(receipt['exit']) is int and receipt['exit']==0 and receipt['head']==head and receipt['exe_sha256']==result['cli_sha256'];pids.append(receipt['pid']);assert [str(v).replace('\\','/') for v in receipt['command']]==[str(v).replace('\\','/') for v in [cli,'resume','--load',str(origin/'saved.ohsave'),'--pack',pack,'--ticks','48','--hash-out','--save-out',str(origin/'cli-end.ohsave')]]
        assert (folder/f'cli-resume-{index}.stdout').read_text().strip()==report['continued']['hash'];(folder/f'cli-resume-{index}.stderr').read_bytes()
        receipt,cli_full=native(folder,result,'cli-full',index,'helper',[helper,'resume',pack,str(origin/'cli-end.ohsave'),'0']);pids.append(receipt['pid']);assert cli_full['pid']==receipt['pid'] and cli_full['initial']==cli_full['resumed']==report['continued'];verify_report(cli_full['resumed'])
        receipt,replay=native(folder,result,'replay',index,'helper',[helper,'replay',pack,str(origin/'journal.zip')]);pids.append(receipt['pid']);assert replay['pid']==receipt['pid'] and replay['result']==report['continued'];verify_report(replay['result'])
        receipt,cli_replay=native(folder,result,'cli-replay',index,'cli',[cli,'repro','run',str(origin/'journal.zip'),'--pack',pack]);pids.append(receipt['pid']);assert cli_replay==report['continued'];verify_report(cli_replay)
        normalized=copy.deepcopy(record);normalized['files'].pop('cli-end.ohsave');summaries.append(normalized)
    assert len(set(pids))==14,'fourteen actual distinct native v5 processes required'
    assert summaries[0]==summaries[1],'repeated v5 bytes and full state differ'
    return dict(head=head,mode=MODE,runs=summaries)

def verify_server(directory,parent,force,files):
    from economy_wire_reference import verify_query
    receipt=load(directory/'result.json');record=parent['runs'][0]
    assert receipt['head']==receipt['capture_head']==parent['head'] and receipt['capture_mode']==MODE
    assert type(receipt['dirty']) is bool and receipt['dirty']==parent['dirty'] and receipt['force'] is force
    assert receipt['http_status']==200 and type(receipt['server_exit']) is int and receipt['server_exit']==0 and type(receipt['query_exit']) is int and receipt['query_exit']==0
    assert receipt['server_cwd']==parent['checkout_root']
    assert receipt['source_preserved'] is True and receipt['save_preserved'] is True
    assert receipt['input_files']==record['source_files']==files(directory/'packs/testland')
    assert receipt['save_sha256']==record['files']['paused.ohsave']['sha256']
    assert receipt['input_header']==header(directory.parent.parent/'economy-v5/run-1/paused.ohsave')
    assert receipt['pack_hash']==f"{record['pack']['content_hash']:016x}"
    for key in ('exe_sha256','served_js_sha256','built_js_sha256'):assert re.fullmatch('[0-9a-f]{64}',receipt[key])
    assert receipt['built_js_sha256']==receipt['served_js_sha256']
    assert type(receipt['server_pid']) is int and receipt['server_pid']>0
    argv=[str(v).replace('\\','/') for v in receipt['server_command']]
    assert argv[0].split('/')[-1].removesuffix('.exe')=='oh_server' and argv[1]=='--port' and argv[3]=='--pack-root' and argv[5:8]==['--scenario','wp14_native','--load-save']
    assert argv[4].endswith('/economy-v5/packs') and argv[8].endswith('/economy-v5/run-1/paused.ohsave') and argv[9:]==(['--force'] if force else [])
    assert receipt['query_command']==['node','crates/oh_server/tests/economy_query.cjs',receipt['url']]
    for file in ('server.stdout','server.stderr','query.stderr'):(directory/file).read_bytes()
    startup=(directory/'server.stdout').read_text();assert receipt['url'] in startup and receipt['pack_hash'] in startup
    query=load(directory/'query.stdout');assert query==receipt['query'] and query['pid']!=receipt['server_pid']
    definitions=tomllib.loads((directory/'packs/testland/common/economy/wp14_native.toml').read_text())
    verify_query(query,record,definitions)
    return receipt
