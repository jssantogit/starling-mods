#!/usr/bin/env python3
"""Validate and reproducibly package Starling/Horizon native source mods (.hnmod).

This packages Rust source bundles, NOT compiled game libraries; installation
and compilation require Starling's native mod build pipeline and user approval.
"""
import argparse
import json
import re
import stat
import tomllib
import zipfile
from pathlib import Path


class PackageError(ValueError):
    """A source directory violates the native mod package contract."""


_REV = re.compile(r"[0-9a-f]{40}\Z")
_ID = re.compile(r"[a-z0-9_]{1,64}\Z")
_ALLOWED = {"schema", "id", "version", "game", "horizon", "mixins", "hooks"}
_STAMP = (1980, 1, 1, 0, 0, 0)


def _manifest(root: Path) -> dict:
    try:
        info = json.loads((root / "horizon-native.json").read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise PackageError(f"horizon-native.json: {exc}") from exc
    if not isinstance(info, dict) or set(info) - _ALLOWED:
        raise PackageError("horizon-native.json has unrecognized fields or is not an object")
    if type(info.get("schema")) is not int or info["schema"] != 1:
        raise PackageError("schema must be 1")
    mod_id = info.get("id")
    if not isinstance(mod_id, str) or not _ID.fullmatch(mod_id):
        raise PackageError("id must match [a-z0-9_]{1,64}")
    for name in ("version", "game"):
        if not isinstance(info.get(name), str) or not info[name].strip():
            raise PackageError(f"{name} must be a nonempty string")
    for name in ("horizon", "mixins"):
        value = info.get(name)
        if not isinstance(value, str) or not _REV.fullmatch(value):
            raise PackageError(f"{name} must be a 40-character lowercase SHA")
    if "hooks" in info and (type(info["hooks"]) is not int or info["hooks"] < 1):
        raise PackageError("hooks must be a positive integer")
    descriptor = root / "mixins" / mod_id / "horizon-mixin.toml"
    if not descriptor.is_file() or descriptor.is_symlink():
        raise PackageError(f"missing or unsafe mixins/{mod_id}/horizon-mixin.toml")
    try:
        mixin = tomllib.loads(descriptor.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise PackageError(f"horizon-mixin.toml: {exc}") from exc
    if mixin.get("mod", {}).get("id") != mod_id:
        raise PackageError("mixin mod id differs from horizon-native.json id")
    if mixin.get("mod", {}).get("version") != info["version"]:
        raise PackageError("mixin version differs from horizon-native.json version")
    if mixin.get("compatibility", {}).get("horizon_revision") != info["horizon"]:
        raise PackageError("mixin compatibility horizon_revision differs from horizon-native.json horizon")
    return info


def _allowed_source(rel: Path) -> bool:
    parts = rel.parts
    if len(parts) < 3 or parts[0] != "mixins":
        return False
    within = parts[2:]
    if rel.suffix in (".rs", ".toml"):
        return True
    if "data" in within[:-1] and len(within) >= 2:
        return True
    name = within[-1]
    if name == ".gitignore" or any(name.startswith(p) for p in (
        "README", "LICENSE", "LICENCE", "COPYING", "NOTICE"
    )):
        return len(within) in (1, 2)
    return False


def package(root: Path, output: Path) -> Path:
    """Return deterministic .hnmod bytes after validating native source."""
    root, output = Path(root), Path(output)
    if not root.is_dir() or root.is_symlink():
        raise PackageError("mod root must be a real directory")
    if output.suffix != ".hnmod":
        raise PackageError("output extension must be .hnmod")
    if output.resolve().is_relative_to(root.resolve()):
        raise PackageError("output must be outside the source directory")
    info = _manifest(root)
    paths = [root / "horizon-native.json"]
    bundle = root / "mixins" / info["id"]
    if bundle.is_symlink():
        raise PackageError("symlink source directories are not allowed")
    for file in sorted(root.rglob("*")):
        if file.is_symlink():
            raise PackageError(f"symlink is not allowed: {file.relative_to(root)}")
        if file.is_dir():
            continue
        if not file.is_file():
            raise PackageError(f"nonregular file is not allowed: {file.relative_to(root)}")
        rel = file.relative_to(root)
        if rel == Path("horizon-native.json"):
            continue
        if rel.parts[:2] != ("mixins", info["id"]) or not _allowed_source(rel):
            raise PackageError(f"source path is not allowed: {rel}")
        paths.append(file)
    if len(paths) > 512:
        raise PackageError("native source has more than 512 files")
    if sum(p.stat().st_size for p in paths) > 64 * 1024 * 1024:
        raise PackageError("native source exceeds 64 MiB unpacked")
    output.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for file in sorted(paths, key=lambda p: p.relative_to(root).as_posix()):
            entry = zipfile.ZipInfo(file.relative_to(root).as_posix(), _STAMP)
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = (stat.S_IFREG | 0o644) << 16
            archive.writestr(entry, file.read_bytes(), compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return output


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        built = package(args.source, args.output)
    except PackageError as exc:
        parser.error(str(exc))
    print(f"Created {built} (source bundle only; native build not verified)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
