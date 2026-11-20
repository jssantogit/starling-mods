//! Lightweight, opt-in process probes for Starling's existing diagnostics.
//! Called at the start of recording and once per emitted sample, not per frame.
//! No persistent files, identifiers, network access, GPU commands or mutations.

use serde_json::{Value, json};
use std::time::Instant;

fn parse_status() -> Value {
    let Ok(text) = std::fs::read_to_string("/proc/self/status") else {
        return Value::Null;
    };
    let mut values = serde_json::Map::new();
    for line in text.lines() {
        let Some((name, tail)) = line.split_once(':') else { continue };
        let number = tail.split_whitespace().next().and_then(|s| s.parse::<u64>().ok());
        let Some(number) = number else { continue };
        match name {
            "VmRSS" | "VmHWM" | "VmSize" => {
                values.insert(format!("{name}_kb"), json!(number));
            }
            "Threads" | "voluntary_ctxt_switches" | "nonvoluntary_ctxt_switches" => {
                values.insert(name.to_string(), json!(number));
            }
            _ => {}
        }
    }
    Value::Object(values)
}

// /proc/self/schedstat is a task-scoped Linux counter, not a whole-CPU or
// whole-GPU utilization counter. Treat these as cumulative and optional.
fn read_schedstat() -> Value {
    let Ok(text) = std::fs::read_to_string("/proc/self/schedstat") else {
        return Value::Null;
    };
    let mut iter = text.split_whitespace();
    match (
        iter.next().and_then(|s| s.parse::<u64>().ok()),
        iter.next().and_then(|s| s.parse::<u64>().ok()),
        iter.next().and_then(|s| s.parse::<u64>().ok()),
    ) {
        (Some(on_cpu_ns), Some(runqueue_wait_ns), Some(timeslices)) => json!({
            "on_cpu_ns": on_cpu_ns,
            "runqueue_wait_ns": runqueue_wait_ns,
            "timeslices": timeslices,
        }),
        _ => Value::Null,
    }
}

pub(super) fn enrich(context: &mut Value, gpu_name: Option<&str>) {
    let start = Instant::now();
    let label = gpu_name.unwrap_or("unknown");
    let g68 = label.to_ascii_lowercase().contains("mali-g68");
    let status = parse_status();
    let schedstat = read_schedstat();
    let overhead_us = start.elapsed().as_micros().min(u64::MAX as u128) as u64;
    context["exycraft"] = json!({
        "schema": 1,
        "version": "0.1.0-alpha.1",
        "mode": "diagnostics_only",
        "gpu_name": label,
        "gpu_family": if g68 { "mali-g68" } else { "unclassified" },
        "exynos_1380_confirmed": false,
        "proc_status": status,
        "proc_self_schedstat": schedstat,
        "probe_overhead_us": overhead_us,
    });
}
