# Exycraft — diagnostics-only native experiment (0.1.0-alpha.1)

**State:** source prototype; no APK, verified ARM64 build or device benchmark yet.
The repository's existing cumulative performance ports remain independent.
Exycraft performs **no optimization** and makes no frame-rate claims.

## What it changes

The source mod adds a cancellable HEAD injection to
`horizon_client_render::gfx::Renderer::diagnostic_pass_times` in the
**pinned** Starling revision `09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2`.
Starling calls this method for performance samples. Because its original body
contains macros unsupported by RETURN rewriting, the HEAD hook reproduces the
pinned method's **world** and **shader** timing fields exactly, then appends an
`exycraft` field to the existing `gpu_passes` report. This must be parity-tested
when updating Starling; it does not replace the menu, recorder or ZIP exporter.

The optional fields are:

| JSON field | Source | Interpretation |
| --- | --- | --- |
| `gpu_name`, `gpu_family` | wgpu renderer adapter name | Mali-G68 family guess; **does not establish Exynos 1380** |
| `proc_status.VmRSS_kb` | `/proc/self/status` | Snapshot of app resident memory, in KiB |
| `proc_status.VmHWM_kb` | `/proc/self/status` | High-water RSS |
| `proc_status.Threads` | `/proc/self/status` | Thread count snapshot |
| `proc_status.*ctxt_switches` | `/proc/self/status` | Cumulative Linux counters; not FPS |
| `proc_self_schedstat.*` | `/proc/self/schedstat` | Task-scoped run time and wait counters, if readable |
| `probe_overhead_us` | monotonic timer | Cost of probe calls, excluding the stock recorder |

If Android blocks a procfs read, that object is `null`. GPU clocks, GPU
utilization, thermal throttling attribution and hardware counters **are not
collected** by this mod. The Starling recorder already provides frame
times, GPU pass timing when available, memory and thermal information.
No privileged sysfs access or driver replacement is attempted.

The probe is only invoked while the user has enabled Starling's existing
performance recording. Data is
stored and exported by the **existing local Starling pipeline**. No
background telemetry, identifiers, network requests or new automatic
exports.

## Capture procedure

1. Install a validated build of the exact compatible Starling revision and
   the Exycraft source bundle using its official native-mod build flow.
2. Open **Options → Diagnostics**. Start performance recording. For GPU
   pass timers, separately enable detailed GPU profiling; it has overhead.
3. Warm up the world, then record a consistent, repeatable route for
   60–120 seconds. Use the same settings, camera movement, resource packs
   and shader packs across baseline and mod builds.
4. Stop recording, export **diagnostic ZIP** and review it before sharing.
5. On a computer with Python 3.11+, analyze the ZIP:

```sh
python3 tools/exycraft_analyze.py starling-diagnostics-123.zip
python3 tools/exycraft_analyze.py starling-diagnostics-123.zip --json
```

The analyzer reads only performance JSONL and the ZIP manifest; it never
extracts the archive or reads chat, logs, accounts or world files. Its
outputs intentionally omit raw context that could contain personal data.

The native hook produces a field under each **periodic** `sample.gpu_passes`.
Startup/menu samples using the renderer can also contain the field. The
recording header is unchanged.

## Interpreting results

The performance JSONL contains one-second aggregates, not individual
frame history. The analyzer reports per-window medians and means **as
such**. In particular, median per-window 1% lows are not a true whole-run
1% low. GPU timings may be delayed, missing or perturb the run. GPU-vs-CPU
timing comparisons are *candidates* for investigation, not proof of
hardware bottlenecks. The procfs schedstat counter concerns a Linux task,
not the whole process GPU utilization.

Starling's exporter preserves a header and a bounded trailing portion of
long JSONL files. Always inspect `export_truncated` and missing end
records. Absence of a field means unknown, never zero.

## Build and compatibility

```sh
python3 tools/package_hnmod.py mods/exycraft/native --output target/exycraft-0.1.0-alpha.1.hnmod
python3 -m unittest discover -s tests -v
```

Packaging checks source integrity but does **not** compile the mod.
The new Exycraft native CI workflow must pass mixin plan/apply and Android
ARM64 cargo check before any installation claim. The release pipeline
must also verify actual device boot, recording, export and frame overhead.

This is an original diagnostic extension, not a port of a third-party mod.
