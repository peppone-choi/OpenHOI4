#!/usr/bin/env python3
"""Check shipped data and assets against provenance records (REQ-LEG-02)."""
import argparse
from datetime import datetime
import hashlib
from pathlib import Path, PurePosixPath
import re
import sys
import tomllib

ASSET_ROOTS = ("assets", "data/packs", "client/public", "client/src")
CLIENT_SOURCE_SUFFIXES = {".ts", ".tsx", ".js", ".jsx", ".css"}
LICENSES = {"CC0-1.0", "CC-BY-4.0", "CC-BY-SA-4.0", "OFL-1.1", "public-domain", "original"}
IMAGE_SUFFIXES = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".avif", ".bmp", ".tif", ".tiff"}


def check_ai_review(root: Path, target: Path, record: dict) -> list[str]:
    """Check human review records and file integrity, not legal approval."""
    errors = []
    prefix = f"{record['path']}: AI-generated asset"

    def error(field: str, reason: str) -> None:
        errors.append(f"{prefix}: {field} {reason}")

    def text_field(table: dict, field: str) -> str | None:
        value = table.get(field)
        if not isinstance(value, str) or not value.strip():
            error(field, "must be a nonempty string")
            return None
        return value

    def digest(table: dict, field: str, file: Path | None) -> None:
        expected = text_field(table, field)
        if expected is None:
            return
        if not re.fullmatch(r"[0-9a-f]{64}", expected):
            error(field, "must be a lowercase SHA-256 digest")
            return
        if file is not None:
            try:
                with file.open("rb") as stream:
                    actual = hashlib.file_digest(stream, "sha256").hexdigest()
                if actual != expected:
                    error(field, "does not match the referenced file")
            except OSError as exc:
                error(field, f"cannot read referenced file: {exc}")

    def reference(table: dict, field: str) -> Path | None:
        path = text_field(table, field)
        if path is None:
            return None
        relative = PurePosixPath(path)
        if relative.is_absolute() or ".." in relative.parts or "\\" in path or ":" in path or str(relative) != path:
            error(field, "must be a normalized relative POSIX path")
            return None
        file = root / path
        try:
            if not file.resolve().is_relative_to(root):
                error(field, "resolves outside root")
                return None
            if not file.is_file() or file.stat().st_size == 0:
                error(field, "must reference an existing nonempty file")
                return None
        except (OSError, RuntimeError) as exc:
            error(field, f"cannot resolve referenced file: {exc}")
            return None
        return file

    if target.suffix.lower() not in IMAGE_SUFFIXES:
        error("path", "must refer to an image for release (OPEN-09)")
    provenance = record.get("ai_provenance")
    if not isinstance(provenance, dict):
        error("ai_provenance", "must be a table")
    else:
        text_field(provenance, "tool")
        terms = reference(provenance, "usage_terms_path")
        digest(provenance, "usage_terms_sha256", terms)
    review = record.get("distribution_review")
    if not isinstance(review, dict):
        error("distribution_review", "must be a table")
    else:
        if review.get("status") != "approved":
            error("status", "must be 'approved' for release (OPEN-09)")
        text_field(review, "reviewer")
        timestamp = text_field(review, "reviewed_at")
        if timestamp is not None:
            try:
                if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(Z|[+-](?:[01][0-9]|2[0-3]):[0-5][0-9])", timestamp):
                    raise ValueError("invalid timestamp format")
                datetime.fromisoformat(timestamp)
            except ValueError:
                error("reviewed_at", "must be a valid timestamp with seconds and timezone (YYYY-MM-DDTHH:MM:SSZ or +/-HH:MM)")
        evidence = reference(review, "evidence_path")
        digest(review, "evidence_sha256", evidence)
        digest(review, "asset_sha256", target)
    return errors


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
            errors.extend(check_ai_review(root, target, record))
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
    parser.add_argument("--release", action="store_true", help="require provenance and approved human distribution review records for AI images (OPEN-09)")
    args = parser.parse_args()
    errors = check(args.root.resolve(), args.release)
    for error in errors:
        print(f"[ASSET] {error}", file=sys.stderr)
    print(f"assets: {len(errors)} error(s)")
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
