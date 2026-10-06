#!/usr/bin/env python3
"""Check shipped data and assets against provenance records (REQ-LEG-02)."""
import argparse
from pathlib import Path, PurePosixPath
import sys
import tomllib

ASSET_ROOTS = ("assets", "data/packs", "client/public", "client/src")
CLIENT_SOURCE_SUFFIXES = {".ts", ".tsx", ".js", ".jsx", ".css"}
LICENSES = {"CC0-1.0", "CC-BY-4.0", "CC-BY-SA-4.0", "OFL-1.1", "public-domain", "original"}


def check(root: Path, release: bool) -> list[str]:
    errors = []
    manifest = root / "assets/ASSETS.toml"
    try:
        records = tomllib.loads(manifest.read_text(encoding="utf-8"))["asset"]
        if not isinstance(records, list):
            raise ValueError("asset must be an array")
    except (OSError, ValueError, KeyError) as exc:
        return [f"invalid manifest: {exc}"]
    registered = set()
    for record in records:
        if not isinstance(record, dict):
            errors.append("asset record must be a table")
            continue
        path = record.get("path", "")
        if not isinstance(path, str) or not path:
            errors.append("asset path must be a nonempty relative POSIX path")
            continue
        relative = PurePosixPath(path)
        if relative.is_absolute() or ".." in relative.parts or "\\" in path or ":" in path or str(relative) != path:
            errors.append(f"{path}: path must be a normalized relative POSIX path")
            continue
        target = root / path
        if not target.resolve().is_relative_to(root):
            errors.append(f"{path}: relative path resolves outside root")
            continue
        if path in registered:
            errors.append(f"{path}: duplicate record")
        registered.add(path)
        if not target.is_file():
            errors.append(f"{path}: missing file")
        for field in ("author", "source", "license"):
            if not isinstance(record.get(field), str) or not record[field].strip():
                errors.append(f"{path}: missing {field}")
        license = record.get("license")
        if not isinstance(license, str) or license not in LICENSES:
            errors.append(f"{path}: unapproved license {license!r}")
        if target.suffix.lower() in {".ttf", ".otf", ".woff", ".woff2"} and license != "OFL-1.1":
            errors.append(f"{path}: font license must be OFL-1.1")
        for field in ("modified", "ai_generated"):
            if type(record.get(field)) is not bool:
                errors.append(f"{path}: {field} must be a boolean")
        if not isinstance(record.get("notes"), str):
            errors.append(f"{path}: notes must be a string")
        if release and record.get("ai_generated") is True:
            errors.append(f"{path}: AI-generated asset blocked for release (OPEN-09)")
    for folder in ASSET_ROOTS:
        directory = root / folder
        if not directory.exists():
            continue
        for file in sorted(directory.rglob("*")):
            if file.is_file() and file != manifest:
                if folder == "client/src" and file.suffix.lower() in CLIENT_SOURCE_SUFFIXES:
                    continue
                path = file.relative_to(root).as_posix()
                if not file.resolve().is_relative_to(root):
                    errors.append(f"{path}: relative path resolves outside root")
                if path not in registered:
                    errors.append(f"{path}: unregistered asset/data file")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--release", action="store_true", help="reject AI-generated assets pending OPEN-09")
    args = parser.parse_args()
    errors = check(args.root.resolve(), args.release)
    for error in errors:
        print(f"[ASSET] {error}", file=sys.stderr)
    print(f"assets: {len(errors)} error(s)")
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
