# Starling Greedy alpha.2 — Android visual parity test plan

**Not a performance release.** The first objective is to prove correct rendering on an Android ARM64 device.

- Build source: Starling `09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2`, Greedy Meshing 0.1.0-alpha.2 only.
- CI: [Build isolated Starling Greedy test APK](../../.github/workflows/build-greedymeshing-apk.yml). Use only a **successful** run's artifact `starling-greedymeshing-alpha2-arm64`.
- ZIP includes `starling-dev-0.8.0-arm64-v8a.apk` (file name from upstream builder), `build.json` and a source `.hnmod`. The APK **installs as a distinct app called Starling Greedy**, package `io.github.juanluci.starling.greedy`. It should not replace Starling Dev/Preview/stable. The `.hnmod` does not need importing.
- Only use disposable test worlds; the binary is an unsigned-by-release, debug-signed experiment with native-code privileges.
- **Disable shader packs and use default textures for all first tests.** Shader-pack activation is not fully guarded, especially `oldLighting=true`; do not use it with important worlds or a signed-in account you cannot recover.

## Functional / visual parity

1. Boot with default graphical settings. Confirm the title screen, world creation and survival/creative play work.
2. Build a 4×4 stone floor, walls in both X/Z orientations and a 4×4 ceiling; inspect top/bottom/side faces. Compare textures with an unmodded build, same view/light/time.
3. Repeat with dirt, granite, planks and rotated directional blocks. Observe UV repetition at seams and all six surface normals.
4. Place slabs, stairs, fences, leaves, water, glass, grass, torches and tinted blocks alongside the test wall. They must remain visually identical and no surface may disappear.
5. Put an opaque 2×2 patch next to translucent overlays and check the original overlay is not erased.
6. Change daylight, cave light, gamma/night vision; then break and replace blocks repeatedly at chunk boundaries. No stale chunks or incorrect merged faces.
7. Reload a world and cross between dimensions. Then compare GPU frame time and quad/vertex counts with a source-identical unmodded `quick` build; do not compare directly with a release built under different compiler profiles.

Report observations as *pass/fail by scenario* plus screenshots at matching positions. Do not infer improved FPS from a single snapshot; no gains are guaranteed, especially in grass-heavy/GPU-fill-limited scenes.

## Stop conditions

Stop the test and report logs if the app crashes, surfaces vanish, lighting flickers, textures become stretched or any unexpected account/world permission dialog appears. Do not merge the PR or publish releases until these are addressed and shader-pack fallback is complete.
