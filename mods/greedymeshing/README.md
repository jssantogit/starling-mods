# Greedy Meshing — Rust source adaptation (feasibility prototype)

**Status: rectangle-sweep core only. NOT an installable mod, NOT linked to Starling and NOT benchmarked on Android.**

- Original mod: [Greedy Meshing Mod](https://github.com/programmer1o1/GreedyMeshingMod) by Sierra, MIT, pinned at `cafb82a28247dc2652bd11759d0075939d406617` (release 0.6.1, supports Minecraft 26.3).
- Target: [Starling](https://github.com/starling-client/starling) at `09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2` (Minecraft 26.3).
- Our `core.rs` adapts the original's **plane-by-plane greedy sweep concept** into standalone Rust code over precomputed material/visibility keys. This is a new Rust implementation, not the Java/Fabric code.
- Geometry tests assert no disappearing/duplicated faces, no overlap, no merging of differing keys, 16x16 section boundaries, and handling all 256 unique face keys.

## What we learned

See [AUDIT.md](AUDIT.md) for the source-level analysis and integration gates. Starling already merges faces for distant LOD terrain, but detailed nearby sections still follow baked models and per-block quads. This is a plausible gap for the original mod's technique, provided we adapt sprite tiling and preserve lighting and resource-pack behavior.

Run locally with Rust 1.95+:

```sh
cargo test --offline --manifest-path tests/rust/greedymeshing-core/Cargo.toml
```

**Do not package this as an .hnmod** until the actual Starling renderer hook, shader pipeline and parity tests exist. A successful unit test for rectangle grouping alone proves nothing about FPS or visual equivalence.

License: MIT, original copyright (c) 2026 Sierra. Preserve upstream attribution if copying any substantial source in later iterations.
