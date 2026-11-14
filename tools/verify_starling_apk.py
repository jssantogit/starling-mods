"""Verify a signed Starling Dev APK includes the exact native mod library.

Signature and Android package name are checked separately by the GitHub workflow.
"""
import argparse
import hashlib
import json
import re
import zipfile
from pathlib import Path

STARLING_REVISION = "09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2"
STARLING_VERSION = "0.8.0"
MOD_ID = "badoptimizations"
MOD_VERSION = "0.1.0-alpha.2"
ABI = "arm64-v8a"
LIBRARY_IN_APK = f"lib/{ABI}/libhorizon_android.so"


class BuildVerificationError(ValueError):
    pass


def check_build(manifest: Path, library: Path, apk: Path) -> dict:
    manifest, library, apk = map(Path, (manifest, library, apk))
    try:
        data = json.loads(manifest.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise BuildVerificationError(f"Invalid build.json: {exc}") from exc
    if not isinstance(data, dict):
        raise BuildVerificationError("Build manifest must be an object")
    for field, expected in (
        ("commit", STARLING_REVISION), ("version", STARLING_VERSION), ("abi", ABI)
    ):
        if data.get(field) != expected:
            raise BuildVerificationError(f"{field} does not match the pinned game")
    mods = data.get("mods")
    if not isinstance(mods, list) or mods != [{"id": MOD_ID, "version": MOD_VERSION}]:
        raise BuildVerificationError("Expected exactly the BadOptimizations alpha.2 mod")
    name = f"starling-dev-{STARLING_VERSION}-{ABI}.apk"
    if apk.name != name:
        raise BuildVerificationError(f"Expected separate Dev APK named {name}")
    sha = data.get("sha256")
    if not isinstance(sha, str) or not re.fullmatch(r"[0-9a-f]{64}", sha):
        raise BuildVerificationError("Invalid native library digest")
    try:
        content = library.read_bytes()
    except OSError as exc:
        raise BuildVerificationError(f"Missing native library: {exc}") from exc
    digest = hashlib.sha256(content).hexdigest()
    if digest != sha or len(content) != data.get("size"):
        raise BuildVerificationError("Native library hash/size differs from build.json")
    try:
        with zipfile.ZipFile(apk) as archive:
            for name in ("AndroidManifest.xml", "classes.dex", LIBRARY_IN_APK):
                if name not in archive.namelist():
                    raise BuildVerificationError(f"APK missing {name}")
            if hashlib.sha256(archive.read(LIBRARY_IN_APK)).hexdigest() != digest:
                raise BuildVerificationError("APK contains a different native library")
    except (OSError, zipfile.BadZipFile) as exc:
        raise BuildVerificationError(f"Invalid APK: {exc}") from exc
    return data


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--apk", type=Path, required=True)
    args = parser.parse_args()
    try:
        data = check_build(args.manifest, args.library, args.apk)
    except BuildVerificationError as exc:
        parser.error(str(exc))
    print("Verified Dev APK", data["commit"][:12], data["mods"][0]["version"], data["sha256"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
