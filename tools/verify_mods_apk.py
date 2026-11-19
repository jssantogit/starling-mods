"""Check experimental Mods APK provenance and exact embedded native library."""
import argparse
import hashlib
import json
import re
import zipfile
from pathlib import Path

COMMIT = "09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2"
APK_NAME = "starling-dev-0.8.0-arm64-v8a.apk"
LIBRARY = "lib/arm64-v8a/libhorizon_android.so"


def verify(manifest: Path, library: Path, apk: Path):
    data = json.loads(manifest.read_text(encoding="utf-8"))
    assert data.get("commit") == COMMIT, "Wrong Starling source revision"
    assert data.get("abi") == "arm64-v8a", "Wrong architecture"
    assert data.get("version") == "0.8.0", "Wrong Starling version"
    assert data.get("mods") == [
        {"id": "badoptimizations", "version": "0.1.0-alpha.2"},
        {"id": "greedymeshing", "version": "0.1.0-alpha.2"},
    ], "Expected both native mods"
    assert apk.name == APK_NAME, "Wrong APK variant/filename"
    checksum = hashlib.sha256(library.read_bytes()).hexdigest()
    assert re.fullmatch(r"[0-9a-f]{64}", checksum), "Invalid SHA-256"
    assert data["sha256"] == checksum and data["size"] == library.stat().st_size, "Library mismatch"
    with zipfile.ZipFile(apk) as z:
        assert "AndroidManifest.xml" in z.namelist() and "classes.dex" in z.namelist()
        assert LIBRARY in z.namelist(), "ARM64 native library missing"
        assert hashlib.sha256(z.read(LIBRARY)).hexdigest() == checksum, "APK library differs"
    print("Verified native Mods APK", checksum)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--apk", type=Path, required=True)
    opts = parser.parse_args()
    verify(opts.manifest, opts.library, opts.apk)
