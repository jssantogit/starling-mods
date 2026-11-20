"""Exycraft diagnosis-contract tests. No Android device or network required."""
import importlib.util
import json
import tempfile
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("exycraft_analyze", ROOT / "tools" / "exycraft_analyze.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)


def fixture_zip(path, rows, truncated=False):
    lines = "\n".join(json.dumps(row) for row in rows) + "\n"
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr("horizon-performance-fixture.jsonl", lines)
        archive.writestr("manifest.json", json.dumps({
            "files": [{"file": "horizon-performance-fixture.jsonl",
                       "input_truncated_by_export": truncated}]
        }))
        archive.writestr("latest.log", "private user data must not be included by analyzer")


class AnalyzerTests(unittest.TestCase):
    def test_observed_metrics_and_unknown_gpu(self):
        with tempfile.TemporaryDirectory() as temp:
            dest = Path(temp) / "sample.zip"
            fixture_zip(dest, [
                {"type": "header", "schema": 1, "source_commit": "test",
                 "metadata": {"driver": {"name": "Mali-G68"}}},
                {"type": "sample", "frames": {
                    "frames": 40, "seconds": 1, "p95_ms": 28,
                    "low_1pct_fps": 27, "cpu": {"avg_ms": 7},
                    "gpu": {"avg_ms": 21}},
                 "dropped_samples": 1,
                 "timings": [{"stage": "mesh_section", "wall_ms": 12.5}],
                 "gpu_passes": {"world": [{"stage": "terrain", "p95_ms": 15.5,
                                           "samples": 15}]},
                 "slow_events": [{"type": "slow_frame"}],
                 "context": {"exycraft": {"probe_overhead_us": 30,
                    "proc_status": {"VmRSS_kb": 140000},
                    "proc_self_schedstat": {"runqueue_wait_ns": 1000000}}}},
                {"type": "sample", "frames": {
                    "frames": 60, "seconds": 1, "p95_ms": 17,
                    "low_1pct_fps": 51, "cpu": {"avg_ms": 5},
                    "gpu": None},
                 "dropped_samples": 2, "context": {
                     "exycraft": {"probe_overhead_us": 50,
                         "proc_status": {"VmRSS_kb": 150000},
                         "proc_self_schedstat": {"runqueue_wait_ns": 4000000}}}},
                {"type": "end", "reason": "stopped"},
            ], truncated=True)
            result = mod.analyze(dest)
            self.assertEqual(len(result["sessions"]), 1)
            s = result["sessions"][0]
            self.assertEqual(s["frames_recorded"], 100)
            self.assertEqual(s["estimated_fps_from_recorded_frames"], 50)
            self.assertEqual(s["median_window_p95_frame_ms"], 22.5)
            self.assertEqual(s["median_window_1pct_low_fps"], 39)
            self.assertEqual(s["gpu_dominant_window_candidates"], 1)
            self.assertEqual(s["unavailable_cpu_gpu_windows"], 1)
            self.assertEqual(s["gpu_timed_windows"], 1)
            self.assertEqual(s["max_rss_kb_observed"], 150000)
            self.assertEqual(s["median_exycraft_probe_overhead_us"], 40)
            self.assertEqual(s["leader_task_runqueue_wait_delta_ms"], 3)
            self.assertEqual(s["slow_events_recorded"], 1)
            self.assertEqual(s["top_cpu_worker_stages"][0]["stage"], "mesh_section")
            self.assertEqual(s["top_gpu_passes_rolling_p95"][0]["stage"], "world:terrain")
            self.assertTrue(s["export_truncated"])
            self.assertTrue(s["complete_end_record"])
            self.assertNotIn("private user data", mod.render(result))

    def test_probe_inside_gpu_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "gpu-probe.zip"
            fixture_zip(dest, [
                {"type": "header", "schema": 1},
                {"type": "sample", "frames": {"frames": 60, "seconds": 1},
                 "gpu_passes": {"world": [], "shader": None,
                    "exycraft": {"probe_overhead_us": 10,
                        "proc_status": {"VmRSS_kb": 200000}}}},
                {"type": "end", "reason": "stopped"},
            ])
            info = mod.analyze(dest)["sessions"][0]
            self.assertEqual(info["exycraft_windows"], 1)
            self.assertEqual(info["max_rss_kb_observed"], 200000)
            self.assertEqual(info["median_exycraft_probe_overhead_us"], 10)

    def test_missing_data_not_zero_and_partial_rows(self):
        with tempfile.TemporaryDirectory() as temp:
            dest = Path(temp) / "sample.zip"
            with zipfile.ZipFile(dest, "w") as archive:
                archive.writestr("horizon-performance-fixture.jsonl",
                    '{"type":"header","schema":1}\n'
                    '{"type":"sample","frames":{"frames":1}}\n'
                    '{broken}\n')
            result = mod.analyze(dest)
            s = result["sessions"][0]
            self.assertIsNone(s["estimated_fps_from_recorded_frames"])
            self.assertIsNone(s["mean_window_gpu_ms"])
            self.assertIsNone(s["max_rss_kb_observed"])
            self.assertFalse(s["complete_end_record"])
            self.assertEqual(s["unavailable_cpu_gpu_windows"], 1)
            self.assertTrue(any("invalid/redacted" in w for w in result["warnings"]))

    def test_rejects_non_zip(self):
        with tempfile.TemporaryDirectory() as temp:
            dest = Path(temp) / "invalid.zip"
            dest.write_bytes(b"plain text")
            with self.assertRaises(ValueError):
                mod.analyze(dest)


class NativeBundleTests(unittest.TestCase):
    def test_exycraft_package_contract(self):
        package_spec = importlib.util.spec_from_file_location(
            "hnmod_packager", ROOT / "tools" / "package_hnmod.py")
        package_mod = importlib.util.module_from_spec(package_spec)
        package_spec.loader.exec_module(package_mod)
        root = ROOT / "mods" / "exycraft" / "native"
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "exycraft.hnmod"
            package_mod.package(root, output)
            with zipfile.ZipFile(output) as z:
                self.assertIn("mixins/exycraft/probe.rs", z.namelist())
                self.assertIn("mixins/exycraft/after.rs", z.namelist())


if __name__ == "__main__":
    unittest.main()
