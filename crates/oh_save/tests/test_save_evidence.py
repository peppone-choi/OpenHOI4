"""Negative tests of evidence validation. Copied runner labels are synthetic;
these tests are never evidence that Linux/macOS executables actually ran.
"""
import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1] / 'tools'
sys.path.insert(0, str(TOOLS))
SPEC = importlib.util.spec_from_file_location('save_gate', TOOLS / 'check_save_determinism.py')
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)

class EvidenceFailures(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory(dir=gate.ROOT / 'target')
        cls.source = Path(cls.directory.name) / 'actual-windows-or-current-os'
        with contextlib.redirect_stdout(io.StringIO()):
            gate.capture(cls.source)

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def artifacts(self, include=gate.RUNNERS):
        directory = tempfile.TemporaryDirectory(dir=gate.ROOT / 'target')
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        for runner in include:
            folder = root / f'save-{runner}'
            shutil.copytree(self.source, folder)
            record = json.loads((folder / 'result.json').read_text())
            record['platform'] = {'ubuntu-latest': 'Linux', 'windows-latest': 'Windows', 'macos-latest': 'Darwin'}[runner]
            record['dirty'] = False  # synthetic negative predicate test only
            (folder / 'result.json').write_text(json.dumps(record))
        return root

    def mutate(self, root, file, mutate):
        path = root / 'save-windows-latest' / file
        data = json.loads(path.read_text())
        mutate(data)
        path.write_text(json.dumps(data))

    def fails(self, root):
        with self.assertRaises((ValueError, AssertionError, OSError, KeyError)):
            gate.compare(root)

    def test_missing_os(self):
        self.fails(self.artifacts(gate.RUNNERS[:2]))

    def test_different_head(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(head='0' * 40))
        self.fails(root)

    def test_dirty_source(self):
        root = self.artifacts()
        self.mutate(root, 'result.json', lambda r: r.update(dirty=True))
        self.fails(root)

    def test_hash_and_fixture_bytes_disagreement(self):
        root = self.artifacts()
        path = root / 'save-windows-latest/run-1/saved.ohsave'
        path.write_bytes(path.read_bytes() + b'\x00')
        self.fails(root)

    def test_m0_run_cannot_replace_resume(self):
        root = self.artifacts()
        self.mutate(root, 'cli-resume-1.command.json', lambda r: r.update(command=['oh_cli', 'run', '--scenario', 'testland']))
        self.fails(root)

    def test_failed_command(self):
        root = self.artifacts()
        self.mutate(root, 'restart-1.command.json', lambda r: r.update(exit=1))
        self.fails(root)

    def test_cached_same_process(self):
        root = self.artifacts()
        capture = json.loads((root / 'save-windows-latest/capture-1.stdout').read_text())
        self.mutate(root, 'restart-1.stdout', lambda r: r.update(pid=capture['pid']))
        self.fails(root)

    def test_rawbit_canonical_mismatch(self):
        root = self.artifacts()
        self.mutate(root, 'capture-1.stdout', lambda r: r['split']['world']['inputs']['states'][0].update(infrastructure=0))
        self.fails(root)

if __name__ == '__main__':
    unittest.main()
