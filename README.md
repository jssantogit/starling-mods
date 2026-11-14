# Starling Mods

Community workspace for **independent performance-mod adaptations for [Starling](https://github.com/starling-client/starling)**, the Android port of Horizon. The canonical source for all mods we develop here is this repository.

**Status: infrastructure established; no performance mod has been ported or released yet.** A candidate directory is not an installable mod. Do not import its files into the game.

## Catalog

| Mod / inspiration | Scope | Status |
| --- | --- | --- |
| [BadOptimizations](mods/badoptimizations/README.md) | Investigate avoiding redundant lightmap updates, idle debug work | **Research** (no source bundle) |
| Gnetum | Check HUD update costs against existing Starling code | Backlog |
| Particle Core | Investigate particle update overhead beyond existing culling | Backlog |
| C2ME | Evaluate chunk task scheduling (large, engine-level changes) | Backlog |
| Lithium | Evaluate simulation, collisions and entity updates | Backlog |

## How a mod becomes installable

1. Audit the *current Starling implementation* and the original mod's behavior/license.
2. Identify a demonstrably missing optimization and establish a baseline measurement.
3. Implement a Rust source patch using [horizon-mixins](https://github.com/horizon-rust/horizon-mixins), keeping changes as narrow as possible.
4. Test behavior equivalence, benchmark and compatibility against an **exact Starling source revision**.
5. Package the verified source as `.hnmod`; on ARM64 Starling recompiles the game with enabled mods after user approval. Publishing a ZIP alone does not prove the mod runs.

A `.jar` from Modrinth cannot run in Starling. `.hmod` is for content/data, not performance code. Native `.hnmod` files contain source patches and may access everything the game can access on the phone.

## Packaging native sources

After a mod has a working `horizon-native.json` and `mixins/<id>/horizon-mixin.toml`, validate and package it with Python 3.11+:

```sh
python3 -m unittest discover -s tests -v
python3 tools/package_hnmod.py mods/<id> --output dist/<id>.hnmod
```

The packager checks IDs, exact pinned source revisions, mixin manifest agreement, allowed paths, source limits and symlinks; it creates reproducible ZIP bytes. **It does not compile Rust, apply injections or verify performance**, so native compilation/gameplay tests remain mandatory before a release.

Documentation: [porting policy](docs/PORTING.md) · [benchmark protocol](docs/BENCHMARKS.md) · [first candidate](mods/badoptimizations/README.md).

## Provenance and licensing

This repository is MIT-licensed (see [LICENSE](LICENSE)). Every port retains the original mod's attribution and complies with the original license; source references and changes belong in each mod's README. An adaptation is not endorsed by the original mod author, Starling, Mojang or Microsoft. No Minecraft proprietary resources or third-party jars are committed here.