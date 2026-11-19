# Starling Mods — cumulative Android test APK (alpha.1)

**Status: experimental source integration, not a release.** The combined branch preserves the two existing independent native Rust ports and compiles both using Starling's official `native_build.sh SOURCE OUT BUNDLE...` interface. Source build revision: `09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2` (Minecraft 26.3). Native-mixins revision: `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`.

## What is actually included

1. [BadOptimizations (Thosea; MIT)](https://github.com/imthosea/BadOptimizations) `0.1.0-alpha.2`: lightmap short-circuit, skip preparation for an empty particle engine, and FOV fast path. Original source pinned at `5fa3416f6148eaa4045399cf7fa24a5427cc2991`.
2. [Greedy Meshing Mod (Sierra; MIT)](https://github.com/programmer1o1/GreedyMeshingMod) `0.1.0-alpha.2`: safe 2×2 merging of opaque full-cube axial quads in six directions. Original source pinned at `cafb82a28247dc2652bd11759d0075939d406617`.

**Not yet ported:** ImmediatelyFast, Particle Core and FerriteCore. See [candidate audit](CANDIDATES.md); these are not claimed as improvements nor included in the APK.

## Combined build

The [combined Android workflow](../../.github/workflows/build-starling-mods-combined.yml) builds both bundles into the *same native library*, checks the provenance manifest lists exactly both mods, verifies the library SHA-256 against the embedded APK copy, and checks the Android signature and distinct package `io.github.juanluci.starling.mods` with folder `StarlingMods`.

The experimental app is independent of stock Starling, Starling Dev (BadOptimizations), and Starling Greedy. `PROFILE=quick` is held fixed for this alpha build; a fair baseline requires identical Starling commit, profile, Android settings and scene.

Run the local tests:

```sh
python3 -m unittest discover -s tests -v
cargo test --manifest-path tests/rust/greedymeshing-core/Cargo.toml
cargo test --manifest-path tests/rust/lightmap-cache/Cargo.toml
cargo test --manifest-path tests/rust/fov-fastpath/Cargo.toml
```

## Test gate

Use a disposable world, default resource pack, and **no shader packs**, including oldLighting. Greedy's shader fallback is unverified. Confirm visual parity before performance claims, and compare GPU frame time, median FPS, 1% lows, memory, and temperature over repeated same-path trials. A signed CI APK does not imply successful boot or gain.
