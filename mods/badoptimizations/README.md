# BadOptimizations → Starling (26.3)

**Status: experimental native *source prototype*. Not verified by Horizon Mixins' full Starling build and NOT tested in game on Android. Do not publish as a working optimization or recommend installing it yet.**

## Provenance

- Original mod: [BadOptimizations](https://github.com/imthosea/BadOptimizations) by Thosea, `oldsrc/26.3` at [`5fa3416`](https://github.com/imthosea/BadOptimizations/tree/5fa3416f6148eaa4045399cf7fa24a5427cc2991) (the published 26.3 source rather than the experimental `main`). Original license at this revision: MIT, Copyright 2025 Thosea.
- Starling source: [`09fa16c`](https://github.com/starling-client/starling/tree/09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2) (Minecraft 26.3).
- `horizon-mixins` tool: `ddd9700b21997c95d95ac6ee5ecafe3830cf622a`, pinned by Starling's `packaging/android/native_build.sh` at the chosen revision.
- The Rust implementation under `native/mixins/badoptimizations` is original source inspired by the original mod's technique; no Java source is copied. See `NOTICE.md` in the bundle.

## Implemented scope in this prototype

The original Java mod's lightmap optimization avoids redundant texture computation. Starling *already* skips unchanged GPU uploads in `crates/horizon-client-render/src/gfx.rs`, `Renderer::update_lightmap`. But it still recalculates all 256 pixels before comparing them each render call.

The prototype injects a cancellable `HEAD` into `Renderer::update_lightmap`, with a `RETURN` observer, to skip the original CPU calculation **only when the renderer instance's cache key and output bytes match what was previously computed and uploaded**. The per-renderer cache is installed through Horizon Mixins' `[[state]]`, avoiding process-global state. This intentionally does **not** change the GPU-upload logic.

Key inputs are every value read by the current method: environment's ambient colour (RGB), sky light colour (RGB), block light tint (RGB), sky light factor, darkness, gamma, night vision, flicker, boss darkening and `fullbright`. Comparison uses exact float bits. The first render, device reset, changed inputs, changed stored pixels and an uninitialized texture always execute the original code.

**Not yet ported:** particle extraction, debug renderer, toast and entity microoptimizations. Sky-colour caching from Minecraft 1.21.10 and below is obsolete for data-driven 26.3. Java mod FPS measurements do not predict Starling performance.

## Layout

- `native/horizon-native.json`: pinned native bundle metadata.
- `native/mixins/badoptimizations/horizon-mixin.toml`: typed source targets, renderer state and HEAD/RETURN hooks.
- `native/mixins/badoptimizations/runtime.rs`: independent cache implementation and 7 Rust unit tests.
- `native/mixins/badoptimizations/{before,after}.rs`: adapters to Starling's renderer.
- `tests/rust/lightmap-cache/Cargo.toml`: standalone Cargo test harness, using the **same** `runtime.rs` included by the mixin.
- `tests/test_badoptimizations.py`: Python metadata, wiring and package tests.

## Checks

```bash
python -m unittest discover -s tests -v
cargo test --manifest-path tests/rust/lightmap-cache/Cargo.toml --offline
python tools/package_hnmod.py mods/badoptimizations/native --output dist/badoptimizations-0.1.0-alpha.1.hnmod
```

Passing standalone tests and ZIP validation does **not** prove the Rust hooks resolve in Starling, that the full Android library compiles or that gameplay visuals stay identical. The source and mixin tool require a real `horizon-mixins plan/apply/check` against the exact Starling revision and a native ARM64 build.

## Still required before a release

- [x] Identify original source revision, license, and concrete Starling target.
- [x] Implement first source-level caching algorithm and isolated cache/packaging tests.
- [ ] Validate the actual Horizon Mixins plan and staged Rust compilation against Starling `09fa16c`.
- [ ] Compile through Starling's ARM64 native-build path, boot and reload graphics.
- [ ] Parity-test day/night, dimensions, underwater, gamma, darkness/night vision, other mod hooks and shaders.
- [ ] Measure CPU work saved, median/P99 frame times, memory use and thermal behavior against the pinned stock build.

**Compatibility warning:** a source-mixin against `Renderer::update_lightmap` is version-sensitive and may interfere with other mixins touching the same method. A later Starling version must be re-audited and repinned. Native mods execute with full Android app privileges; only use trusted code.
