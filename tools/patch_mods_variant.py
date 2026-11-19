"""Prepare an isolated Starling Mods test-APK identity in the disposable build.

Never modifies the pinned Starling Git checkout or the game library. The
upstream source copy has already been built through native_build.sh.
"""
import argparse
from pathlib import Path

APK_REL = "packaging/android/build_apk.py"
JAVA_REL = "packaging/android/java/io/github/juanluci/starling/StarlingActivity.java"


def one(text: str, old: str, new: str, path: str) -> str:
    if text.count(old) != 1:
        raise ValueError(f"{path}: expected exactly one upstream marker: {old!r}")
    return text.replace(old, new, 1)


def isolated_mods_variant(root: Path) -> None:
    apk = root / APK_REL
    java = root / JAVA_REL
    apk_source = apk.read_text(encoding="utf-8")
    java_source = java.read_text(encoding="utf-8")

    updated_apk = one(apk_source, 'DEV_SUFFIX = ".dev"', 'DEV_SUFFIX = ".mods"', APK_REL)
    updated_apk = one(
        updated_apk, '"dev": (DEV_SUFFIX, " Dev")', '"dev": (DEV_SUFFIX, " Mods")', APK_REL
    )
    updated_java = one(
        java_source,
        'String name = id.endsWith(".dev") ? "StarlingDev"',
        'String name = id.endsWith(".mods") ? "StarlingMods"\n'
        '                : id.endsWith(".dev") ? "StarlingDev"',
        JAVA_REL,
    )
    # Only write once all exact markers have been found in both files.
    apk.write_text(updated_apk, encoding="utf-8")
    java.write_text(updated_java, encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_directory", type=Path)
    args = parser.parse_args()
    isolated_mods_variant(args.build_directory)
    print("Test app isolated: io.github.juanluci.starling.mods / StarlingMods")
