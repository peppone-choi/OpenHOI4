"""WP-01 requirement fixtures; invoke the production checkers as processes."""
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
            return str(v).lower() if isinstance(v, bool) else json.dumps(v)
        content = "[[asset]]\n" + "\n".join(f"{key} = {value(v)}" for key, v in fields.items())
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
