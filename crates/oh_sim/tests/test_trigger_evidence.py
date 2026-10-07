"""Synthetic predicate regressions built from one actual local capture.
Labels copied here are never additional actual three-OS success evidence.
"""
import copy
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
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
        cls.expected_head=gate.strict_json(cls.source/'result.json')['head']

    def setUp(self):
        scratch = Path(os.environ.get('TRIGGER_SCRATCH_ROOT', ROOT / 'target/trigger-predicate-scratch'))
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=scratch)
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
            gate._compare_for_head(self.root, self.expected_head, emit=False)

    def test_synthetic_positive_is_predicate_only(self):
        gate._compare_for_head(self.root, self.expected_head, emit=False)

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

    def public_fixture(self, evidence_relative='target/artifacts'):
        # Actual Git reads in an isolated ignored fixture; no mock or tracked
        # checkout mutation. OS labels in its artifacts remain synthetic.
        repo = self.root / 'public-source'
        repo.mkdir()
        shutil.copyfile(ROOT / '.gitignore', repo / '.gitignore')
        for relative in ('crates/oh_sim/tools/check_trigger_determinism.py',
                         'crates/oh_save/tools/save_reference.py',
                         'crates/oh_save/tools/check_save_current.py',
                         'crates/oh_save/defines.toml'):
            target = repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / relative, target)
        def git(*args):
            return subprocess.check_output(['git', '--no-optional-locks', '-C', str(repo), *args], text=True, encoding='utf-8').strip()
        git('init', '--quiet')
        git('-c', 'core.autocrlf=false', 'add', '.')
        git('-c', 'user.name=Predicate test', '-c', 'user.email=predicate@invalid', 'commit', '--quiet', '-m', 'Isolated public comparer input')
        current = git('rev-parse', 'HEAD')
        evidence = repo / evidence_relative
        evidence.mkdir(parents=True)
        for runner in gate.RUNNERS:
            folder = evidence / f'trigger-{runner}'
            shutil.copytree(self.root / f'trigger-{runner}', folder)
            for name in ('source-before.json', 'source-after.json', 'result.json'):
                value = gate.strict_json(folder / name)
                value['head'] = current
                write(folder / name, value)
            reseal(folder)
        return repo, evidence, current

    def workflow_download_path(self):
        workflow = (ROOT / '.github/workflows/trigger-determinism.yml').read_text(encoding='utf-8')
        download = workflow.split('      - uses: actions/download-artifact@', 1)[1]
        destination = next(line.strip()[6:] for line in download.splitlines() if line.strip().startswith('path: '))
        command = next(line.strip()[5:] for line in download.splitlines() if line.strip().startswith('run: '))
        arguments = shlex.split(command)
        self.assertEqual(arguments[:4], ['python', 'crates/oh_sim/tools/check_trigger_determinism.py', 'compare', '--root'])
        self.assertEqual(len(arguments), 5)
        self.assertEqual(destination, arguments[4], 'download and public comparer must use the same path')
        self.assertFalse(Path(destination).is_absolute())
        self.assertNotIn('..', Path(destination).parts)
        return destination

    def fixture_git(self, repo, *args):
        return subprocess.run(['git', '--no-optional-locks', '-C', str(repo), *args], capture_output=True, text=True, encoding='utf-8')

    def test_public_workflow_download_keeps_source_clean(self):
        destination = self.workflow_download_path()
        repo, evidence, _ = self.public_fixture('trigger-evidence')
        payload = gate.files(evidence)
        self.assertEqual(self.fixture_git(repo, 'diff', '--exit-code').returncode, 0)
        legacy = self.run_public(repo, evidence, receipt_suffix='legacy')
        self.assertNotEqual(legacy.returncode, 0)
        self.assertIn('current source checkout is dirty', legacy.stderr.decode(errors='replace'))
        moved = repo / destination
        self.assertTrue(evidence.resolve().is_relative_to(repo.resolve()))
        self.assertTrue(moved.resolve().is_relative_to(repo.resolve()))
        if moved != evidence:
            moved.parent.mkdir(parents=True, exist_ok=True)
            shutil.move(str(evidence), str(moved))
        self.assertEqual(gate.files(moved), payload, 'download payload bytes must remain identical')
        result = self.run_public(repo, moved, receipt_suffix='workflow')
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))
        self.assertEqual(self.fixture_git(repo, 'status', '--porcelain').stdout, '')
        self.assertEqual(self.fixture_git(repo, 'check-ignore', '-q', str(moved / 'trigger-windows-latest/result.json')).returncode, 0)

    def test_public_legacy_unignored_download_is_rejected(self):
        repo, evidence, _ = self.public_fixture('trigger-evidence')
        self.assertEqual(self.fixture_git(repo, 'diff', '--exit-code').returncode, 0)
        self.assertIn('?? trigger-evidence/', self.fixture_git(repo, 'status', '--porcelain').stdout)
        self.assertEqual(self.fixture_git(repo, 'check-ignore', '-q', str(evidence / 'trigger-windows-latest/result.json')).returncode, 1)
        result = self.run_public(repo, evidence)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('current source checkout is dirty', result.stderr.decode(errors='replace'))

    def test_public_workflow_path_missing_os_is_rejected(self):
        repo, evidence, _ = self.public_fixture(self.workflow_download_path())
        shutil.rmtree(evidence / 'trigger-macos-latest')
        result = self.run_public(repo, evidence)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('exact three OS artifacts required', result.stderr.decode(errors='replace'))

    def test_public_workflow_path_resigned_invalid_save_is_rejected(self):
        repo, evidence, _ = self.public_fixture(self.workflow_download_path())
        folder = evidence / 'trigger-windows-latest'
        path = folder / 'run-1/ended.ohsave'
        raw = bytearray(path.read_bytes())
        raw[-1] ^= 1
        path.write_bytes(raw)
        reseal(folder)
        result = self.run_public(repo, evidence)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(self.fixture_git(repo, 'status', '--porcelain').stdout, '')

    def run_public(self, repo, evidence, receipt_suffix=None):
        cmd = [sys.executable, str(repo / 'crates/oh_sim/tools/check_trigger_determinism.py'), 'compare', '--root', str(evidence)]
        def source_identity():
            def git(*args):
                return subprocess.check_output(['git', '--no-optional-locks', '-C', str(repo), *args], text=True, encoding='utf-8').strip()
            return {'head':git('rev-parse','HEAD'),'status':git('status','--porcelain'),
                    'index_sha256':hashlib.sha256((repo/'.git/index').read_bytes()).hexdigest()}
        before = source_identity()
        executable_sha = hashlib.sha256(Path(sys.executable).read_bytes()).hexdigest()
        process = subprocess.Popen(cmd, cwd=repo, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        stdout, stderr = process.communicate()
        result = subprocess.CompletedProcess(cmd, process.returncode, stdout, stderr)
        after = source_identity()
        self.assertEqual(before, after, 'public comparison must preserve source/index')
        destination = Path(os.environ.get('TRIGGER_PUBLIC_EVIDENCE', ROOT / 'target/trigger-public-predicates')) / self._testMethodName
        if receipt_suffix is not None:
            destination = destination / receipt_suffix
        destination.mkdir(parents=True, exist_ok=False)
        (destination / 'stdout').write_bytes(result.stdout)
        (destination / 'stderr').write_bytes(result.stderr)
        write(destination / 'receipt.json', {'command':cmd,'cwd':str(repo),'native_exit':result.returncode,
              'pid':process.pid,'executable':sys.executable,'executable_sha256':executable_sha,
              'source_before':before,'source_after':after,'synthetic_os_labels':True})
        return result

    def test_public_all_same_stale_head_is_refused(self):
        repo, evidence, current = self.public_fixture()
        stale = 'f44e16e079cad68e21825a1d9c7bcf1745170412'
        self.assertNotEqual(current, stale)
        for runner in gate.RUNNERS:
            folder = evidence / f'trigger-{runner}'
            for name in ('source-before.json', 'source-after.json', 'result.json'):
                value = gate.strict_json(folder / name)
                value['head'] = stale
                write(folder / name, value)
            reseal(folder)
        self.assertNotEqual(self.run_public(repo,evidence).returncode, 0)

    def test_public_dirty_current_source_is_refused(self):
        repo, evidence, _ = self.public_fixture()
        tracked = repo / 'crates/oh_sim/tools/check_trigger_determinism.py'
        tracked.write_bytes(tracked.read_bytes()+b'\n# Controlled dirty Git fixture.\n')
        self.assertNotEqual(self.run_public(repo,evidence).returncode, 0)

    def test_public_synthetic_same_clean_source_predicate(self):
        repo, evidence, _ = self.public_fixture()
        result = self.run_public(repo,evidence)
        self.assertEqual(result.returncode, 0, result.stderr.decode(errors='replace'))


if __name__ == '__main__':
    unittest.main()
