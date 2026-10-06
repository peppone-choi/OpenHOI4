"""WP-01 requirement fixtures; invoke the production checkers as processes."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.stdout.reconfigure(encoding="utf-8")


def run(*args):
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
    print(f"$ {' '.join(map(str, args))}\nexit={result.returncode}\n{result.stdout}{result.stderr}")
    return result


class PlatformTests(unittest.TestCase):
    def test_req_plat_01_buildable_workspace_and_ci(self):
        with (ROOT / "Cargo.toml").open("rb") as stream:
            workspace = tomllib.load(stream)
        names = ["core", "data", "sim", "ai", "save", "proto", "server", "cli"]
        self.assertEqual(set(workspace["workspace"]["members"]), {f"crates/oh_{n}" for n in names})
        for name in names:
            self.assertTrue((ROOT / f"crates/oh_{name}/Cargo.toml").is_file())
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        for required in ["ubuntu-latest", "windows-latest", "macos-latest", "cargo build --workspace", "cargo test --workspace", "npm --prefix client ci", "npm --prefix client run build", "tools/check_docs.py"]:
            self.assertIn(required, workflow)
        package = json.loads((ROOT / "client/package.json").read_text())
        for command in ["dev", "build", "test", "typecheck"]:
            self.assertIn(command, package["scripts"])


class AssetTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / "assets").mkdir()
        (self.root / "data/packs/demo").mkdir(parents=True)
        (self.root / "data/packs/demo/defines.toml").write_text("[demo]\n", encoding="utf-8")
        self.path = "data/packs/demo/defines.toml"

    def manifest(self, **overrides):
        fields = dict(path=self.path, author="Fixture author", source="original", license="original", modified=False, ai_generated=False, notes="test")
        fields.update(overrides)
        def value(v):
            if isinstance(v, dict):
                return "{ " + ", ".join(f"{k} = {value(item)}" for k, item in v.items()) + " }"
            return str(v).lower() if isinstance(v, bool) else json.dumps(v)
        content = "[[asset]]\n" + "\n".join(f"{key} = {value(v)}" for key, v in fields.items())
        if self.path != "data/packs/demo/defines.toml" and fields["path"] != "data/packs/demo/defines.toml":
            content += '\n\n[[asset]]\npath = "data/packs/demo/defines.toml"\nauthor = "Fixture author"\nsource = "original"\nlicense = "original"\nmodified = false\nai_generated = false\nnotes = "test"'
        (self.root / "assets/ASSETS.toml").write_text(content + "\n", encoding="utf-8")

    def check(self, code, diagnostic="", *extra):
        result = run(sys.executable, str(ROOT / "tools/check_assets.py"), "--root", str(self.root), *extra)
        self.assertEqual(result.returncode, code)
        self.assertIn(diagnostic, result.stdout + result.stderr)

    def test_req_leg_02_registered_data_passes(self):
        self.manifest()
        self.check(0)

    def test_req_leg_02_unregistered_data_fails(self):
        (self.root / "assets/ASSETS.toml").write_text("asset = []\n")
        self.check(1, "unregistered")

    def test_req_leg_02_unregistered_public_file_fails(self):
        self.manifest()
        path = self.root / "client/public/unlisted.svg"
        path.parent.mkdir(parents=True)
        path.write_text("<svg/>")
        self.check(1, "unregistered")

    def test_req_leg_02_unregistered_localization_fails(self):
        self.manifest()
        path = self.root / "client/src/locales/ko.ftl"
        path.parent.mkdir(parents=True)
        path.write_text("title = test")
        self.check(1, "unregistered")

    def test_req_leg_02_unregistered_imported_asset_fails(self):
        self.manifest()
        path = self.root / "client/src/assets/logo.svg"
        path.parent.mkdir(parents=True)
        path.write_text("<svg/>")
        self.check(1, "unregistered")

    def test_req_leg_02_missing_metadata_fails(self):
        self.manifest(author="")
        self.check(1, "author")

    def test_req_leg_02_duplicate_fails(self):
        self.manifest()
        path = self.root / "assets/ASSETS.toml"
        path.write_text(path.read_text() * 2)
        self.check(1, "duplicate")

    def test_req_leg_02_missing_file_fails(self):
        self.manifest(path="assets/missing.png")
        self.check(1, "missing")

    def test_req_leg_02_traversal_fails(self):
        self.manifest(path="../outside.png")
        self.check(1, "relative")

    def test_req_leg_02_unapproved_asset_license_fails(self):
        self.manifest(license="CC-BY-NC-4.0")
        self.check(1, "license")

    def test_req_leg_02_boolean_string_fails(self):
        self.manifest(ai_generated="false")
        self.check(1, "boolean")

    def test_req_leg_02_ai_asset_release_fails(self):
        self.manifest(ai_generated=True)
        self.check(1, "AI-generated", "--release")

    def test_req_leg_02_ai_asset_dev_passes(self):
        self.manifest(ai_generated=True)
        self.check(0)

    def reviewed_fields(self):
        # Synthetic policy fixtures only; no generated image or real review.
        self.path = "assets/fixture.png"
        (self.root / self.path).write_bytes(b"synthetic image fixture")
        (self.root / "docs").mkdir(exist_ok=True)
        (self.root / "docs/terms.txt").write_text("Synthetic terms fixture", encoding="utf-8")
        (self.root / "docs/review.txt").write_text("Synthetic review fixture", encoding="utf-8")
        def digest(path):
            return hashlib.sha256((self.root / path).read_bytes()).hexdigest()
        return dict(ai_generated=True,
                    ai_provenance=dict(tool="Synthetic tool (fixture)",
                                       usage_terms_path="docs/terms.txt",
                                       usage_terms_sha256=digest("docs/terms.txt")),
                    distribution_review=dict(status="approved", reviewer="Fixture reviewer",
                                             reviewed_at="2026-10-06T09:00:00+09:00",
                                             evidence_path="docs/review.txt",
                                             evidence_sha256=digest("docs/review.txt"),
                                             asset_sha256=digest(self.path)))

    def test_req_leg_02_reviewed_ai_image_release_passes(self):
        self.manifest(**self.reviewed_fields())
        self.check(0, "", "--release")

    def test_req_leg_02_reviewed_ai_missing_fields_fail(self):
        for table in ("ai_provenance", "distribution_review"):
            fields = self.reviewed_fields()
            for key in list(fields[table]):
                with self.subTest(table=table, field=key):
                    fields = self.reviewed_fields()
                    del fields[table][key]
                    self.manifest(**fields)
                    self.check(1, key, "--release")
            fields = self.reviewed_fields()
            del fields[table]
            self.manifest(**fields)
            self.check(1, table, "--release")

    def test_req_leg_02_reviewed_ai_wrong_fields_fail(self):
        for table in ("ai_provenance", "distribution_review"):
            for key in self.reviewed_fields()[table]:
                for invalid in (False, "", "   ", 123, ["approved"]):
                    with self.subTest(table=table, field=key, invalid=invalid):
                        fields = self.reviewed_fields()
                        fields[table][key] = invalid
                        self.manifest(**fields)
                        self.check(1, key, "--release")
            for invalid in (False, "approved", ["approved"]):
                fields = self.reviewed_fields()
                fields[table] = invalid
                self.manifest(**fields)
                self.check(1, table, "--release")

    def test_req_leg_02_reviewed_ai_unapproved_status_fails(self):
        for status in ("pending", "rejected", "Approved", "false"):
            fields = self.reviewed_fields()
            fields["distribution_review"]["status"] = status
            self.manifest(**fields)
            self.check(1, "status", "--release")

    def test_req_leg_02_reviewed_ai_invalid_timestamp_fails(self):
        for timestamp in ("2026-10-06", "2026-10-06T09:00:00", "2026-02-30T09:00:00Z", "yesterday"):
            fields = self.reviewed_fields()
            fields["distribution_review"]["reviewed_at"] = timestamp
            self.manifest(**fields)
            self.check(1, "reviewed_at", "--release")

    def test_req_leg_02_reviewed_ai_reference_validation_fails(self):
        for table, key in (("ai_provenance", "usage_terms_path"), ("distribution_review", "evidence_path")):
            for path in ("docs/missing.txt", "docs", "../outside.txt", "C:/outside.txt", "docs\\review.txt", "docs/./review.txt"):
                fields = self.reviewed_fields()
                fields[table][key] = path
                self.manifest(**fields)
                self.check(1, key, "--release")
            fields = self.reviewed_fields()
            (self.root / fields[table][key]).write_bytes(b"")
            self.manifest(**fields)
            self.check(1, key, "--release")

    def test_req_leg_02_reviewed_ai_hash_mismatch_fails(self):
        for table, key in (("ai_provenance", "usage_terms_sha256"), ("distribution_review", "evidence_sha256"), ("distribution_review", "asset_sha256")):
            for digest in ("0" * 64, "invalid", "a" * 63):
                fields = self.reviewed_fields()
                fields[table][key] = digest
                self.manifest(**fields)
                self.check(1, key, "--release")

    def test_req_leg_02_reviewed_ai_changed_files_fail(self):
        for path, field in (("assets/fixture.png", "asset_sha256"), ("docs/terms.txt", "usage_terms_sha256"), ("docs/review.txt", "evidence_sha256")):
            self.manifest(**self.reviewed_fields())
            (self.root / path).write_bytes(b"changed after review")
            self.check(1, field, "--release")

    def test_req_leg_02_reviewed_ai_forbidden_license_fails(self):
        fields = self.reviewed_fields()
        self.manifest(**fields, license="CC-BY-NC-4.0")
        self.check(1, "license", "--release")

    def test_req_leg_02_reviewed_ai_nonimage_fails(self):
        fields = self.reviewed_fields()
        self.manifest(**fields, path="data/packs/demo/defines.toml")
        self.check(1, "image", "--release")

    def test_req_leg_02_nonai_release_and_font_checks_preserved(self):
        self.manifest()
        self.check(0, "", "--release")
        self.path = "assets/font.woff2"
        (self.root / self.path).write_bytes(b"synthetic font")
        self.manifest(license="CC0-1.0")
        self.check(1, "font license", "--release")
        self.manifest(license="OFL-1.1")
        self.check(0, "", "--release")


class NpmLicenseTests(unittest.TestCase):
    def fixture(self, license):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        root = Path(temp.name)
        (root / "package.json").write_text(json.dumps(dict(name="fixture-root", version="1.0.0", private=True, dependencies={"fixture-dep": "1.0.0"})))
        dep = root / "node_modules/fixture-dep"
        dep.mkdir(parents=True)
        metadata = dict(name="fixture-dep", version="1.0.0")
        if license is not None:
            metadata["license"] = license
        (dep / "package.json").write_text(json.dumps(metadata))
        (root / "package-lock.json").write_text(json.dumps({"lockfileVersion": 3, "packages": {"": {"name": "fixture-root"}, "node_modules/fixture-dep": metadata}}))
        return root

    def check(self, license, code):
        result = run("node", str(ROOT / "tools/check_npm_licenses.cjs"), "--root", str(self.fixture(license)))
        self.assertEqual(result.returncode, code)
        self.assertIn("license", result.stdout + result.stderr)
        if code:
            self.assertIn("fixture-dep", result.stdout + result.stderr)

    def test_req_leg_03_permitted_license_passes(self):
        self.check("MIT", 0)

    def test_req_leg_03_gpl_dependency_fails(self):
        self.check("GPL-3.0-only", 1)

    def test_req_leg_03_unknown_license_fails(self):
        self.check(None, 1)

    def test_req_leg_03_and_requires_both_licenses(self):
        self.check("MIT AND GPL-3.0-only", 1)

    def test_req_leg_03_or_accepts_permitted_choice(self):
        self.check("MIT OR Apache-2.0", 0)

    def test_req_leg_03_exception_not_silently_ignored(self):
        self.check("Apache-2.0 WITH LLVM-exception", 1)


class CargoLicenseTests(unittest.TestCase):
    def test_req_leg_03_cargo_gpl_and_bincode_fail(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("// test root\n")
            (root / "Cargo.toml").write_text('[package]\nname="fixture-root"\nversion="0.1.0"\nedition="2024"\npublish=false\n[dependencies]\ngpl-dep={path="gpl-dep",version="0.1.0"}\nbincode={path="bincode",version="0.1.0"}\n')
            for name, license in [("gpl-dep", "GPL-3.0-only"), ("bincode", "MIT")]:
                dep = root / name
                (dep / "src").mkdir(parents=True)
                (dep / "src/lib.rs").write_text("// violation fixture\n")
                (dep / "Cargo.toml").write_text(f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2024"\nlicense="{license}"\n')
            cargo = shutil.which("cargo") or str(Path.home() / ".cargo/bin/cargo.exe")
            result = run(cargo, "deny", "--manifest-path", str(root / "Cargo.toml"), "--config", str(ROOT / "deny.toml"), "check", "licenses", "bans")
            self.assertNotEqual(result.returncode, 0)
            for diagnostic in ["GPL-3.0-only", "bincode", "not explicitly allowed", "banned"]:
                self.assertIn(diagnostic, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
