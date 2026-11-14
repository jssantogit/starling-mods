"""Contract tests for native mod packaging, without game assets or Android."""
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from tools.package_hnmod import PackageError, package

HORIZON = "a" * 40
MIXINS = "b" * 40


class NativePackageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.mod = self.root / "candidate"
        bundle = self.mod / "mixins" / "candidate"
        bundle.mkdir(parents=True)
        (self.mod / "horizon-native.json").write_text(json.dumps({
            "schema": 1, "id": "candidate", "version": "0.1.0",
            "game": "26.3", "horizon": HORIZON, "mixins": MIXINS
        }), encoding="utf-8")
        (bundle / "horizon-mixin.toml").write_text(
            '[mod]\nid = "candidate"\nversion = "0.1.0"\n'
            f'[compatibility]\nhorizon_revision = "{HORIZON}"\n',
            encoding="utf-8")
        (bundle / "helper.rs").write_text("pub fn value() -> u8 { 1 }\n", encoding="utf-8")

    def test_valid_source_archive_has_expected_members(self):
        out = package(self.mod, self.root / "out.hnmod")
        with zipfile.ZipFile(out) as archive:
            self.assertEqual(archive.namelist(), [
                "horizon-native.json",
                "mixins/candidate/helper.rs",
                "mixins/candidate/horizon-mixin.toml",
            ])
            self.assertIn(b"pub fn value", archive.read("mixins/candidate/helper.rs"))

    def test_output_is_reproducible(self):
        one = package(self.mod, self.root / "one.hnmod").read_bytes()
        two = package(self.mod, self.root / "two.hnmod").read_bytes()
        self.assertEqual(one, two)

    def test_rejects_different_id(self):
        p = self.mod / "mixins/candidate/horizon-mixin.toml"
        p.write_text(p.read_text().replace('id = "candidate"', 'id = "other"'))
        with self.assertRaisesRegex(PackageError, "id"):
            package(self.mod, self.root / "bad.hnmod")

    def test_rejects_unpinned_source(self):
        p = self.mod / "horizon-native.json"
        p.write_text(p.read_text().replace(HORIZON, "main"))
        with self.assertRaisesRegex(PackageError, "horizon"):
            package(self.mod, self.root / "bad.hnmod")

    def test_rejects_unknown_files(self):
        (self.mod / "mixins/candidate/run.sh").write_text("echo surprise")
        with self.assertRaisesRegex(PackageError, "not allowed"):
            package(self.mod, self.root / "bad.hnmod")

    def test_rejects_symlinks(self):
        (self.mod / "mixins/candidate/alias.rs").symlink_to("helper.rs")
        with self.assertRaisesRegex(PackageError, "symlink"):
            package(self.mod, self.root / "bad.hnmod")

    def test_rejects_unknown_manifest_fields(self):
        p = self.mod / "horizon-native.json"
        value = json.loads(p.read_text())
        value["extra"] = True
        p.write_text(json.dumps(value))
        with self.assertRaisesRegex(PackageError, "fields"):
            package(self.mod, self.root / "bad.hnmod")

    def test_rejects_absent_mixin_manifest(self):
        (self.mod / "mixins/candidate/horizon-mixin.toml").unlink()
        with self.assertRaisesRegex(PackageError, "horizon-mixin.toml"):
            package(self.mod, self.root / "bad.hnmod")


if __name__ == "__main__":
    unittest.main()
