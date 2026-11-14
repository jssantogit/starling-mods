# BadOptimizations → Starling 26.3: feature-by-feature scope

This matrix audits **concepts**, not Java binaries. Original source: Thosea's [BadOptimizations at `5fa3416`](https://github.com/imthosea/BadOptimizations/tree/5fa3416f6148eaa4045399cf7fa24a5427cc2991) (`oldsrc/26.3`, MIT). Target source: Starling [`09fa16c`](https://github.com/starling-client/starling/tree/09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2). A candidate does **not** imply a bottleneck exists on Android.

| Technique in original 26.3 source | Starling status | Next action |
| --- | --- | --- |
| Avoid redundant lightmap calculation (`MixinLightmapExtractor`) | **Prototype implemented.** Starling already avoids identical GPU uploads; the native patch skips the preceding 256-pixel CPU calculation when the cache key and pixels match. Linux and ARM64 app Cargo checks passed. | Build a final ARM64 library, verify gameplay/visual parity and record A/B timings. |
| Debug renderer early exit (`MixinDebugRenderer`) | **Not ported.** Original skips `emitGizmos` with an empty renderer list. | Find the equivalent Rust draw path and measure whether it already returns early. |
| Particle extraction early exit (`MixinParticleManager`) | **Not ported.** Original skips extraction with no particle groups; this differs from culling invisible particles. | Locate the Rust extraction queue, check the existing empty-state guard, and profile it. |
| Toast render early exit (`MixinToastComponent`) | **Not ported.** Original skips when GUI hidden or the toast/sound queues are empty. | Determine whether Starling's HUD schedules idle toast work at all. |
| Skip tutorial tick outside demo (`MixinTutorial`) | **Not ported; applicability unknown.** | First verify that Starling has comparable demo-only tutorial work. |
| Avoid redundant camera FOV calculations (`MixinCamera`) | **Not ported.** Original special-cases zero FOV-effect scale while preserving scoped view. | Audit the Rust camera path and all its inputs before considering a shortcut. |
| Entity-renderer lookup caching (`MixinEntityRendererDispatcher`, `MixinEntityType`, `MixinEntity`) | **Not ported; potentially Java-specific.** | Audit Rust entity rendering and reload-time invalidation; only cache a real bottleneck. |
| Block-entity renderer lookup caching (`MixinBlockEntityRenderDispatcher`, `MixinBlockEntityType`) | **Not ported; possibly already inexpensive in Rust.** | Check existing Rust block-entity renderer caches first. |
| Other `Config.java` toggles (sky angle, entity flags, etc.) | **Not audited.** A config toggle alone does not establish active optimized code in 26.3. | Identify source logic and native equivalents before implementing. |
| Legacy sky-colour caching | **Excluded.** The original project's changelog removed it after 1.21.10 with data-driven sky colours. | No 26.3 port without new evidence. |

## Acceptance rules

For each technique: verify the exact upstream implementation and license; find and profile the equivalent Starling path; reproduce with tests before changing logic; apply mixins and compile with the pinned source; validate ARM64 gameplay and visual parity; then publish measurable CPU/frame-time improvement or a documented negative result.

**Lightmap target:** `crates/horizon-client-render/src/gfx.rs`, `Renderer::update_lightmap` in pinned Starling. The existing GPU-upload guard uses `lightmap_uploaded` and `lightmap_pixels`. The current thread-local prototype uses the 16 input values read by this method, the last 1024 output bytes, renderer identity and upload flag. Hook compatibility, resource-reload behavior and GPU recovery still need in-game tests.
