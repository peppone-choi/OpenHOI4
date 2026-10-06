#!/usr/bin/env python3
"""Collect a repeated core probe or compare all three OS artifacts. No game ticks."""
import argparse
from pathlib import Path
import re
import subprocess
import sys

RUNNERS = ("ubuntu-latest", "windows-latest", "macos-latest")
PROBE = ("cargo", "run", "--locked", "--quiet", "-p", "oh_core", "--example", "determinism")


def parse_hash(output):
    value = output.strip()
    if not re.fullmatch(r"[0-9a-f]{16}", value):
        raise ValueError("core hash must contain exactly 16 lowercase hex digits")
    return value


def capture(output_directory):
    hashes = [parse_hash(subprocess.run(PROBE, check=True, text=True,
                                       stdout=subprocess.PIPE).stdout) for _ in range(2)]
    if hashes[0] != hashes[1]:
        raise ValueError(f"core probe two runs mismatch: {hashes}")
    output_directory.mkdir(parents=True, exist_ok=True)
    (output_directory / "core-hash.txt").write_text(hashes[0] + "\n", encoding="utf-8")
    print(f"core probe two runs: {hashes[0]}")


def compare(artifact_root):
    expected = {f"core-hash-{runner}" for runner in RUNNERS}
    actual = {path.name for path in artifact_root.iterdir()}
    if actual != expected:
        raise ValueError(f"exactly three OS artifacts required: expected={sorted(expected)}, actual={sorted(actual)}")
    hashes = {}
    for runner in RUNNERS:
        path = artifact_root / f"core-hash-{runner}" / "core-hash.txt"
        hashes[runner] = parse_hash(path.read_text(encoding="utf-8"))
    if len(set(hashes.values())) != 1:
        raise ValueError(f"core OS hash mismatch: {hashes}")
    for runner, value in hashes.items():
        print(f"{runner}: {value}")
    return next(iter(hashes.values()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="command", required=True)
    capture_parser = subcommands.add_parser("capture")
    capture_parser.add_argument("--out", type=Path, required=True)
    compare_parser = subcommands.add_parser("compare")
    compare_parser.add_argument("--root", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "capture":
            capture(args.out)
        else:
            compare(args.root)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"core determinism: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
