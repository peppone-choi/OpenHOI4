"""Current evidence predicates; synthetic runner copies do not prove 3OS execution."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
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


if __name__ == '__main__':
    unittest.main()
