"""Tests for our local Android artifact validator and the build workflow."""
import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from tools.verify_starling_apk import (
    ABI, LIBRARY_IN_APK, MOD_ID, MOD_VERSION, STARLING_REVISION,
    STARLING_VERSION, BuildVerificationError, check_build,
)

ROOT = Path(__file__).resolve().parents[1]


class NativeApkTests(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        root = Path(tmp.name)
        self.manifest = root / "build.json"
        self.library = root / "libhorizon_android.so"
        self.apk = root / f"starling-dev-{STARLING_VERSION}-{ABI}.apk"
        self.blob = b"\x7fELF" + b"test"
        self.library.write_bytes(self.blob)
        self.data = {
            "commit": STARLING_REVISION, "version": STARLING_VERSION, "abi": ABI,
            "mods": [{"id": MOD_ID, "version": MOD_VERSION}],
            "sha256": hashlib.sha256(self.blob).hexdigest(), "size": len(self.blob),
        }
        self.regenerate()

    def regenerate(self, embedded=None, omit=False):
        self.manifest.write_text(json.dumps(self.data), encoding="utf-8")
        with zipfile.ZipFile(self.apk, "w") as archive:
            archive.writestr("AndroidManifest.xml", b"manifest")
            archive.writestr("classes.dex", b"dex")
            if not omit:
                archive.writestr(LIBRARY_IN_APK, embedded if embedded is not None else self.blob)

    def test_valid_build(self):
        self.assertEqual(check_build(self.manifest, self.library, self.apk), self.data)

    def test_rejects_wrong_source(self):
        self.data["commit"] = "b" * 40
        self.regenerate()
        with self.assertRaisesRegex(BuildVerificationError, "commit"):
            check_build(self.manifest, self.library, self.apk)

    def test_rejects_missing_mod(self):
        self.data["mods"] = []
        self.regenerate()
        with self.assertRaisesRegex(BuildVerificationError, "BadOptimizations"):
            check_build(self.manifest, self.library, self.apk)

    def test_rejects_tampered_library(self):
        self.library.write_bytes(b"tampered")
        with self.assertRaisesRegex(BuildVerificationError, "hash/size"):
            check_build(self.manifest, self.library, self.apk)

    def test_rejects_different_embedded_library(self):
        self.regenerate(embedded=b"different")
        with self.assertRaisesRegex(BuildVerificationError, "different native library"):
            check_build(self.manifest, self.library, self.apk)

    def test_rejects_missing_apk_library(self):
        self.regenerate(omit=True)
        with self.assertRaisesRegex(BuildVerificationError, "APK missing"):
            check_build(self.manifest, self.library, self.apk)

    def test_does_not_accept_preview_apk(self):
        wrong = self.apk.with_name(self.apk.name.replace("-dev-", "-preview-"))
        self.apk.rename(wrong)
        with self.assertRaisesRegex(BuildVerificationError, "separate Dev APK"):
            check_build(self.manifest, self.library, wrong)

    def test_uses_starling_official_source_build_script(self):
        contents = (ROOT / ".github/workflows/build-starling-dev-apk.yml").read_text()
        self.assertIn(STARLING_REVISION, contents)
        self.assertIn("packaging/android/native_build.sh", contents)
        self.assertIn("VARIANT: --dev", contents)
        self.assertIn("PROFILE: quick", contents)
        self.assertIn("persist-credentials: false", contents)
        self.assertIn("starling-dev-badoptimizations-arm64", contents)


if __name__ == "__main__":
    unittest.main()
