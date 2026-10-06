#!/usr/bin/env python3
"""Reject networking/async dependencies in deterministic simulation layers."""
import json
import subprocess
import sys


def main() -> int:
    metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], text=True))
    packages = {item["id"]: item for item in metadata["packages"]}
    graph = {item["id"]: item["dependencies"] for item in metadata["resolve"]["nodes"]}
    forbidden = {"tokio", "axum", "hyper", "reqwest", "async-std", "smol", "mio", "socket2"}
    failed = False
    for package in metadata["packages"]:
        if package["name"] not in {"oh_core", "oh_data", "oh_sim", "oh_ai", "oh_save"}:
            continue
        # Keep the documented cargo tree command visible in evidence.
        subprocess.run(["cargo", "tree", "--locked", "--package", package["name"], "--edges", "normal,build"], check=True)
        pending = list(graph[package["id"]])
        visited = set()
        while pending:
            dependency = pending.pop()
            if dependency in visited:
                continue
            visited.add(dependency)
            if packages[dependency]["name"] in forbidden:
                print(f"{package['name']}: forbidden dependency {packages[dependency]['name']}", file=sys.stderr)
                failed = True
            pending.extend(graph[dependency])
    return int(failed)


if __name__ == "__main__":
    sys.exit(main())
