"""Actual fresh native Host I/O. Generated fixtures are explicitly synthetic."""
import copy, hashlib, io, json, os, shutil, struct, subprocess, tempfile, tomllib, unittest, warnings, zipfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
BIN = ROOT / ('target/debug/oh_cli.exe' if os.name == 'nt' else 'target/debug/oh_cli')
RUNS = ROOT / 'target/evidence/WP-25-M2-r2'
RUNS.mkdir(parents=True, exist_ok=True)
EVIDENCE = Path(tempfile.mkdtemp(prefix='native-', dir=RUNS))
NUMBER = 0

def snapshot(source,target):
    try: shutil.copyfile(source,target)
    except PermissionError as error:
        # A deliberate native exclusive-handle test prevents inspection until release.
        target.with_suffix(target.suffix+'.capture-error.json').write_text(json.dumps({'path':str(source),'error':str(error)}),encoding='utf-8')

def native(args, inputs='', expected=0):
    global NUMBER
    NUMBER += 1
    label = f'{NUMBER:04d}'
    for flag,suffix in [('--load','input.ohsave'),('--out','target-before.bin')]:
        if flag in args:
            source=Path(args[args.index(flag)+1])
            if source.is_file(): snapshot(source,EVIDENCE/(label+'.'+suffix))
    if len(args)>2 and args[:2]==['repro','run'] and Path(args[2]).is_file():
        snapshot(args[2],EVIDENCE/(label+'.input.zip'))
    p = subprocess.run([str(BIN), *map(str,args)], input=inputs.encode(), capture_output=True, cwd=ROOT, timeout=120)
    if '--out' in args:
        target=Path(args[args.index('--out')+1])
        if target.is_file(): snapshot(target,EVIDENCE/(label+'.target-after.bin'))
    (EVIDENCE/(label+'.stdout')).write_bytes(p.stdout)
    (EVIDENCE/(label+'.stderr')).write_bytes(p.stderr)
    (EVIDENCE/(label+'.json')).write_text(json.dumps({'command':[str(BIN), *map(str,args)], 'cwd':str(ROOT), 'native_exit':p.returncode, 'stdin':inputs}), encoding='utf-8')
    if p.returncode != expected: raise AssertionError(f'{args}: expected {expected}, actual {p.returncode}: {p.stderr.decode(errors="replace")}')
    return p


def example_process(command, path):
    p=subprocess.run(command,capture_output=True,cwd=ROOT,timeout=120)
    (path/'producer.stdout').write_bytes(p.stdout)
    (path/'producer.stderr').write_bytes(p.stderr)
    (path/'producer.json').write_text(json.dumps({'command':list(map(str,command)),'cwd':str(ROOT),'native_exit':p.returncode}),encoding='utf-8')
    return p

def stream(commands): return ''.join(json.dumps(x)+'\n' for x in commands)
def enqueue(tick,seq,command,nation=0): return dict(op='enqueue',tick=tick,nation=nation,sequence=seq,command=command)
def members(path):
    with zipfile.ZipFile(path) as z: return {i.filename:z.read(i) for i in z.infolist()}
def archive(path, items, compression=zipfile.ZIP_STORED):
    with warnings.catch_warnings():
        warnings.simplefilter('ignore')
        with zipfile.ZipFile(path,'w',compression=compression) as z:
            for name,data in items: z.writestr(name,data)
def fingerprint(path):
    if path.is_file(): return hashlib.sha256(path.read_bytes()).hexdigest()
    return [(str(p.relative_to(path)),hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted(path.rglob('*')) if p.is_file()]

class Native(unittest.TestCase):
    def setUp(self):
        self.path=Path(tempfile.mkdtemp(prefix='case-',dir=EVIDENCE))
    def tearDown(self): pass  # retain original input/recorded ZIPs and synthetic fixture bytes
    def record(self,commands,extra=()):
        out=self.path/'live.zip'
        p=native(['repro','record','--out',out,*extra],stream(commands))
        state=json.loads(p.stdout.splitlines()[-1])
        replay=native(['repro','run',out,*(['--pack',extra[extra.index('--pack')+1]] if '--pack' in extra else [])])
        self.assertEqual(state,json.loads(replay.stdout))
        return out,state,json.loads(members(out)['commands.log'])
    def test_req_sav_05_live_stdin_record_and_fresh_replay(self):
        commands=[enqueue(0,1,{'Pause':True}),dict(op='pump'),enqueue(0,2,{'SetSpeed':5}),dict(op='pump'),enqueue(0,3,{'Pause':False}),dict(op='step'),dict(op='step')]
        out,state,events=self.record(commands)
        self.assertEqual(state['dto']['state']['tick'],2)
        self.assertEqual([e['before_tick'] for e in events],[0,0,0,0,0,0,1])
        self.assertEqual([events[i]['outcome']['advanced'] for i in (1,3,5)],[False,False,True])
        # Independent standard-library ZIP reader consumes production bytes.
        self.assertEqual(set(members(out)),{'bundle.toml','commands.log','commands.txt','final.json','start.ohsave'})
    def test_live_host_flushes_before_eof(self):
        out=self.path/'interactive.zip'
        p=subprocess.Popen([str(BIN),'repro','record','--out',str(out)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=ROOT)
        try:
            p.stdin.write(b'{"op":"step"}\n'); p.stdin.flush()
            import queue, threading
            response=queue.Queue()
            threading.Thread(target=lambda:response.put(p.stdout.readline()),daemon=True).start()
            event=json.loads(response.get(timeout=120))
            self.assertTrue(event['outcome']['advanced'])
            self.assertFalse(out.exists())
            p.stdin.close(); p.stdin=None
            stdout,stderr=p.communicate(timeout=120)
            self.assertEqual(p.returncode,0,stderr)
            (self.path/'interactive.stdout').write_bytes(json.dumps(event).encode()+b'\n'+stdout)
            (self.path/'interactive.stderr').write_bytes(stderr)
            (self.path/'interactive.json').write_text(json.dumps({'command':[str(BIN),'repro','record','--out',str(out)],'cwd':str(ROOT),'native_exit':p.returncode,'stdin':'{\"op\":\"step\"}\n'}),encoding='utf-8')
            self.assertEqual(json.loads(stdout),json.loads(native(['repro','run',out]).stdout))
        finally:
            if p.poll() is None: p.kill(); p.wait()
    def test_semantic_error_consumed_duplicate_and_future_preserved(self):
        out,state,events=self.record([enqueue(0,1,{'SetSpeed':0}),enqueue(0,1,{'SetSpeed':5}),enqueue(99,10,{'Pause':True}),dict(op='step'),enqueue(0,3,{'Pause':True})])
        self.assertEqual(events[1]['outcome']['error'],'DuplicateCommand')
        self.assertEqual(events[3]['outcome']['commands'][0]['error'],'InvalidSpeed')
        self.assertEqual(events[4]['outcome']['error'],'PastCommand')
        self.assertEqual(len(state['dto']['queue']),1)
        self.assertEqual(state['dto']['queue'][0]['tick'],99)
        # Existing pending queue appears once in start save, never new input log.
        save=self.path/'start.ohsave'; save.write_bytes(members(out)['start.ohsave'])
        final=self.path/'end.ohsave'
        native(['run','--pack','data/packs/testland','--scenario','m1','--ticks','3','--seed','1','--save-out',final])
        self.record([enqueue(3,9,{'Pause':True}),dict(op='pump')],['--load',final,'--pack','data/packs/testland'])
    def test_start_snapshot_pending_not_double_enqueued(self):
        out,state,_=self.record([enqueue(99,7,{'Pause':True}),dict(op='step')])
        # Start recording again from full end state using normal CLI save from a native producer.
        fixture=self.path/'movement'; fixture.mkdir()
        example=ROOT/('target/debug/examples/movement_fixture.exe' if os.name=='nt' else 'target/debug/examples/movement_fixture')
        p=example_process([str(example),'capture',str(fixture)],fixture)
        self.assertEqual(p.returncode,0,p.stderr)
        start=fixture/'movement-v2.ohsave'; before=fingerprint(start)
        _,state,events=self.record([dict(op='step')]*23,['--load',start,'--pack','data/packs/testland'])
        actual=json.loads(p.stdout)['continuous']
        for key in ('dto','hash','canonical_hex'): self.assertEqual(state[key],actual[key])
        self.assertEqual(before,fingerprint(start))
        self.assertTrue(any(e['outcome']['commands'] for e in events))
    def test_v3_actual_strait_progress_arrival_and_stop_queue_replay(self):
        fixture=self.path/'strait'; fixture.mkdir()
        example=ROOT/('target/debug/examples/strait_fixture.exe' if os.name=='nt' else 'target/debug/examples/strait_fixture')
        p=example_process([str(example),'capture',str(fixture)],fixture)
        self.assertEqual(p.returncode,0,p.stderr)
        start=fixture/'strait-v3.ohsave'; original=fingerprint(start)
        _,state,_=self.record([dict(op='step')]*23,['--load',start,'--pack','data/packs/testland'])
        self.assertEqual(state['format'],3)
        for key in ('dto','hash','canonical_hex'): self.assertEqual(state[key],json.loads(p.stdout)['continuous'][key])
        self.assertEqual(original,fingerprint(start))
    def test_whole_phase_clock_error_preserves_queue_clock_state(self):
        start=self.path/'clock.ohsave'
        example=ROOT/('target/debug/examples/repro_clock.exe' if os.name=='nt' else 'target/debug/examples/repro_clock')
        p=example_process([str(example),str(start)],self.path)
        self.assertEqual(p.returncode,0,p.stderr)
        initial=json.loads(p.stdout); tick=initial['dto']['state']['tick']
        commands=[enqueue(tick,1,{'SetSpeed':2}),dict(op='step'),enqueue(tick,2,{'Pause':True}),dict(op='pump')]
        _,state,events=self.record(commands,['--load',start,'--pack','data/packs/testland'])
        self.assertEqual(events[1]['outcome']['error'],'ClockOverflow')
        self.assertEqual(events[1]['after_hash'],events[0]['after_hash'])
        self.assertEqual(events[3]['before_tick'],tick)
        self.assertEqual([c['sequence'] for c in events[3]['outcome']['commands']],[1,2])
        self.assertEqual(state['dto']['state']['hour'],23)
        self.assertTrue(state['dto']['state']['paused'])
    def test_v4_flags_checkpoint_ended_refusal_future_queue(self):
        fixture=self.path/'trigger'; fixture.mkdir()
        example=ROOT/('target/debug/examples/trigger_fixture.exe' if os.name=='nt' else 'target/debug/examples/trigger_fixture')
        p=example_process([str(example),'capture',str(fixture)],fixture)
        self.assertEqual(p.returncode,0,p.stderr)
        expected=json.loads(p.stdout)['cases']
        for case,pack,count in [('active','pack',48),('ended','pack',0),('empty','pack-empty',3),('initial','pack-initial',0),('paused_condition','pack-paused_condition',0)]:
            with self.subTest(case=case):
                start=fixture/(case+'.ohsave'); before=fingerprint(start)
                commands=[dict(op='step')]*count
                if case in ('active','ended','initial'): commands += [dict(op='pump'),enqueue(48,100,{'Pause':True})]
                _,state,events=self.record(commands,['--load',start,'--pack',fixture/pack])
                if case=='active':
                    for key in ('dto','hash','canonical_hex'): self.assertEqual(state[key],expected['ended'][key])
                    self.assertEqual(len(state['dto']['queue']),5)
                if case in ('active','ended','initial'):
                    self.assertEqual(events[-1]['outcome']['error'],'ScenarioEnded')
                    self.assertEqual(events[-2]['outcome']['error'],'ScenarioEnded')
                self.assertEqual(before,fingerprint(start))
    def test_zip_hostile_names_duplicates_symlink_and_unknown(self):
        good,_,_=self.record([dict(op='step')]); original=members(good)
        for name in ['/bundle.toml','C:/bundle.toml','C:bundle.toml','//host/bundle.toml','../bundle.toml','a/../bundle.toml','a\\..\\bundle.toml','bundle.toml\\','BUNDLE.TOML','./bundle.toml','bundle.toml\x00x','extra.bin']:
            with self.subTest(name=name):
                path=self.path/'bad.zip'
                # zipfile truncates NUL names; mutate bytes instead for that case.
                items=list(original.items()); items[0]=(name,items[0][1]); archive(path,items)
                if '\x00' in name:
                    archive(path,list(original.items()))
                    path.write_bytes(path.read_bytes().replace(b'bundle.toml',b'bundle\x00toml'))
                native(['repro','run',path],expected=1)
        archive(self.path/'dup.zip',list(original.items())+[('bundle.toml',original['bundle.toml'])]); native(['repro','run',self.path/'dup.zip'],expected=1)
        for attr in [0o120777<<16,0x400,0x10]:
            path=self.path/'link.zip'
            with zipfile.ZipFile(path,'w') as z:
                for name,data in original.items():
                    item=zipfile.ZipInfo(name); item.external_attr=attr; item.create_system=3
                    z.writestr(item,data)
            native(['repro','run',path],expected=1)
        self.assertFalse((self.path/'bundle.toml').exists())
    def test_zip_truncation_crc_lengths_offsets_bomb_and_methods(self):
        good,_,_=self.record([dict(op='step')]); original=good.read_bytes()
        variants=[original[:-1],original[:-22],original+b'x',b'PK',original[:40]]
        for offset in [14,18,22,26,28]:
            b=bytearray(original); b[offset]^=1; variants.append(b)
        central=original.index(b'PK\x01\x02')
        for offset in [8,10,16,20,24,28,30,32,34,38,42]:
            b=bytearray(original); b[central+offset]^=1; variants.append(b)
        for offset in [8,10,12,16,20]:
            b=bytearray(original); b[-22+offset]^=1; variants.append(b)
        b=bytearray(original); struct.pack_into('<I',b,central+24,0xffffffff); variants.append(b)
        b=bytearray(original); b[35]^=1; variants.append(b)
        for i,b in enumerate(variants):
            with self.subTest(i=i):
                path=self.path/'bad.zip'; path.write_bytes(b); native(['repro','run',path],expected=1)
        archive(self.path/'deflated.zip',list(members(good).items()),zipfile.ZIP_DEFLATED)
        native(['repro','run',self.path/'deflated.zip'],expected=1)
        bomb=self.path/'bomb.zip'; archive(bomb,[('commands.log',b'0'*1000000)],zipfile.ZIP_DEFLATED)
        native(['repro','run',bomb],expected=1)
    def test_schema_hash_seed_context_log_full_state_tamper_and_input_preservation(self):
        good,_,_=self.record([dict(op='step')]); original=members(good)
        pack=ROOT/'data/packs/testland'; beforepack=fingerprint(pack)
        for field,value in [('schema',2),('engine_version','future'),('scenario','unknown'),('national',False),('pack_id','wrong'),('pack_version','99.0.0'),('pack_hash','0000000000000000'),('effective_defines_hash','0000000000000000'),('seed',2),('start_save',False),('start_tick',1),('start_hash','0000000000000000'),('end_tick',2),('expected_hash','0000000000000000'),('events',100001)]:
            with self.subTest(field=field):
                m=copy.deepcopy(original); text=m['bundle.toml'].decode(); old=tomllib.loads(text)[field]
                def literal(v): return str(v).lower() if type(v) is bool else json.dumps(v)
                m['bundle.toml']=text.replace(f'{field} = {literal(old)}',f'{field} = {literal(value)}').encode()
                path=self.path/'bad.zip'; archive(path,m.items()); native(['repro','run',path],expected=1)
        variants=[]
        for name in original:
            m=copy.deepcopy(original); del m[name]; variants.append(m)
        m=copy.deepcopy(original); m['bundle.toml']+=b'unknown = 1\n'; variants.append(m)
        m=copy.deepcopy(original); m['bundle.toml']=m['bundle.toml'].replace(b'schema = 1\n',b''); variants.append(m)
        for mutate in [lambda es:es[0].update(index=1), lambda es:es.append(es[0]),lambda es:es[0].update(before_tick=1),lambda es:es[0]['input'].update(op='unknown'),lambda es:es[0].update(unknown=1),lambda es:es[0]['outcome'].update(advanced=False),lambda es:es[0].update(after_hash='0000000000000000')]:
            m=copy.deepcopy(original); es=json.loads(m['commands.log']); mutate(es); m['commands.log']=json.dumps(es).encode(); variants.append(m)
        m=copy.deepcopy(original); m['commands.txt']+=b'forged\n'; variants.append(m)
        m=copy.deepcopy(original); f=json.loads(m['final.json']); f['dto']['state']['speed']=5; m['final.json']=json.dumps(f).encode(); variants.append(m)
        m=copy.deepcopy(original); f=json.loads(m['final.json']); f['canonical_hex']='00'; m['final.json']=json.dumps(f).encode(); variants.append(m)
        m=copy.deepcopy(original); m['start.ohsave']=b'bad'; variants.append(m)
        for i,m in enumerate(variants):
            with self.subTest(i=i):
                path=self.path/'bad.zip'; archive(path,m.items()); before=fingerprint(path)
                native(['repro','run',path],expected=1); self.assertEqual(before,fingerprint(path))
        self.assertEqual(beforepack,fingerprint(pack))
        self.assertEqual(original,members(good))
    def test_failures_preserve_existing_target_input_save_and_native_commit_error(self):
        good,_,_=self.record([dict(op='step')]); save=self.path/'input.ohsave'; save.write_bytes(members(good)['start.ohsave'])
        target=self.path/'existing.zip'; target.write_bytes(b'original target')
        original=fingerprint(save)
        for text in ['{bad}\n',json.dumps({'op':'step','unknown':1})+'\n','x'*16385+'\n',json.dumps(enqueue(0,1,{'Unknown':1}))+'\n']:
            native(['repro','record','--out',target,'--load',save,'--pack','data/packs/testland'],text,1)
            self.assertEqual(target.read_bytes(),b'original target'); self.assertEqual(fingerprint(save),original)
        native(['repro','record','--out',save,'--load',save,'--pack','data/packs/testland'],stream([dict(op='step')]),1)
        self.assertEqual(fingerprint(save),original)
        native(['repro','record','--out',ROOT/'data/packs/testland/manifest.toml'],expected=1)
        directory=self.path/'directory.zip'; directory.mkdir(); (directory/'keep').write_bytes(b'keep')
        native(['repro','record','--out',directory],stream([dict(op='step')]),1)
        self.assertEqual((directory/'keep').read_bytes(),b'keep')
        self.assertFalse(list(self.path.glob('.tmp*')))
    def test_atomic_replace_failure_after_real_temp_write_preserves_target(self):
        target=self.path/'locked.zip'; target.write_bytes(b'original target')
        (self.path/'locked-before.bin').write_bytes(target.read_bytes())
        if os.name=='nt':
            import ctypes
            from ctypes import wintypes
            kernel=ctypes.WinDLL('kernel32',use_last_error=True)
            kernel.CreateFileW.argtypes=[wintypes.LPCWSTR,wintypes.DWORD,wintypes.DWORD,wintypes.LPVOID,wintypes.DWORD,wintypes.DWORD,wintypes.HANDLE]
            kernel.CreateFileW.restype=wintypes.HANDLE
            kernel.CloseHandle.argtypes=[wintypes.HANDLE]
            handle=kernel.CreateFileW(str(target),0x80000000,0,None,3,0x80,None)
            self.assertNotEqual(handle,ctypes.c_void_p(-1).value)
            try:
                result=native(['repro','record','--out',target],stream([dict(op='step')]),1)
                self.assertIn(b'failed to persist temporary file',result.stderr.lower())
            finally: kernel.CloseHandle(handle)
        else:
            # Nonroot Linux runner: temp-file creation fails on immutable directory.
            self.path.chmod(0o555)
            try: native(['repro','record','--out',target],stream([dict(op='step')]),1)
            finally: self.path.chmod(0o755)
        self.assertEqual(target.read_bytes(),b'original target')
        (self.path/'locked-after.bin').write_bytes(target.read_bytes())
        self.assertFalse(list(self.path.glob('.tmp*')))
    def test_req_perf_03_native_bench_completed_state_and_zero_rejection(self):
        p=native(['bench','--pack','data/packs/testland','--scenario','m1','--seed','1000','--steps','24'])
        result=json.loads(p.stdout)
        self.assertGreater(result['elapsed_ns'],0)
        self.assertEqual(result['state']['dto']['state']['tick'],24)
        self.assertEqual(result['steps'],24)
        native(['bench','--pack','data/packs/testland','--scenario','m1','--seed','1000','--steps','0'],expected=1)
    def test_native_reparse_or_symlink_input_and_output_alias_refused(self):
        source=self.path/'real'; source.mkdir()
        good,_,_=self.record([dict(op='step')])
        shutil.copyfile(good,source/'good.zip')
        alias=self.path/'alias'
        if os.name=='nt':
            p=subprocess.run(['powershell.exe','-NoProfile','-Command','New-Item','-ItemType','Junction','-Path',str(alias),'-Target',str(source)],capture_output=True,cwd=ROOT,timeout=120)
            (self.path/'junction.stdout').write_bytes(p.stdout)
            (self.path/'junction.stderr').write_bytes(p.stderr)
            self.assertEqual(p.returncode,0,p.stderr)
        else: alias.symlink_to(source,target_is_directory=True)
        native(['repro','run',alias/'good.zip'],expected=1)
        target=source/'existing.zip'; target.write_bytes(b'preserve')
        native(['repro','record','--out',alias/'existing.zip'],stream([dict(op='step')]),1)
        self.assertEqual(target.read_bytes(),b'preserve')
    def test_caller_cannot_select_historical_purpose_force_or_seed_override(self):
        native(['repro','record','--out',self.path/'x.zip','--force'],expected=1)
        native(['repro','record','--out',self.path/'x.zip','--purpose','cli_v1_resume'],expected=1)
        native(['repro','record','--out',self.path/'x.zip','--load',ROOT/'crates/oh_save/tests/fixtures/m1-v1.ohsave','--pack','data/packs/testland'],expected=1)
        native(['repro','run','missing.zip','--seed','7'],expected=1)

if __name__ == '__main__': unittest.main(verbosity=2)
