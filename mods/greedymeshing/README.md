# Greedy Meshing — Rust source adaptation (feasibility prototype)

**Status: experimental native source adapter. The 16-test Rust core and `horizon-mixins plan/apply` + patched renderer `cargo check` on Linux have passed in [integration CI](https://github.com/jssantogit/starling-mods/actions/runs/43633951718). Android ARM64 app `cargo check` has also [passed](https://github.com/jssantogit/starling-mods/actions/runs/43634253942). A linked APK, gameplay visual parity and benchmarks remain unverified. Do not install this as a release.**

- Original mod: [Greedy Meshing Mod](https://github.com/programmer1o1/GreedyMeshingMod) by Sierra, MIT, pinned at `cafb82a28247dc2652bd11759d0075939d406617` (release 0.6.1, supports Minecraft 26.3).
- Target: [Starling](https://github.com/starling-client/starling) at `09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2` (Minecraft 26.3).
- Our `core.rs` adapts the original's **plane-by-plane greedy sweep concept** into standalone Rust code over precomputed material/visibility keys. The native source now adds a post-meshing RETURN hook on `Mesher<'_>::mesh_reusing` and a conservative **top-face-only 2×2 adapter** using the existing near-section UV encoding. This is a new Rust implementation, not the Java/Fabric code.
- 16 Rust tests assert no disappearing/duplicated faces, no overlap, no merging of differing keys, 16x16 boundaries, handling all 256 unique face keys, bounded 2×2 rectangles, canonical UVs, mirrored/rotated textures and safe fallback.

## What we learned

See [AUDIT.md](AUDIT.md) for the source-level analysis and integration gates. Starling already merges faces for distant LOD terrain, but detailed nearby sections still follow baked models and per-block quads. This is a plausible gap for the original mod's technique, provided we adapt sprite tiling and preserve lighting and resource-pack behavior.

Run locally with Rust 1.95+:

```sh
cargo test --offline --manifest-path tests/rust/greedymeshing-core/Cargo.toml
```

The [integration workflow](../../.github/workflows/greedymeshing-integration.yml) packages the source `.hnmod` as a **diagnostic artifact only**, applies the actual source hook and checks compilation. A successful compiler check does not prove the source mod runs or increases FPS; it needs a controlled Android build and in-game visual tests.

License: MIT, original copyright (c) 2026 Sierra. Preserve upstream attribution if copying any substantial source in later iterations.
