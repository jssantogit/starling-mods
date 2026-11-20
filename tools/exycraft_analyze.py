#!/usr/bin/env python3
"""Analyze Starling's existing opt-in diagnostic ZIPs (no device access).

All figures are descriptive. One-second percentiles and 1%-lows cannot be
combined into exact whole-run percentiles from the exported summaries.
"""
import argparse
import json
import math
import statistics
import sys
import zipfile
from pathlib import Path

MAX_MEMBER = 4 * 1024 * 1024


def number(value):
    """Finite nonnegative measurement or None (unknown is never zero)."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    if not math.isfinite(value) or value < 0:
        return None
    return float(value)


def nested(source, *keys):
    for key in keys:
        if not isinstance(source, dict):
            return None
        source = source.get(key)
    return source


def median(rows):
    rows = [v for v in rows if v is not None]
    return round(statistics.median(rows), 3) if rows else None


def mean(rows):
    rows = [v for v in rows if v is not None]
    return round(statistics.mean(rows), 3) if rows else None


def safe_member(archive, name):
    member = archive.getinfo(name)
    if member.file_size > MAX_MEMBER:
        raise ValueError(f"member exceeds {MAX_MEMBER} bytes: {name}")
    with archive.open(member) as f:
        data = f.read(MAX_MEMBER + 1)
    if len(data) > MAX_MEMBER:
        raise ValueError(f"member exceeds limit when read: {name}")
    return data


def stage_summary(samples):
    """Aggregate names without adding overlapping CPU or GPU durations."""
    cpu_stages = {}
    gpu_stages = {}
    for row in samples:
        spans = row.get("timings")
        if isinstance(spans, list):
            for span in spans:
                if not isinstance(span, dict):
                    continue
                name, wall = span.get("stage"), number(span.get("wall_ms"))
                if isinstance(name, str) and 0 < len(name) <= 80 and wall is not None:
                    cpu_stages.setdefault(name, []).append(wall)
        passes = row.get("gpu_passes")
        if not isinstance(passes, dict):
            continue
        for family in ("world", "shader"):
            stages = passes.get(family)
            if not isinstance(stages, list):
                continue
            for stage in stages:
                if not isinstance(stage, dict):
                    continue
                name = stage.get("stage")
                p95 = number(stage.get("p95_ms"))
                count = number(stage.get("samples"))
                if (isinstance(name, str) and 0 < len(name) <= 80
                        and p95 is not None and count is not None and count > 0):
                    gpu_stages.setdefault(f"{family}:{name}", []).append(p95)
    cpu = sorted(
        [{"stage": k, "median_observed_wall_ms": median(v), "windows": len(v)}
         for k, v in cpu_stages.items()],
        key=lambda x: x["median_observed_wall_ms"], reverse=True
    )[:15]
    gpu = sorted(
        [{"stage": k, "median_rolling_p95_ms": median(v), "windows": len(v)}
         for k, v in gpu_stages.items()],
        key=lambda x: x["median_rolling_p95_ms"], reverse=True
    )[:15]
    return cpu, gpu


def summarize(name, rows, truncated=False):
    header = next((row for row in rows if row.get("type") == "header"), {})
    samples = [row for row in rows if row.get("type") == "sample"]
    ends = [row for row in rows if row.get("type") == "end"]
    frames = [row.get("frames") for row in samples]
    frames = [v for v in frames if isinstance(v, dict)]
    counts = [number(v.get("frames")) for v in frames]
    durations = [number(v.get("seconds")) for v in frames]
    total_frames = sum(v for v in counts if v is not None)
    total_frame_seconds = sum(v for v in durations if v is not None)
    fps = (round(total_frames / total_frame_seconds, 3)
           if total_frames > 0 and total_frame_seconds > 0 else None)
    gpu_windows = [number(nested(v, "gpu", "avg_ms")) for v in frames]
    cpu_windows = [number(nested(v, "cpu", "avg_ms")) for v in frames]
    gpu_dominant = 0
    cpu_dominant = 0
    ambiguous = 0
    unavailable = 0
    for cpu, gpu in zip(cpu_windows, gpu_windows):
        if cpu is None or gpu is None:
            unavailable += 1
        elif gpu > cpu * 1.25:
            gpu_dominant += 1
        elif cpu > gpu * 1.25:
            cpu_dominant += 1
        else:
            ambiguous += 1
    exy = [nested(v, "gpu_passes", "exycraft") or nested(v, "context", "exycraft") for v in samples]
    exy = [v for v in exy if isinstance(v, dict)]
    rss = [number(nested(v, "proc_status", "VmRSS_kb")) for v in exy]
    over = [number(v.get("probe_overhead_us")) for v in exy]
    waits = [number(nested(v, "proc_self_schedstat", "runqueue_wait_ns")) for v in exy]
    # Cumulative counters: only compare first/last if both exist; missing != 0.
    observed = [v for v in waits if v is not None]
    wait_delta = (round((observed[-1] - observed[0]) / 1e6, 3)
                  if len(observed) >= 2 and observed[-1] >= observed[0] else None)
    slow_events = [event for v in samples
                   for event in (v.get("slow_events") if isinstance(v.get("slow_events"), list) else [])
                   if isinstance(event, dict)]
    cpu_stages, gpu_stages = stage_summary(samples)
    phase_names = ("update", "tick", "prepare", "render")
    phase_averages = {
        phase: mean([number(nested(v, "phases_avg_ms", phase)) for v in frames])
        for phase in phase_names
    }
    contexts = [s.get("context") for s in samples if isinstance(s.get("context"), dict)]
    resident = [number(nested(c, "renderer", "resident_sections")) for c in contexts]
    uploaded = [number(nested(c, "renderer", "uploaded_bytes_total")) for c in contexts]
    metadata = header.get("metadata") if isinstance(header.get("metadata"), dict) else {}
    return {
        "file": name,
        "schema": header.get("schema"),
        "build_commit": header.get("source_commit"),
        "gpu": nested(metadata, "driver", "name"),
        "sample_windows": len(samples),
        "frames_recorded": int(total_frames),
        "recorded_frame_seconds": round(total_frame_seconds, 3),
        "estimated_fps_from_recorded_frames": fps,
        "median_window_p95_frame_ms": median([number(v.get("p95_ms")) for v in frames]),
        "median_window_1pct_low_fps": median([number(v.get("low_1pct_fps")) for v in frames]),
        "mean_window_cpu_ms": mean(cpu_windows),
        "mean_window_gpu_ms": mean(gpu_windows),
        "gpu_timed_windows": sum(v is not None for v in gpu_windows),
        "gpu_dominant_window_candidates": gpu_dominant,
        "cpu_dominant_window_candidates": cpu_dominant,
        "similar_cpu_gpu_windows": ambiguous,
        "unavailable_cpu_gpu_windows": unavailable,
        "slow_events_recorded": len(slow_events),
        "phases_mean_window_ms": phase_averages,
        "top_cpu_worker_stages": cpu_stages,
        "top_gpu_passes_rolling_p95": gpu_stages,
        "max_resident_sections_observed": max([v for v in resident if v is not None], default=None),
        "max_uploaded_bytes_total_observed": max([v for v in uploaded if v is not None], default=None),
        "dropped_samples_reported": max(
            [number(v.get("dropped_samples")) for v in samples
             if number(v.get("dropped_samples")) is not None], default=0),
        "exycraft_windows": len(exy),
        "max_rss_kb_observed": max([v for v in rss if v is not None], default=None),
        "median_exycraft_probe_overhead_us": median(over),
        "leader_task_runqueue_wait_delta_ms": wait_delta,
        "export_truncated": truncated,
        "complete_end_record": bool(ends),
    }


def analyze(path):
    path = Path(path)
    if not zipfile.is_zipfile(path):
        raise ValueError(f"not a ZIP file: {path}")
    with zipfile.ZipFile(path) as archive:
        # The program never extracts files or reads credentials / log payloads.
        names = [name for name in archive.namelist()
                 if name.startswith("horizon-performance-") and name.endswith(".jsonl")
                 and "/" not in name and "\\" not in name]
        manifest = {}
        if "manifest.json" in archive.namelist():
            try:
                manifest = json.loads(safe_member(archive, "manifest.json"))
            except (ValueError, UnicodeError, json.JSONDecodeError):
                manifest = {}
        truncations = {
            item.get("file") for item in manifest.get("files", [])
            if isinstance(item, dict) and item.get("input_truncated_by_export") is True
        }
        sessions = []
        warnings = []
        for name in sorted(names):
            try:
                raw = safe_member(archive, name).decode("utf-8")
            except (UnicodeError, ValueError, KeyError) as exc:
                warnings.append(f"{name}: {exc}")
                continue
            rows = []
            for i, line in enumerate(raw.splitlines(), 1):
                if not line.strip():
                    continue
                try:
                    parsed = json.loads(line)
                except json.JSONDecodeError:
                    warnings.append(f"{name}: invalid/redacted JSONL row {i}")
                    continue
                if isinstance(parsed, dict):
                    rows.append(parsed)
            sessions.append(summarize(name, rows, name in truncations))
        if not sessions:
            warnings.append("No valid performance JSONL found; enable recording while playing, stop, then export.")
        return {"schema": 1, "source_zip": path.name, "sessions": sessions, "warnings": warnings}


def render(analysis):
    lines = ["# Exycraft diagnostic analysis", "",
             f"Source: {analysis['source_zip']}", ""]
    for s in analysis["sessions"]:
        lines += [f"## {s['file']}", "",
                  f"- Captured windows: {s['sample_windows']}; frames: {s['frames_recorded']}",
                  f"- Estimated FPS from recorded frame durations: {s['estimated_fps_from_recorded_frames']}",
                  f"- Median of per-window frame p95: {s['median_window_p95_frame_ms']} ms",
                  f"- Median of per-window 1% lows: {s['median_window_1pct_low_fps']} FPS",
                  f"- Mean of per-window CPU/GPU averages: {s['mean_window_cpu_ms']} / {s['mean_window_gpu_ms']} ms",
                  f"- Candidate GPU-dominant / CPU-dominant / similar / unknown windows: "
                  f"{s['gpu_dominant_window_candidates']} / {s['cpu_dominant_window_candidates']} / "
                  f"{s['similar_cpu_gpu_windows']} / {s['unavailable_cpu_gpu_windows']}",
                  f"- Mean phase durations (ms): {s['phases_mean_window_ms']}",
                  f"- CPU/worker stages (not additive): {s['top_cpu_worker_stages'][:5]}",
                  f"- GPU passes (rolling p95): {s['top_gpu_passes_rolling_p95'][:5]}",
                  f"- Resident sections / uploaded bytes peak: {s['max_resident_sections_observed']} / {s['max_uploaded_bytes_total_observed']}",
                  f"- Reported slow events: {s['slow_events_recorded']}; dropped samples: {s['dropped_samples_reported']}",
                  f"- Exycraft measurements: {s['exycraft_windows']}; observed max RSS: {s['max_rss_kb_observed']} KiB",
                  f"- Median Exycraft collection overhead: {s['median_exycraft_probe_overhead_us']} us",
                  f"- Leader-task runqueue wait delta: {s['leader_task_runqueue_wait_delta_ms']} ms",
                  f"- Export truncated: {s['export_truncated']}; end record present: {s['complete_end_record']}", ""]
    for warning in analysis["warnings"]:
        lines.append(f"Warning: {warning}")
    lines += ["", "Method: per-window p95 and 1%-low summaries are NOT exact whole-run percentiles.",
              "CPU spans overlap; GPU timestamps may lag or be unavailable.",
              "CPU/GPU dominance is a heuristic, not proof of a bottleneck.",
              "The leader-task scheduler counter is not a measure of whole-device or render-thread wait.",
              "Missing metrics remain unknown rather than zero; profiling can change performance."]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("zip", type=Path, help="ZIP exported via Starling Options > Diagnostics")
    parser.add_argument("--json", action="store_true", help="print machine-readable summary")
    args = parser.parse_args()
    try:
        analysis = analyze(args.zip)
    except (ValueError, OSError, zipfile.BadZipFile) as exc:
        parser.error(str(exc))
    if args.json:
        print(json.dumps(analysis, indent=2, ensure_ascii=False))
    else:
        print(render(analysis))
    return 0


if __name__ == "__main__":
    sys.exit(main())
