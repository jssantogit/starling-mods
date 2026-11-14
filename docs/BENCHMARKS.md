# Android performance measurement

Compare stock Starling and patched Starling on the SAME source revision; make exactly one optimization change at a time.

Record: SoC/CPU/GPU, Android/GPU-driver version, Starling commit, game version, world seed/location, render distance and simulation distance, resolution scale, shader pack, FPS cap, entities, power/charging state, and temperature/ambient conditions. Disable unrelated mods.

Warm up the world; run at least five matched trials while alternating A/B order to counter thermal drift. Record median frame time, 1% low FPS or P99 frame time, average FPS, CPU/GPU timings when available, memory, thermal trend, terrain-load time, and targeted counts of redundant work (lightmap calculations/uploads, etc.).

For lightmap caching, verify parity during day-night changes, weather, dimension/biome changes, brightness/gamma settings, potion effects (night vision/darkness), lightning and shaders. Provide a reproducible scenario, measurement method, raw values, spread, visual/behavioral comparison and verdict: improvement, inconclusive or regression.

Do not cite Java Edition desktop FPS numbers as evidence of Starling/Android gains.