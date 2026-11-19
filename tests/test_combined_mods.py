"""Regression checks: both native source ports must ship in the same APK."""
import hashlib
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

from tools.verify_mods_apk import verify, COMMIT, APK_NAME, LIBRARY

ROOT = Path(__file__).resolve().parents[1]


class CombinedModTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.library = self.root / "libhorizon_android.so"
        self.blob = b"\\x7fELF" + b"combined-native-ports"
        self.library.write_bytes(self.blob)
        self.apk = self.root / APK_NAME
        self.manifest = self.root / "build.json"
        self.data = {
            "commit": COMMIT, "abi": "arm64-v8a", "version": "0.8.0",
            "mods": [
                {"id": "badoptimizations", "version": "0.1.0-alpha.2"},
                {"id": "greedymeshing", "version": "0.1.0-alpha.2"},
            ],
            "sha256": hashlib.sha256(self.blob).hexdigest(),
            "size": len(self.blob),
        }
        self.write()

    def write(self, embedded=None):
        self.manifest.write_text(json.dumps(self.data))
        with zipfile.ZipFile(self.apk, "w") as z:
            z.writestr("AndroidManifest.xml", b"manifest")
            z.writestr("classes.dex", b"dex")
            z.writestr(LIBRARY, self.blob if embedded is None else embedded)

    def test_accepts_exact_combination(self):
        verify(self.manifest, self.library, self.apk)

    def test_rejects_mod_missing(self):
        self.data["mods"].pop()
        self.write()
        with self.assertRaisesRegex(AssertionError, "both native mods"):
            verify(self.manifest, self.library, self.apk)

    def test_rejects_different_native_library(self):
        self.write(embedded=b"tampered")
        with self.assertRaisesRegex(AssertionError, "differs"):
            verify(self.manifest, self.library, self.apk)

    def test_rejects_wrong_game_revision(self):
        self.data["commit"] = "0" * 40
        self.write()
        with self.assertRaisesRegex(AssertionError, "revision"):
            verify(self.manifest, self.library, self.apk)

    def test_workflow_passes_both_source_bundle_directories(self):
        w = (ROOT / ".github/workflows/build-starling-mods-combined.yml").read_text()
        self.assertIn("native_build.sh", w)
        self.assertIn("native/mixins/badoptimizations", w)
        self.assertIn("native/mixins/greedymeshing", w)
        self.assertIn("PROFILE: quick", w)
        self.assertIn("starling-mods-combined-alpha1-arm64", w)
        self.assertIn("io.github.juanluci.starling.mods", w)


if __name__ == "__main__":
    unittest.main()
