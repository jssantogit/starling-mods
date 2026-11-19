import tempfile
import unittest
from pathlib import Path

from tools.patch_mods_variant import APK_REL, JAVA_REL, isolated_mods_variant


class ModsVariantPatchTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name)
        self.apk = self.root / APK_REL
        self.java = self.root / JAVA_REL
        self.apk.parent.mkdir(parents=True)
        self.java.parent.mkdir(parents=True)
        self.apk.write_text(
            'DEV_SUFFIX = ".dev"\n'
            'VARIANTS = {"dev": (DEV_SUFFIX, " Dev"), "preview": (".preview", " Preview")}\n',
            encoding="utf-8",
        )
        self.java.write_text(
            'String name = id.endsWith(".dev") ? "StarlingDev"\n'
            '                : id.endsWith(".preview") ? "StarlingPreview" : "Starling";\n',
            encoding="utf-8",
        )

    def test_mods_app_has_independent_package_and_folder(self):
        isolated_mods_variant(self.root)
        self.assertIn('DEV_SUFFIX = ".mods"', self.apk.read_text())
        self.assertIn('"dev": (DEV_SUFFIX, " Mods")', self.apk.read_text())
        self.assertIn('id.endsWith(".mods") ? "StarlingMods"', self.java.read_text())
        self.assertIn('id.endsWith(".dev") ? "StarlingDev"', self.java.read_text())
        self.assertIn('id.endsWith(".preview") ? "StarlingPreview"', self.java.read_text())

    def test_unknown_upstream_java_is_not_partially_modified(self):
        self.java.write_text("upstream changed", encoding="utf-8")
        before = self.apk.read_bytes()
        with self.assertRaises(ValueError):
            isolated_mods_variant(self.root)
        self.assertEqual(self.apk.read_bytes(), before)

    def test_refuses_double_patching(self):
        isolated_mods_variant(self.root)
        with self.assertRaises(ValueError):
            isolated_mods_variant(self.root)


if __name__ == "__main__":
    unittest.main()
