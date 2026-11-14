# BadOptimizations — native Starling feasibility audit

**Research only — there is no functional port or installable .hnmod here.**

Original: https://github.com/imthosea/BadOptimizations
Modrinth: https://modrinth.com/mod/badoptimizations
Original license: MIT. Before adopting code, pin and attribute its exact source revision.
Target: Starling ARM64, Minecraft 26.3; exact Starling revision not yet selected.

## Candidate A — avoid redundant lightmap work

The original mod skips lightmap calculations and GPU uploads when lighting inputs have not changed. First locate the Rust update/upload path and check whether Starling already caches it. Profile the hot path on Android; only then propose a dirty flag/cache key covering ALL relevant changes: gamma, effects, dimension, day/night, weather, shader behavior and so on. Test every invalidation path. Accept only if behavior remains identical and targeted work and frame time measurably improve.

## Candidate B — debug renderer idle work

Investigate if any inactive debug subsystem still performs expensive work. Guard only truly redundant computation, never valid diagnostics; benchmark.

## Do not copy obsolete sky-color optimization

BadOptimizations removed the old sky-color caching scheme in modern Minecraft because sky colors became data driven. A pre-1.21.11 cubic-sampler shortcut is not an appropriate 26.3 port without new source evidence.

## Checks before implementation

- [ ] Identify exact Starling source revision and original BadOptimizations revision.
- [ ] Verify target Rust code and whether the optimization already exists.
- [ ] Measure actual Android baseline and code-path invocation frequency.
- [ ] Add failing regression tests; implement minimal native injection.
- [ ] Build .hnmod via real native tooling and test launch on ARM64.
- [ ] Verify parity and publish reproducible A/B results.

A directory of planning notes is not a release.