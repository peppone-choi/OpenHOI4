"""Synthetic predicate regressions built from one actual local capture.
Labels copied here are never additional actual three-OS success evidence.
"""
import copy
import importlib.util
import json
import os
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('trigger_gate', ROOT / 'crates/oh_sim/tools/check_trigger_determinism.py')
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def write(path, value):
    path.write_text(json.dumps(value, indent=2), encoding='utf-8')


def reseal(folder):
    manifest = gate.strict_json(folder / 'manifest.json')
    manifest.update(gate.strict_json(folder / 'result.json'))
    manifest['files'] = gate.files(folder)
    manifest['files'].pop('manifest.json')
    write(folder / 'manifest.json', manifest)


class TriggerPredicates(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = Path(os.environ.get('TRIGGER_TEST_CAPTURE', ROOT / 'target/trigger-determinism'))
        gate.verify_folder(cls.source, require_clean=False)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        for runner, system in gate.RUNNERS.items():
            folder = self.root / f'trigger-{runner}'
            shutil.copytree(self.source, folder)
            before = gate.strict_json(folder / 'source-before.json')
            before.update(status='', diff='', cached='')
            write(folder / 'source-before.json', before)
            write(folder / 'source-after.json', before)
            result = gate.strict_json(folder / 'result.json')
            result.update(dirty=False, platform=system)
            write(folder / 'result.json', result)
            reseal(folder)
        self.folder = self.root / 'trigger-windows-latest'

    def tearDown(self):
        self.temp.cleanup()

    def rejects(self):
        with self.assertRaises((AssertionError, KeyError, ValueError, FileNotFoundError)):
            gate.compare_artifacts(self.root, emit=False)

    def test_synthetic_positive_is_predicate_only(self):
        gate.compare_artifacts(self.root, emit=False)

    def test_missing_os(self):
        shutil.rmtree(self.root / 'trigger-macos-latest')
        self.rejects()

    def test_dirty_even_if_head_equal(self):
        for name in ('source-before.json', 'source-after.json'):
            value = gate.strict_json(self.folder / name)
            value['status'] = ' M crates/oh_sim/src/trigger.rs'
            write(self.folder / name, value)
        result = gate.strict_json(self.folder / 'result.json')
        result['dirty'] = True
        write(self.folder / 'result.json', result)
        reseal(self.folder)
        self.rejects()

    def test_different_head(self):
        for name in ('source-before.json', 'source-after.json', 'result.json'):
            value = gate.strict_json(self.folder / name)
            value['head'] = '0' * 40
            write(self.folder / name, value)
        reseal(self.folder)
        self.rejects()

    def test_canonical_hash_and_definition_tampering_even_if_manifest_resigned(self):
        for key in ('hash', 'canonical_hex', 'definition', 'state'):
            path = self.folder / 'capture-1/stdout'
            original = path.read_bytes()
            value = json.loads(original)
            case = value['cases']['ended']
            if key == 'hash':
                case['hash'] = '0' * 16
            elif key == 'canonical_hex':
                case['canonical_hex'] += '00'
            elif key == 'definition':
                case['dto']['trigger']['definitions_hash'] ^= 1
            else:
                case['dto']['trigger']['flags'][0][1] = []
            write(path, value)
            reseal(self.folder)
            self.rejects()
            path.write_bytes(original)
        reseal(self.folder)

    def test_save_bytes_tampering_resigned(self):
        path = self.folder / 'run-1/ended.ohsave'
        value = bytearray(path.read_bytes())
        value[-1] ^= 1
        path.write_bytes(value)
        reseal(self.folder)
        self.rejects()

    def test_missing_native_stdout(self):
        (self.folder / 'native-resume-1-ended/stdout').unlink()
        reseal(self.folder)
        self.rejects()

    def test_native_exit_pid_executable_and_action(self):
        path = self.folder / 'native-resume-1-ended/receipt.json'
        original = path.read_bytes()
        for key in ('exit', 'pid', 'executable_sha256', 'command', 'cwd'):
            value = json.loads(original)
            if key == 'exit':
                value[key] = 1
            elif key == 'pid':
                del value[key]
            elif key == 'executable_sha256':
                value[key] = '0' * 64
            elif key == 'cwd':
                value[key] = '/wrong/source'
            else:
                value[key][1] = 'capture'
            write(path, value)
            reseal(self.folder)
            self.rejects()
            path.write_bytes(original)
        reseal(self.folder)

    def test_pack_source_identity_tampering(self):
        path = self.folder / 'run-1/pack/scenarios/m1/scenario.toml'
        path.write_text(path.read_text(encoding='utf-8').replace('"x", "y"', '"x", "z"'), encoding='utf-8')
        reseal(self.folder)
        self.rejects()


if __name__ == '__main__':
    unittest.main()
