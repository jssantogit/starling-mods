# Greedy Meshing → Starling 26.3: feasibility audit

## Verdict

**Prototype integrated into pinned Starling; not gameplay-ready.** The core rectangle algorithm and conservative native post-mesh top-face adapter now pass `horizon-mixins plan/apply` and Linux `cargo check`. The adapter caps rectangles at 2×2 and reuses near-section UV encoding/tiled WGSL; larger rectangles, other face directions, shader-pack safety and Android in-game parity are still open. Do not infer FPS gains from a reduction in quad count.

## Exact sources reviewed

| Source | Pinned revision / location | Finding |
| --- | --- | --- |
| [Greedy Meshing Mod](https://github.com/programmer1o1/GreedyMeshingMod/tree/cafb82a28247dc2652bd11759d0075939d406617), MIT (Sierra) | `src/main/java/hi/sierra/greedy_meshing/GreedyMesher.java` | Directional sweeps across 16x16 planes, keyed visibility and rectangular merging; testable as a pure Rust core. |
| Original eligibility | `GreedyEligibility.java`, `GreedySpriteSupport.java`, `GreedyLighting.java` | Limit to eligible opaque full-cubes; preserve sprite choices, atlas UV orientation, ambient-occlusion/light and connected/random textures. Aggressive lighting and greedy water are opt-ins with visual tradeoffs. |
| [Starling pinned](https://github.com/starling-client/starling/tree/09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2) | `crates/horizon-client-render/src/mesher.rs` | Near chunks emit baked model faces into `SectionMesh.solid`, with light/AO/tint/CTM decisions for each quad. No equivalent near-section greedy rectangle sweep found. |
| Starling near chunk culling | `mesher/sides.rs`, `mesher/leaf_culling.rs` | Already sorts direction-facing quads and culls leaves. Do not reimplement these features. |
| Starling LOD meshing | `lod/mesh.rs` | **Already merges distant faces** and uses `TerrainVertex::tiled`. Greedy meshing must target *near full-detail sections*, not the LOD path. |
| Starling shader | `terrain.wgsl` + `gfx.rs` | Distant terrain UVs use `TILED_UV_STEPS=32` and vertex-stage `instance_step`; near sections use `UV_STEPS=32768`. The shader already implements tiled sampling, but near quads currently cannot just store UVs > 1 across 16 blocks without a new unambiguous encoding/geometry path. |
| Original shaders | `src/client/resources/assets/minecraft/shaders/core/terrain.vsh` and `terrain.fsh` | Reconstruct per-block UV for merged quads; an equivalent carefully gated WGSL implementation is needed for Starling. |

## Eligibility before merging any face

1. Full, axis-aligned opaque cube whose **actual baked model** has canonical geometry and unit-tiled UVs, not just full collision geometry. Exclude block entities and non-cube render shapes.
2. Exactly the same precomputed **sprite layer, UV orientation, render pipeline and complete per-face lighting signature** (including AO, tint, emission) or a demonstrably equivalent shader-time shading path.
3. Exclude leaves, foliage/grass, transparent/cutout surfaces, moving water, offset or rotated models, animated/per-position random textures, connected textures and overlay quads until their semantics can be preserved. Shader packs must fall back safely to nonmerged drawing unless compatible.
4. Never bridge different chunk sections, visible/invisible face boundaries or distinct materials. Unknown/neighbouring chunks and resource reloads must not hide a legitimate face. Do not lose extra keys under any per-plane capacity cap.

## What has been implemented

- Rust `core.rs` performs a greedy rectangular sweep of one 16x16 plane of prevalidated visible face **keys**. It keeps missing/ineligible cells outside this path and cannot emit overlapping or out-of-bounds rectangles.
- Standalone Rust tests cover uniform, mixed, empty, holed, checkerboard, 256 unique-key and deterministic randomized cases.
- Added native source bundle at `mods/greedymeshing/native` with an explicit `RETURN` hook into `Mesher<'_>::mesh_reusing`, a tested shared `core.rs` and `runtime.rs`.
- The runtime rejects non-solid, emissive, tinted, offset and non-canonical top faces, refuses overlapping/partial overlays, and merges at most 2×2 faces with identical UV orientation and packed light/color. It does not change other face directions, complex geometry or fluids.
- Source hook plan/apply plus modified Linux renderer compilation passed in [CI](https://github.com/jssantogit/starling-mods/actions/runs/43633951718). [Android ARM64 app `cargo check`](https://github.com/jssantogit/starling-mods/actions/runs/43634253942) **passed**. Final library/link, test-world visual inspection, resource reload and performance benchmarking are not complete.
- **Safety caveat:** this adapter skips some shader-pack-specific lighting modes and translucent-layer configurations, but old-lighting shader packs without those indicators may still be active; shader-pack fallback is therefore **not proven**. Do not release without fully gating on pack activity.

## Proposed native integration (follow-up, not implemented)

1. **Prototype done for top faces only:** construct eligibility using padded block state and already-baked quad data after `Mesher::mesh_reusing` finishes, refusing overlapping faces and noncanonical UVs. Expand eligibility only after full parity tests.
2. **Prototype done for upward faces:** use the pure sweep on each horizontal integer plane and emit at most 2×2 merged `TerrainVertex` quads; all rejected faces stay on the original path. Extend to six directions after tests.
3. **Limited 2×2 prototype:** reuse the existing WGSL tiled sampler because packed near-section `u16` UVs cannot represent a full 16-block repeated face; the endpoint 2.0 saturates to 65535. A distinct encoding/shader path is still required for larger rectangles, preserving mip derivatives, sprite phase and chunk-fade semantics.
4. Verify both default pipeline and shader-pack fallback, smooth light, colored lighting, resource reload, biome tint, block overlays and section boundaries using prepared 26.3 resources and screenshot comparisons.
5. Only then produce a native .hnmod/APK and measure the same world, build profile, resolution and shader settings with and without the port. Track quad counts and GPU frame time, not FPS alone.

## Performance expectation

In the user's 35–47 FPS scene GPU time was approximately 20–22 ms, with dense **short grass** occupying the image. The original mod targets large flat opaque-cube faces, **not** the many grass cutout pixels. Therefore a real geometry reduction may yield limited FPS gain in that exact scene even when visually correct. Prioritize test scenes with large uniform stone/dirt surfaces, caves and large flat structures, then validate grass scenes as the negative control.

## Rights

Greedy Meshing Mod copyright (c) 2026 Sierra, licensed MIT. This source adaptation keeps attribution. No Minecraft files or Java jars are included here.
