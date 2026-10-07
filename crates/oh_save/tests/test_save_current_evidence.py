"""Current evidence predicates; synthetic runner copies do not prove 3OS execution."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
import tomllib
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
ROOT = TOOLS.parents[2]
sys.path.insert(0, str(TOOLS))


class PipelineInputs(unittest.TestCase):
    def test_required_server_positive_consumes_separate_current_capture(self):
        workflow = (ROOT / '.github/workflows/save-determinism.yml').read_text()
        self.assertIn('check_save_current.py capture --out target/save-current', workflow)
        self.assertIn('save_native.py --capture target/save-current', workflow)
        self.assertNotIn('save_native.py --capture target/save-determinism', workflow)
        self.assertIn('check_save_determinism.py capture --out target/save-determinism', workflow)
        self.assertIn('check_save_determinism.py compare --root save-evidence', workflow)
        self.assertIn('check_save_current.py compare --root save-current-evidence', workflow)
        self.assertIn('save_historical_native.py --capture target/save-determinism', workflow)


class CurrentEvidenceFailures(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        path = TOOLS / 'check_save_current.py'
        if not path.is_file():
            raise AssertionError('required separate current capture/comparer is missing')
        spec = importlib.util.spec_from_file_location('current_save_gate', path)
        cls.gate = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.gate)
        cls.directory = tempfile.TemporaryDirectory(dir=ROOT / 'target')
        cls.source = Path(cls.directory.name) / 'actual-current-local'
        with contextlib.redirect_stdout(io.StringIO()):
            cls.gate.capture(cls.source)

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def artifacts(self, include=None):
        directory = tempfile.TemporaryDirectory(dir=ROOT / 'target')
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        for runner in self.gate.RUNNERS if include is None else include:
            folder = root / f'current-save-{runner}'
            shutil.copytree(self.source, folder)
            record = json.loads((folder / 'result.json').read_text())
            record['platform'] = self.gate.PLATFORMS[runner]
            record['dirty'] = False  # Synthetic predicate control, never actual OS evidence.
            for boundary in ('before', 'after'):
                record['source'][boundary]['status'] = ''
                (folder / f'source-{boundary}.json').write_text(json.dumps(record['source'][boundary]))
            (folder / 'result.json').write_text(json.dumps(record))
            economy_path=folder/'economy-v5/result.json'
            economy=json.loads(economy_path.read_text());economy['platform']=self.gate.PLATFORMS[runner];economy['dirty']=False
            economy_path.write_text(json.dumps(economy))
            # Explicit synthetic wire/native receipts for comparer predicates only.
            for force, name in ((False, 'server'), (True, 'server-force')):
                server = folder / name; server.mkdir()
                run = record['runs'][0]
                shutil.copytree(folder / 'run-1/pack', server / 'packs/testland')
                pack_hash = f'{run["pack"]["content_hash"]:016x}'
                query = dict(pid=9003, welcome=dict(accepted=True, engine_version=run['paused_header']['engine_version'],
                    packs=[dict(id=run['pack']['id'], version=run['pack']['version'], hash=pack_hash)]),
                    snapshot=dict(state=dict(date='2000-03-01', hour=0, tick='48', paused=True, speed=5)),
                    query=dict(supported=True, world=dict(tick='48', nations=[{}, {}], provinces=[{}]*6)),
                    command=dict(accepted=True), state=dict(state=dict(tick='48', paused=True, speed=2)))
                query['welcome']['type']='Welcome'
                query['snapshot']['type']='Snapshot'
                query['state']=dict(type='Snapshot',state=dict(date='2000-03-01',hour=0,tick='48',paused=True,speed=2))
                query['command'].update(type='CommandResult',sequence='1')
                query['query'].update(type='WorldResult',request='saved-world')
                world=query['query']['world']
                world['nations']=[dict(id=1,tag='NTH',government_key='government-test-republic',
                    support=[dict(name_key='ideology-test-civic',value='0.75'),dict(name_key='ideology-test-local',value='0.25')]),
                    dict(id=2,tag='STH',government_key='government-test-council',
                    support=[dict(name_key='ideology-test-civic',value='0.75'),dict(name_key='ideology-test-local',value='0.25')])]
                world['provinces']=[dict(id=10,state=1,owner=1,controller=1),dict(id=20,state=1,owner=1,controller=2),
                    dict(id=30,state=2,owner=2,controller=2),dict(id=40,state=2,owner=2,controller=2),
                    dict(id=50,state=None,owner=None,controller=None),dict(id=60,state=None,owner=None,controller=None)]
                def row(i,source,op,value,accumulated):
                    return dict(id=str(i),source_key=source,label_key=source or 'infrastructure',
                        operation_key=op,value=value,accumulated=accumulated)
                world['states']=[dict(id=1,owner=1,population='1000',resources=[dict(name_key='resource-steel',value='2')],
                    buildings=[dict(name_key='building-industry',value='1')],infrastructure=dict(base='1',tick='48',final_value='3.0000000005',
                    entries=[row(0,None,'ledger-base','1','1'),row(1,'a.raw','ledger-add','0.0000000002','1.0000000002'),
                             row(2,'b.add','ledger-add','0.5','1.5000000002'),row(3,'c.mul','ledger-multiply','2','3.0000000005')])),
                    dict(id=2,owner=2,population='2000',resources=[dict(name_key='resource-steel',value='0')],
                    buildings=[dict(name_key='building-industry',value='0')],infrastructure=dict(base='0',tick='48',final_value='0',
                    entries=[row(0,None,'ledger-base','0','0')]))]
                query['messages']=[query['welcome'],dict(type='Notice',key='unsupported-create'),query['snapshot'],
                                   query['query'],query['command'],query['state']]
                receipt = dict(head=record['head'], capture_head=record['head'], capture_mode=self.gate.MODE,
                    dirty=False, force=force, server_cwd=record['checkout_root'], http_status=200,
                    server_exit=0, query_exit=0, save_preserved=True, source_preserved=True,
                    save_sha256=run['paused_sha256'], input_header=run['paused_header'], input_files=run['source_files'],
                    pack_hash=pack_hash, exe_sha256='1'*64, served_js_sha256='2'*64, built_js_sha256='2'*64,
                    server_pid=9001+int(force), url='http://127.0.0.1:34567/',
                    server_command=['oh_server','--port','34567','--pack-root',f'/{name}/packs','--load-save','/run-1/paused.ohsave']+(['--force'] if force else []),
                    query_command=['node','crates/oh_server/tests/save_query.cjs','http://127.0.0.1:34567/'], query=query)
                (server / 'result.json').write_text(json.dumps(receipt))
                (server / 'query.stdout').write_text(json.dumps(query))
                for file in ('server.stdout','server.stderr','query.stderr'):
                    (server / file).write_text('')
                (server / 'server.stdout').write_text(receipt['url']+' '+pack_hash)
                # Explicit synthetic economic wire controls, not actual OS server evidence.
                from economy_wire_reference import expected_view,filled
                eco=server/'economy-v5';eco.mkdir();economic_run=economy['runs'][0]
                shutil.copytree(folder/'economy-v5/run-1/pack',eco/'packs/testland')
                definitions=tomllib.loads((eco/'packs/testland/common/economy/wp14_native.toml').read_text())
                view=filled(expected_view(economic_run['paused'],definitions));state=economic_run['paused']['dto']['base']['base']['base']['state']
                economic_pack=f"{economic_run['pack']['content_hash']:016x}"
                economic_query=dict(pid=9010+int(force),welcome=dict(type='Welcome',accepted=True,engine_version=run['paused_header']['engine_version'],packs=[dict(id=economic_run['pack']['id'],version=economic_run['pack']['version'],hash=economic_pack)]),snapshot=dict(type='Snapshot',state=dict(date=f"{state['date']['year']:04}-{state['date']['month']:02}-{state['date']['day']:02}",tick=str(state['tick']),hour=state['hour'],paused=True,speed=state['speed'])),before=dict(type='EconomyResult',request='before',supported=True,reason_key=None,economy=view),after=dict(type='EconomyResult',request='after',supported=True,reason_key=None,economy=view),accepted=dict(type='CommandResult',sequence='1',accepted=True,reason_key=None),rejected=[dict(type='CommandResult',sequence=str(i),accepted=False,reason_key='invalid-message') for i in (2,3,4)])
                economic_query['messages']=[economic_query['welcome'],economic_query['snapshot'],economic_query['before'],economic_query['accepted'],*economic_query['rejected'],economic_query['after']]
                economic_receipt=dict(head=record['head'],capture_head=record['head'],capture_mode='economy-native-v5',dirty=False,force=force,server_pid=9012+int(force),server_cwd=record['checkout_root'],server_command=['oh_server','--port','34567','--pack-root',f'/{name}/economy-v5/packs','--scenario','wp14_native','--load-save','/economy-v5/run-1/paused.ohsave']+(['--force'] if force else []),url=receipt['url'],exe_sha256='1'*64,served_js_sha256='2'*64,built_js_sha256='2'*64,http_status=200,server_exit=0,query_exit=0,query_command=['node','crates/oh_server/tests/economy_query.cjs',receipt['url']],query=economic_query,save_sha256=economic_run['files']['paused.ohsave']['sha256'],input_header=self.gate.economy_current.header(folder/'economy-v5/run-1/paused.ohsave'),input_files=economic_run['source_files'],pack_hash=economic_pack,source_preserved=True,save_preserved=True)
                (eco/'result.json').write_text(json.dumps(economic_receipt));(eco/'query.stdout').write_text(json.dumps(economic_query));(eco/'query.stderr').write_text('');(eco/'server.stdout').write_text(receipt['url']+' '+economic_pack);(eco/'server.stderr').write_text('')

        return root

    def mutate(self, root, file, change):
        path = root / 'current-save-windows-latest' / file
        data = json.loads(path.read_text())
        change(data)
        path.write_text(json.dumps(data))

    def fails(self, root):
        with self.assertRaises((ValueError, AssertionError, OSError, KeyError, TypeError)):
            self.gate.compare(root)

    def test_synthetic_control(self):
        with contextlib.redirect_stdout(io.StringIO()):
            self.gate.compare(self.artifacts())

    def test_missing_os(self):
        self.fails(self.artifacts(self.gate.RUNNERS[:2]))

    def test_different_head(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(head='0' * 40))
        self.fails(root)

    def test_dirty_source(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(dirty=True))
        self.fails(root)

    def test_wrong_current_mode(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(evidence_mode='original-frozen-v1'))
        self.fails(root)

    def test_wrong_format(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(evidence_schema=2))
        self.fails(root)

    def test_query_and_receipt_simultaneous_ledger_tick_tamper(self):
        root = self.artifacts()
        directory = root / 'current-save-windows-latest/server'
        raw = json.loads((directory / 'query.stdout').read_text())
        # Exact independent P05-3 mutant: both serialized copies agree on a wrong tick.
        raw['query']['world']['states'][0]['infrastructure']['tick'] = '49'
        (directory / 'query.stdout').write_text(json.dumps(raw))
        self.mutate(root, 'server/result.json', lambda r: r.update(query=raw))
        self.fails(root)

    def test_wire_ledger_mutants_raw_receipt_and_messages_cannot_self_certify(self):
        changes=[lambda q:q['query']['world']['states'][0]['infrastructure']['entries'].pop(),
                 lambda q:q['query']['world']['states'][0]['infrastructure']['entries'][1].update(source_key='other'),
                 lambda q:q['query']['world']['states'][0]['infrastructure']['entries'][1].update(id='0'),
                 lambda q:q['query']['world']['states'][0]['infrastructure']['entries'][1].update(operation_key='ledger-multiply'),
                 lambda q:q['query']['world']['states'][0]['infrastructure']['entries'][1].update(value='0'),
                 lambda q:q['query']['world']['states'][0]['infrastructure']['entries'][2].update(accumulated='1'),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].update(final_value='3'),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].update(base=None),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].update(tick=48),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].pop('entries'),
                 lambda q:q['query']['world']['states'].append(q['query']['world']['states'][0]),
                 lambda q:q['query']['world']['states'][0].update(id=True),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].update(final_value='Infinity'),
                 lambda q:q['query']['world']['states'][0]['infrastructure'].update(final_value='2147483648'),
                 lambda q:q['query']['world']['states'][1]['infrastructure'].update(tick='49'),
                 lambda q:q['messages'][1].update(key='other-notice'),
                 lambda q:q['command'].update(sequence='2'),
                 lambda q:q['snapshot']['state'].update(hour=False),
                 lambda q:q['state']['state'].update(date='2000-03-02')]
        for i,change in enumerate(changes):
            with self.subTest(mutant=i):
                root=self.artifacts();directory=root/'current-save-windows-latest/server'
                raw=json.loads((directory/'query.stdout').read_text());change(raw)
                # Keep selected message copies consistent too; source authority must reject the content.
                raw['messages']=[raw['welcome'],raw['messages'][1],raw['snapshot'],raw['query'],raw['command'],raw['state']]
                (directory/'query.stdout').write_text(json.dumps(raw))
                self.mutate(root,'server/result.json',lambda r:r.update(query=raw))
                self.fails(root)

    def test_wire_raw_only_receipt_only_and_duplicate_json_fields(self):
        for raw_only in (True,False):
            root=self.artifacts();directory=root/'current-save-windows-latest/server'
            raw=json.loads((directory/'query.stdout').read_text());raw['query']['world']['states'][0]['infrastructure']['tick']='49'
            if raw_only:(directory/'query.stdout').write_text(json.dumps(raw))
            else:self.mutate(root,'server/result.json',lambda r:r.update(query=raw))
            self.fails(root)
        root=self.artifacts();path=root/'current-save-windows-latest/server/query.stdout'
        path.write_text(path.read_text().replace('"tick": "48"','"tick": "49", "tick": "48"',1))
        self.fails(root)

    def test_independent_fixed_decimal_roundtrip_and_bounds(self):
        for value,bits in [('0.0000000002',1),('-0.0000000002',-1),('3.0000000005',12884901890),
                          ('0.000000000116415321826934814453125',0),('0.000000000349245965480804443359375',2)]:
            self.gate.wire_fx(value,bits)
        for value in (True,1,None,'NaN','1e3','2147483648','--1','00.5'):
            with self.assertRaises((AssertionError,TypeError,ValueError)):
                self.gate.wire_fx(value,1)

    def test_failed_or_missing_native_exit(self):
        for value in (1, None):
            with self.subTest(exit=value):
                root = self.artifacts()
                self.mutate(root, 'restart-1.command.json', lambda r: r.update(exit=value))
                self.fails(root)

    def test_missing_raw_stdout_or_stderr(self):
        for file in ('restart-1.stdout', 'cli-resume-2.stderr'):
            with self.subTest(file=file):
                root = self.artifacts()
                (root / 'current-save-windows-latest' / file).unlink()
                self.fails(root)

    def test_same_pid_or_receipt_pid_mismatch(self):
        root = self.artifacts()
        capture = json.loads((root / 'current-save-windows-latest/capture-1.stdout').read_text())
        self.mutate(root, 'restart-1.stdout', lambda r: r.update(pid=capture['pid']))
        self.fails(root)
        root = self.artifacts()
        self.mutate(root, 'restart-1.command.json', lambda r: r.update(pid=1))
        self.fails(root)

    def test_frozen_capture_command_cannot_replace_current(self):
        root = self.artifacts()
        self.mutate(root, 'capture-1.command.json', lambda r: r['command'].__setitem__(1, 'capture'))
        self.fails(root)

    def test_m0_run_cannot_replace_cli_resume(self):
        root = self.artifacts()
        self.mutate(root, 'cli-resume-1.command.json', lambda r: r.update(command=['oh_cli', 'run', '--scenario', 'testland']))
        self.fails(root)

    def test_changed_state_config_queue_ledger_or_canonical(self):
        changes = [
            lambda r: r['split']['state'].update(seed=8),
            lambda r: r['split']['config'].update(initial_speed=2),
            lambda r: r['split']['queue'].pop(),
            lambda r: r['continuous']['world']['inputs']['states'][0]['ledger'].update(value=0),
            lambda r: r.update(canonical_hex='00'),
            lambda r: r.update(split_hash='0' * 16),
        ]
        for i, change in enumerate(changes):
            with self.subTest(condition=i):
                root = self.artifacts()
                self.mutate(root, 'capture-1.stdout', change)
                self.fails(root)

    def test_save_paused_pack_or_header_bytes_changed(self):
        for file in ('run-1/saved.ohsave', 'run-2/paused.ohsave', 'run-1/pack/defines.toml'):
            with self.subTest(file=file):
                root = self.artifacts()
                path = root / 'current-save-windows-latest' / file
                path.write_bytes(path.read_bytes() + b'\x00')
                self.fails(root)
        root = self.artifacts()
        path = root / 'current-save-windows-latest/run-1/saved.ohsave'
        raw = bytearray(path.read_bytes()); raw[4] = 3; path.write_bytes(raw)
        self.fails(root)

    def test_extra_deleted_pack_file_and_forged_pack_summary(self):
        root = self.artifacts()
        (root / 'current-save-windows-latest/run-1/pack/extra.txt').write_text('extra')
        self.fails(root)
        root = self.artifacts()
        (root / 'current-save-windows-latest/run-1/pack/localisation/en/pack.ftl').unlink()
        self.fails(root)
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r['runs'][0]['header'].update(effective_defines_hash=0))
        self.fails(root)

    def test_wrong_executable_sha_or_changed_source_boundary(self):
        root = self.artifacts()
        self.mutate(root, 'restart-1.command.json', lambda r: r.update(exe_sha256='0' * 64))
        self.fails(root)
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r['source']['after'].update(raw_index_sha256='0' * 64))
        self.fails(root)

    def test_missing_server_proof_or_wrong_native_exit_input(self):
        for file in ('server/result.json', 'server-force/server.stderr', 'server/query.stdout'):
            with self.subTest(file=file):
                root = self.artifacts()
                (root / 'current-save-windows-latest' / file).unlink()
                self.fails(root)
        for change in (lambda r: r.update(server_exit=1), lambda r: r.update(save_sha256='0'*64),
                       lambda r: r.update(force=True), lambda r: r['input_header'].update(seed=99)):
            root = self.artifacts()
            self.mutate(root, 'server/result.json', change)
            self.fails(root)

    def test_required_additive_v5_namespace_and_every_native_stage(self):
        for name in ('economy-v5/result.json','economy-v5/restart-1.stdout','economy-v5/cli-full-2.command.json','economy-v5/run-1/journal.zip','economy-v5/run-2/repeat.ohsave'):
            root=self.artifacts();(root/'current-save-windows-latest'/name).unlink();self.fails(root)
        for stage in ('capture','restart','paused','cli-resume','cli-full','replay','cli-replay'):
            root=self.artifacts();self.mutate(root,f'economy-v5/{stage}-1.command.json',lambda r:r.update(exit=1));self.fails(root)

    def test_v5_full_dto_canonical_hash_pack_and_queue_corruption(self):
        for change in (
            lambda r:r['split'].update(canonical_hex='00'),
            lambda r:r['split'].update(hash='0'*16),
            lambda r:r['split']['dto']['queue'].pop(),
            lambda r:r['split']['dto']['economy']['nations']['1'].update(reserved=-1),
            lambda r:r['split']['dto']['economy'].update(definitions_hash=0),
            lambda r:r['pack'].update(content_hash=0)):
            root=self.artifacts();self.mutate(root,'economy-v5/capture-1.stdout',change);self.fails(root)
        for name in ('common/economy/wp14_native.toml','scenarios/wp14_native/scenario.toml'):
            root=self.artifacts();p=root/'current-save-windows-latest/economy-v5/run-1/pack'/name;p.write_bytes(p.read_bytes()+b'\n# mutation');self.fails(root)

    def test_v5_wrong_source_binary_pid_platform_or_repeat_identity(self):
        for change in (lambda r:r.update(head='0'*40),lambda r:r.update(helper_sha256='0'*64),lambda r:r.update(platform='Darwin'),lambda r:r['runs'][1]['files']['repeat.ohsave'].update(sha256='0'*64)):
            root=self.artifacts();self.mutate(root,'economy-v5/result.json',change);self.fails(root)
        root=self.artifacts();self.mutate(root,'economy-v5/cli-replay-2.command.json',lambda r:r.update(pid=0));self.fails(root)

    def test_required_v5_host_native_wire_and_pending_queue_mutants(self):
        for name in ('server/economy-v5/result.json','server-force/economy-v5/query.stdout'):
            root=self.artifacts();(root/'current-save-windows-latest'/name).unlink();self.fails(root)
        for change in (lambda r:r.update(server_exit=1),lambda r:r.update(force=True),lambda r:r['query']['before']['economy']['pending'].pop(),lambda r:r['query']['before']['economy'].update(state_hash='0'*16),lambda r:r['query']['after']['economy']['nations'][0]['political_capital'].update(value='4')):
            root=self.artifacts();self.mutate(root,'server/economy-v5/result.json',change);self.fails(root)


if __name__ == '__main__':
    unittest.main()
