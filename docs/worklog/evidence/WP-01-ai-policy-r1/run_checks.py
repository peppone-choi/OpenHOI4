"""Reproduce WP-01 policy follow-up command logs from this worktree only."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[4]
OUT = Path(__file__).resolve().parent
env = os.environ.copy()
env["PYTHONIOENCODING"] = "utf-8"
env["PATH"] = str(Path.home() / ".cargo/bin") + os.pathsep + env["PATH"]
cargo = shutil.which("cargo", path=env["PATH"])
node = shutil.which("node", path=env["PATH"])
npm = Path(node).parent / "node_modules/npm/bin/npm-cli.js"
commands = {
    "fixtures": [sys.executable, "-m", "unittest", "discover", "-s", "tools/tests", "-p", "test_wp01.py", "-v"],
    "npm-ci": [node, str(npm), "--prefix", "client", "ci"],
    "client-build": [node, str(npm), "--prefix", "client", "run", "build"],
    "fmt": [cargo, "fmt", "--check"],
    "clippy": [cargo, "clippy", "--workspace", "--", "-D", "warnings"],
    "rust-tests": [cargo, "test", "--workspace"],
    "hash-1": [cargo, "run", "-p", "oh_cli", "--", "run", "--scenario", "testland", "--days", "365", "--seed", "1", "--hash-out"],
    "hash-2": [cargo, "run", "-p", "oh_cli", "--", "run", "--scenario", "testland", "--days", "365", "--seed", "1", "--hash-out"],
    "assets": [sys.executable, "tools/check_assets.py", "--release"],
    "docs": [sys.executable, "tools/check_docs.py"],
    "diff": ["git", "diff", "--check"],
}
selected = sys.argv[1:] or list(commands)
results_path = OUT / "results.json"
results = json.loads(results_path.read_text(encoding="utf-8")) if results_path.exists() else {}
for name in selected:
    command = commands[name]
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    log = f"workdir: {ROOT}\ncommand: {subprocess.list2cmdline(command)}\nexit: {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    (OUT / f"{name}.txt").write_text(log, encoding="utf-8")
    results[name] = {"command": command, "workdir": str(ROOT), "exit": result.returncode, "log": f"{name}.txt"}
    results_path.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{name}: exit {result.returncode}", flush=True)
    if result.returncode:
        print((result.stdout + result.stderr)[-3000:], flush=True)
sys.exit(int(any(results[name]["exit"] for name in selected)))
