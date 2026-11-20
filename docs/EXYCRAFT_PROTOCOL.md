# Exycraft diagnostic experiment protocol (alpha.1)

**Question:** Which stages correlate with frame-time spikes on Mali-G68,
and is the newer Android client slower than the older one in a controlled
test? No graphics optimizations should be enabled in Exycraft alpha.1.

## A/B matrix

For each chosen scene, collect at least three baseline and three Exycraft
runs. Fix the Starling commit, build profile (quick/release), resolution
scale, Vulkan/OpenGL backend, FPS cap, view/simulation distance, texture
pack, shaders, battery/charging policy, and the route/camera movement.
Allow comparable thermal warm-up and cool-down. Repeat under both stock
GPU timers disabled and detailed GPU timers enabled to estimate overhead.

Record in the test notes: phone model, firmware/OS version, app revision,
renderer backend, target FPS, display refresh rate, resolution, world
conditions, test start/stop, and whether resource packs or mods changed.
Do not save world seed, account data, coordinates or unique device IDs.

## Questions and interpretations

- Frame p50/p95/p99 and 1% lows: use *separate per-window summaries*.
  Exported aggregates cannot reconstruct whole-run percentiles.
- GPU vs CPU time: GPU timestamps lag and may be unavailable; CPU time
  excludes waits. Correlation alone is not root-cause proof.
- GPU passes: inspect stage names and completed-sample counts. Missing
  timers are unknown, not free work.
- Meshing/streaming: correlate named CPU/worker spans, pending queues,
  uploads, resident sections and vertex bytes from Starling's context.
- Memory: compare process RSS and high-water RSS, uploaded bytes and
  section-vertex bytes. Different snapshots may be sampled at different
  moments.
- Scheduling: procfs task wait deltas and voluntary/non-voluntary context
  switches are optional indirect clues, not per-core CPU saturation.
- Thermal: rely on Stock Starling Android thermal values when present;
  don't claim throttling from a high temperature alone.
- Collector overhead: inspect `probe_overhead_us` and compare recordings
  with and without Exycraft; it excludes the original recorder and GPU
  timestamp costs.

## Quality gate

An experiment is invalid if its build versions, graphics settings or
recording window differ materially without annotation. Reject any
interpretation of absent GPU values as zero. Do not classify Exynos 1380
from Mali-G68 alone. Compare multiple runs and report uncertainty.

Local reports stay private unless explicitly exported. Review ZIP content
before sharing because even redacted logs may reveal private information.
