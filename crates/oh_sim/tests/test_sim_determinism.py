"""Exercise actual simulation evidence capture/comparison failure paths."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

TOOL = Path(__file__).resolve().parents[1] / "tools/check_sim_determinism.py"
spec = importlib.util.spec_from_file_location("sim_evidence", TOOL)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)
HASH = "0123456789abcdef\n"

class SimEvidenceTests(unittest.TestCase):
    def tree(self, root):
        for runner in gate.RUNNERS:
            folder = root / f"sim-hash-{runner}"
            folder.mkdir()
            for name in ("sim-hash.txt", "run-1.txt", "run-2.txt"):
                (folder / name).write_text(HASH, encoding="utf-8")

    def test_capture_runs_real_1000_tick_cli_twice(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(gate.subprocess, "run") as run:
            run.return_value = subprocess.CompletedProcess(gate.COMMAND, 0, HASH)
            gate.capture(Path(tmp))
            self.assertEqual(run.call_count, 2)
            for call in run.call_args_list:
                self.assertEqual(call.args[0], ("cargo", "run", "--locked", "--quiet", "-p", "oh_cli", "--", "run", "--scenario", "testland", "--ticks", "1000", "--seed", "1", "--hash-out"))
                self.assertTrue(call.kwargs["check"])
            self.assertEqual((Path(tmp) / "run-2.txt").read_text(), HASH)

    def test_capture_rejects_repeated_hash_mismatch(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(gate.subprocess, "run") as run:
            run.side_effect = [subprocess.CompletedProcess([], 0, HASH), subprocess.CompletedProcess([], 0, "fedcba9876543210\n")]
            with self.assertRaises(ValueError): gate.capture(Path(tmp))
            self.assertFalse((Path(tmp) / "sim-hash.txt").exists())

    def test_capture_rejects_command_failure(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(gate.subprocess, "run", side_effect=subprocess.CalledProcessError(1, gate.COMMAND)):
            with self.assertRaises(subprocess.CalledProcessError): gate.capture(Path(tmp))

    def test_compare_accepts_three_equal_os(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); self.tree(root)
            self.assertEqual(gate.compare(root), HASH.strip())

    def test_compare_rejects_missing_extra_os_and_missing_repeat(self):
        for change in ("missing_os", "extra_os", "missing_repeat"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp); self.tree(root)
                if change == "missing_os":
                    folder = root / "sim-hash-macos-latest"
                    for file in folder.iterdir(): file.unlink()
                    folder.rmdir()
                elif change == "extra_os": (root / "sim-hash-fake").mkdir()
                else: (root / "sim-hash-macos-latest/run-2.txt").unlink()
                with self.assertRaises((ValueError, OSError)): gate.compare(root)

    def test_compare_rejects_invalid_or_mismatched_hash(self):
        for value in ("", "0123456789abcde", "0123456789abcdeg", "0123456789ABCDEF", "0123456789abcdef\nextra", "fedcba9876543210\n"):
            with self.subTest(value=value), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp); self.tree(root)
                (root / "sim-hash-windows-latest/sim-hash.txt").write_text(value)
                with self.assertRaises(ValueError): gate.compare(root)

    def test_compare_rejects_repeat_disagreement(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); self.tree(root)
            (root / "sim-hash-windows-latest/run-2.txt").write_text("fedcba9876543210\n")
            with self.assertRaises(ValueError): gate.compare(root)

    def test_compare_rejects_cross_os_mismatch_even_when_each_repeat_matches(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); self.tree(root)
            for name in ("sim-hash.txt", "run-1.txt", "run-2.txt"):
                (root / "sim-hash-macos-latest" / name).write_text("fedcba9876543210\n")
            with self.assertRaisesRegex(ValueError, "OS hash mismatch"): gate.compare(root)

    def test_failed_recapture_clears_old_success_marker(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(gate.subprocess, "run", side_effect=subprocess.CalledProcessError(1, gate.COMMAND)):
            marker = Path(tmp) / "sim-hash.txt"
            marker.write_text(HASH)
            with self.assertRaises(subprocess.CalledProcessError): gate.capture(Path(tmp))
            self.assertFalse(marker.exists())

class M1EvidenceTests(unittest.TestCase):
    def test_capture_invokes_actual_m1_pack_and_scenario_twice(self):
        with tempfile.TemporaryDirectory() as tmp, patch.object(gate.subprocess,"run") as run:
            run.return_value=subprocess.CompletedProcess([],0,HASH)
            gate.capture(Path(tmp),True)
            self.assertEqual(run.call_count,2)
            for call in run.call_args_list:
                self.assertIn("--pack",call.args[0]); self.assertIn("data/packs/testland",call.args[0]);self.assertIn("m1",call.args[0]);self.assertNotIn("--scenario testland", " ".join(call.args[0]))
    def test_compare_requires_m1_command_identity_on_three_os(self):
        import json
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            command=["cargo","run","--locked","--quiet","-p","oh_cli","--","run","--pack","data/packs/testland","--scenario","m1","--ticks","1000","--seed","1","--hash-out"]
            for runner in gate.RUNNERS:
                folder=root/f"m1-sim-hash-{runner}";folder.mkdir()
                for name in ("sim-hash.txt","run-1.txt","run-2.txt"):(folder/name).write_text(HASH)
                (folder/"command.json").write_text(json.dumps(command))
            self.assertEqual(gate.compare(root,True),HASH.strip())
            (root/"m1-sim-hash-windows-latest/command.json").write_text(json.dumps(list(gate.COMMAND)))
            with self.assertRaisesRegex(ValueError,"command identity mismatch"):gate.compare(root,True)
    def test_m1_does_not_accept_m0_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);SimEvidenceTests().tree(root)
            with self.assertRaises(ValueError):gate.compare(root,True)
    def test_m1_failed_capture_cannot_leave_a_success_marker(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(gate.subprocess,"run",side_effect=subprocess.CalledProcessError(1,[])):
            root=Path(tmp);(root/"sim-hash.txt").write_text(HASH)
            with self.assertRaises(subprocess.CalledProcessError):gate.capture(root,True)
            self.assertFalse((root/"sim-hash.txt").exists())

if __name__ == "__main__": unittest.main()
