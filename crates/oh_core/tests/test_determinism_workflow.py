"""Failure-path tests for the core CI evidence collector and comparison gate."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location(
    "core_determinism", ROOT / "crates/oh_core/tools/check_determinism.py"
)
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


class CoreDeterminismTests(unittest.TestCase):
    def setUp(self):
        temporary_root = ROOT / "target"
        temporary_root.mkdir(exist_ok=True)
        directory = tempfile.TemporaryDirectory(dir=temporary_root)
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)

    def artifacts(self, hashes=None):
        hashes = hashes or ["0dd81b8754bcc3f9"] * 3
        for runner, value in zip(gate.RUNNERS, hashes):
            path = self.root / f"core-hash-{runner}"
            path.mkdir()
            (path / "core-hash.txt").write_text(value + "\n", encoding="utf-8")

    def test_req_plat_02_three_matching_artifacts_pass(self):
        self.artifacts()
        self.assertEqual(gate.compare(self.root), "0dd81b8754bcc3f9")

    def test_req_plat_02_three_different_artifacts_fail(self):
        self.artifacts(["0dd81b8754bcc3f9", "0000000000000000", "0dd81b8754bcc3f9"])
        with self.assertRaisesRegex(ValueError, "mismatch"):
            gate.compare(self.root)

    def test_req_plat_02_missing_os_fails(self):
        self.artifacts(["0dd81b8754bcc3f9"] * 2)
        with self.assertRaisesRegex(ValueError, "three OS"):
            gate.compare(self.root)

    def test_req_plat_02_missing_hash_file_fails(self):
        self.artifacts()
        (self.root / "core-hash-macos-latest/core-hash.txt").unlink()
        with self.assertRaises(OSError):
            gate.compare(self.root)

    def test_req_plat_02_malformed_hash_fails(self):
        self.artifacts(["0dd81b8754bcc3f9", "not-a-hash", "0dd81b8754bcc3f9"])
        with self.assertRaisesRegex(ValueError, "16 lowercase hex"):
            gate.compare(self.root)

    def test_req_plat_02_extra_artifact_fails(self):
        self.artifacts()
        (self.root / "core-hash-fake-os").mkdir()
        with self.assertRaisesRegex(ValueError, "three OS"):
            gate.compare(self.root)

    def test_req_gen_04_capture_requires_two_matching_runs(self):
        process = subprocess.CompletedProcess([], 0, stdout="0dd81b8754bcc3f9\n")
        with patch.object(gate.subprocess, "run", return_value=process) as run:
            gate.capture(self.root)
        self.assertEqual(run.call_count, 2)
        self.assertEqual((self.root / "core-hash.txt").read_text(), process.stdout)

    def test_req_gen_04_capture_repeated_run_mismatch_fails(self):
        results = [subprocess.CompletedProcess([], 0, stdout=value) for value in
                   ["0dd81b8754bcc3f9\n", "0000000000000000\n"]]
        with patch.object(gate.subprocess, "run", side_effect=results):
            with self.assertRaisesRegex(ValueError, "two runs"):
                gate.capture(self.root)
        self.assertFalse((self.root / "core-hash.txt").exists())

    def test_req_gen_04_failed_probe_does_not_produce_artifact(self):
        with patch.object(gate.subprocess, "run", side_effect=subprocess.CalledProcessError(101, "cargo")):
            with self.assertRaises(subprocess.CalledProcessError):
                gate.capture(self.root)
        self.assertFalse((self.root / "core-hash.txt").exists())

    def test_req_gen_04_malformed_probe_does_not_produce_artifact(self):
        process = subprocess.CompletedProcess([], 0, stdout="\n")
        with patch.object(gate.subprocess, "run", return_value=process):
            with self.assertRaises(ValueError):
                gate.capture(self.root)
        self.assertFalse((self.root / "core-hash.txt").exists())


if __name__ == "__main__":
    unittest.main()
