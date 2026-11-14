# Native performance-mod porting contract

Starling is a native Rust Android game and runs Horizon native source mods (.hnmod) on ARM64. A source bundle is compiled into the game by Starling's infrastructure, not loaded as a Java mod or precompiled .so. Pin the exact game source commit for investigations and releases.

Official references:
- Starling native builds: https://github.com/starling-client/starling/blob/starling/docs/starling-native-mods.md
- Native mod archive format: https://github.com/starling-client/starling/blob/starling/docs/api/mod-files.md
- Native hooks: https://github.com/starling-client/starling/blob/starling/docs/native-mods.md
- Reference template: https://github.com/horizon-rust/horizon-native-mod-template

## Layout

Each native port eventually consists of mods/<slug>/README.md, horizon-native.json and mixins/<mod_id>/horizon-mixin.toml plus Rust source files, tests and any required license notices. A folder with only research notes is NOT a port.

The JSON manifest includes schema=1, id, version, game, horizon (40-digit source revision), mixins (40-digit tool revision) and optional hooks API version. The TOML [mod] id/version and [compatibility] horizon_revision must match. Actual injection target resolution requires horizon-mixins and a source checkout.

## Workflow

1. Audit the original mod's behavior, release and license; don't copy obsolete Java assumptions.
2. Audit the corresponding Rust routine in an exact Starling commit. Identify existing optimizations and source targets.
3. Measure an unmodified Android baseline: find real redundant work.
4. Add a failing regression test first. Then implement the smallest relevant hook or injection.
5. Run tests, source patch application, Rust compilation and Android ARM64 smoke tests.
6. Compare behavior and performance A/B on identical devices/worlds/settings.
7. Package into .hnmod only when source revisions, test results and attribution are recorded.
8. Publish only with successful native compilation, in-game parity and reproducible evidence.

## Release gate

No mod is called released without: exact Starling + source-mod commits, upstream license/attribution, confirmed functional Rust change, passing regression tests, source injection/build validation, working ARM64 launch and actual before/after performance evidence. A valid .hnmod ZIP does not prove any of those.

Native mods run with full game permissions, so only enable trusted source; document a rollback via disabling the mod set in Starling. Prefer named hook points. If an optimization requires stable engine-side interfaces that do not exist, propose the hook upstream rather than inventing one.