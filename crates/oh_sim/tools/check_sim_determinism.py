#!/usr/bin/env python3
"""Capture two real 1000-tick CLI runs; compare all three OS artifacts."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

RUNNERS = ("ubuntu-latest", "windows-latest", "macos-latest")
COMMAND = ("cargo", "run", "--locked", "--quiet", "-p", "oh_cli", "--", "run",
           "--scenario", "testland", "--ticks", "1000", "--seed", "1", "--hash-out")

def parse_hash(output):
    value = output.strip()
    if not re.fullmatch(r"[0-9a-f]{16}", value):
        raise ValueError("simulation hash must contain exactly 16 lowercase hex digits")
    return value

def capture(output_directory, m1=False):
    # Clear an old success marker so failed recapture cannot leave stale success.
    output_directory.mkdir(parents=True, exist_ok=True)
    (output_directory / "sim-hash.txt").unlink(missing_ok=True)
    command = COMMAND if not m1 else ("cargo", "run", "--locked", "--quiet", "-p", "oh_cli", "--", "run", "--pack", "data/packs/testland", "--scenario", "m1", "--ticks", "1000", "--seed", "1", "--hash-out")
    if m1: (output_directory / "command.json").write_text(json.dumps(command), encoding="utf-8")
    hashes = []
    for index in (1, 2):
        output = subprocess.run(command, check=True, text=True, stdout=subprocess.PIPE).stdout
        hashes.append(parse_hash(output))
        (output_directory / f"run-{index}.txt").write_text(output, encoding="utf-8")
    if hashes[0] != hashes[1]:
        raise ValueError(f"simulation two runs mismatch: {hashes}")
    (output_directory / "sim-hash.txt").write_text(hashes[0] + "\n", encoding="utf-8")
    print(f"actual simulation: 1000 ticks, seed 1, two runs: {hashes[0]}")

def compare(artifact_root, m1=False):
    prefix = "m1-sim-hash" if m1 else "sim-hash"
    expected = {f"{prefix}-{runner}" for runner in RUNNERS}
    actual = {path.name for path in artifact_root.iterdir()}
    if actual != expected:
        raise ValueError(f"exactly three OS artifacts required: expected={sorted(expected)}, actual={sorted(actual)}")
    hashes = {}
    for runner in RUNNERS:
        folder = artifact_root / f"{prefix}-{runner}"
        if m1:
            command=json.loads((folder / "command.json").read_text(encoding="utf-8"))
            expected_command=["cargo","run","--locked","--quiet","-p","oh_cli","--","run","--pack","data/packs/testland","--scenario","m1","--ticks","1000","--seed","1","--hash-out"]
            if command != expected_command: raise ValueError(f"{runner}: M1 command identity mismatch")
        value = parse_hash((folder / "sim-hash.txt").read_text(encoding="utf-8"))
        repeats = [parse_hash((folder / f"run-{index}.txt").read_text(encoding="utf-8")) for index in (1, 2)]
        if repeats != [value, value]:
            raise ValueError(f"{runner}: repeated run evidence disagrees with hash: {repeats}")
        hashes[runner] = value
    if len(set(hashes.values())) != 1:
        raise ValueError(f"simulation OS hash mismatch: {hashes}")
    for runner, value in hashes.items(): print(f"{runner}: {value}")
    return next(iter(hashes.values()))

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("capture").add_argument("--out", type=Path, required=True)
    sub.add_parser("compare").add_argument("--root", type=Path, required=True)
    for child in sub.choices.values(): child.add_argument("--m1", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "capture": capture(args.out,args.m1)
        else: compare(args.root,args.m1)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"simulation determinism: {error}", file=sys.stderr)
        return 1
    return 0

if __name__ == "__main__": sys.exit(main())
