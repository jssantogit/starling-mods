"""Prevent CI regressions that omit Android's required native C compiler."""
import unittest
from pathlib import Path

WORKFLOW = Path(__file__).resolve().parents[1] / ".github/workflows/badoptimizations-integration.yml"


class AndroidCrossCompileWorkflowTests(unittest.TestCase):
    def test_pinned_ndk_is_installed_before_android_check(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("ndk;28.2.13676358", workflow)
        self.assertIn("aarch64-linux-android26-clang", workflow)
        self.assertLess(
            workflow.index("ndk;28.2.13676358"),
            workflow.index("Cross-check renderer for Android ARM64"),
        )

    def test_uses_ndk_wrapper_and_target_android_8(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("cargo-ndk --version 3.5.4 --locked", workflow)
        self.assertIn("ANDROID_NDK_HOME=", workflow)
        self.assertIn("cargo ndk -t arm64-v8a --platform 26 check --locked", workflow)


    def test_cargo_ndk_discovers_staged_cargo_workspace(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn("working-directory: target/badoptimizations-stage", workflow)
        self.assertNotIn(
            "--manifest-path target/badoptimizations-stage/Cargo.toml \\",
            workflow.split("- name: Cross-check renderer for Android ARM64 (no APK)")[1],
        )

    def test_android_check_uses_real_app_features(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        step = workflow.split("- name: Cross-check renderer for Android ARM64 (no APK)")[1]
        self.assertIn("-p horizon-android --no-default-features", step)
        self.assertIn("ANDROID_NDK_ROOT=", step)

if __name__ == "__main__":
    unittest.main()
