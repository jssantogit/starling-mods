# BadOptimizations → Starling (26.3)

**Status: experimental native *source prototype*. All combined mixins (lightmap, particles and FOV) passed plan/apply and Linux plus Android ARM64 Cargo checks in [CI](https://github.com/jssantogit/starling-mods/actions/runs/43618551353). A final Android library build, launch/gameplay test and performance benchmarks have NOT been completed. Do not publish it as a working optimization yet.**

## Provenance

- Original mod: [BadOptimizations](https://github.com/imthosea/BadOptimizations) by Thosea, `oldsrc/26.3` at [`5fa3416`](https://github.com/imthosea/BadOptimizations/tree/5fa3416f6148eaa4045399cf7fa24a5427cc2991) (the published 26.3 source rather than the experimental `main`). Original license at this revision: MIT, Copyright 2025 Thosea.
- Starling source: [`09fa16c`](https://github.com/starling-client/starling/tree/09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2) (Minecraft 26.3).
- `horizon-mixins` tool: `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`, pinned by Starling's `packaging/android/native_build.sh` at the chosen revision.
- The Rust implementation under `native/mixins/badoptimizations` is original source inspired by the original mod's technique; no Java source is copied. See `NOTICE.md` in the bundle.

## Implemented scope in this prototype

The original Java mod's lightmap optimization avoids redundant texture computation. Starling *already* skips unchanged GPU uploads in `crates/horizon-client-render/src/gfx.rs`, `Renderer::update_lightmap`. But it still recalculates all 256 pixels before comparing them each render call.

The prototype injects a cancellable `HEAD` into `Renderer::update_lightmap`, with a `RETURN` observer, to skip the original CPU calculation **only when the last-seen renderer identity, cache key and output bytes match what was previously computed and uploaded**. The cache is thread-local and remembers the last renderer identity. This avoids modifying the asynchronous `Renderer::new` constructor (unsupported by the current mixin state-injection mechanism) and prevents reuse across renderers. This intentionally does **not** change the GPU-upload logic.

Key inputs are every value read by the current method: environment's ambient colour (RGB), sky light colour (RGB), block light tint (RGB), sky light factor, darkness, gamma, night vision, flicker, boss darkening and `fullbright`. Comparison uses exact float bits. The first render, device reset, changed inputs, changed stored pixels and an uninitialized texture always execute the original code.

**Also implemented in this combined prototype:** an early return for particle extraction when the particle engine is empty (no quaternions/light cache created), and a fast path for zero FOV-effect scale that preserves the first-person spyglass modifier. **Still excluded or already native:** inactive debug rendering, idle toast work, entity renderer indirections and Java-only microoptimizations. Sky-colour caching from Minecraft 1.21.10 and below is obsolete for data-driven 26.3. Java mod FPS measurements do not predict Starling performance. See [the feature-by-feature port matrix](PORT_MATRIX.md) for the remaining audit scope.

## Layout

- `native/horizon-native.json`: pinned native bundle metadata.
- `native/mixins/badoptimizations/horizon-mixin.toml`: typed source targets, thread-local cache and HEAD/RETURN hooks.
- `native/mixins/badoptimizations/runtime.rs`: independent cache implementation and 9 Rust unit tests.
- `native/mixins/badoptimizations/{before,after}.rs`: adapters to Starling's renderer.
- `native/mixins/badoptimizations/particle_before.rs`: empty particle-extraction guard.
- `native/mixins/badoptimizations/{fov_before,fov_runtime}.rs`: FOV-effect fast path and pure logic tests.
- `tests/rust/lightmap-cache/Cargo.toml` and `tests/rust/fov-fastpath/Cargo.toml`: standalone Cargo test harnesses, each compiling the **same** Rust file used by the mixins.
- `tests/test_badoptimizations.py`: Python metadata, wiring and package tests.

## Checks

```bash
python -m unittest discover -s tests -v
cargo test --manifest-path tests/rust/lightmap-cache/Cargo.toml --offline
cargo test --manifest-path tests/rust/fov-fastpath/Cargo.toml --offline
python tools/package_hnmod.py mods/badoptimizations/native --output dist/badoptimizations-0.1.0-alpha.2.hnmod
```

Passing standalone tests and ZIP validation does **not** prove the Rust hooks resolve in Starling, that the full Android library compiles or that gameplay visuals stay identical. The source and mixin tool require a real `horizon-mixins plan/apply/check` against the exact Starling revision and a native ARM64 build.

## Still required before a release

- [x] Identify original source revision, license, and concrete Starling target.
- [x] Implement first source-level caching algorithm and isolated cache/packaging tests.
- [x] Validate Horizon Mixins plan/apply and staged Rust checks on Linux + Android ARM64 for Starling `09fa16c` ([combined CI](https://github.com/jssantogit/starling-mods/actions/runs/43618551353)).
- [ ] Compile through Starling's ARM64 native-build path, boot and reload graphics.
- [ ] Parity-test day/night, dimensions, underwater, gamma, darkness/night vision, other mod hooks and shaders.
- [ ] Measure CPU work saved, median/P99 frame times, memory use and thermal behavior against the pinned stock build.

**Compatibility warning:** a source-mixin against `Renderer::update_lightmap` is version-sensitive and may interfere with other mixins touching the same method. A later Starling version must be re-audited and repinned. Native mods execute with full Android app privileges; only use trusted code.
