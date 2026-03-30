#!/usr/bin/env python3
"""
Nova GC & Ref-Counting Benchmark Harness
=========================================
Runs each bench_*.nv script multiple times, records wall-clock time,
and writes results to a JSON file for analysis.
"""

import json
import os
import subprocess
import sys
import time
import statistics

NOVA = os.path.join(os.path.dirname(__file__), "..", "..", "target", "release", "nova")
BENCH_DIR = os.path.dirname(__file__)
RESULTS_FILE = os.path.join(BENCH_DIR, "results.json")
RUNS = 5  # how many repetitions per benchmark

BENCHMARKS = {
    "01_mass_alloc":       {"file": "bench_01_mass_alloc.nv",       "category": "allocation",   "desc": "100k short-lived lists"},
    "02_list_grow":        {"file": "bench_02_list_grow.nv",        "category": "allocation",   "desc": "Grow list to 100k elements"},
    "03_string_concat":    {"file": "bench_03_string_concat.nv",    "category": "allocation",   "desc": "10k string concatenations"},
    "04_struct_churn":     {"file": "bench_04_struct_churn.nv",     "category": "allocation",   "desc": "100k struct alloc/dealloc"},
    "05_closure_capture":  {"file": "bench_05_closure_capture.nv",  "category": "retention",    "desc": "50k closures + call all"},
    "06_deep_nesting":     {"file": "bench_06_deep_nesting.nv",     "category": "retention",    "desc": "10k x depth-50 list chains"},
    "07_long_lived":       {"file": "bench_07_long_lived.nv",       "category": "gc_pressure",  "desc": "1k live + 100k temporary"},
    "08_clone_deep":       {"file": "bench_08_clone_deep.nv",       "category": "gc_pressure",  "desc": "50k deep clones of struct"},
    "09_list_replace":     {"file": "bench_09_list_replace.nv",     "category": "mutation",     "desc": "100 passes replacing 10k elements"},
    "10_enum_alloc":       {"file": "bench_10_enum_alloc.nv",       "category": "allocation",   "desc": "100k Some() wrap/unwrap"},
    "11_gc_pause":         {"file": "bench_11_gc_pause.nv",         "category": "latency",      "desc": "200 timed batches of 5k allocs"},
    "12_tuple_throughput": {"file": "bench_12_tuple_throughput.nv",  "category": "allocation",   "desc": "100k tuple alloc/dealloc"},
    "13_baseline":         {"file": "bench_13_baseline.nv",         "category": "baseline",     "desc": "1M int additions (no heap)"},
}


def run_benchmark(filepath: str, runs: int = RUNS) -> dict:
    """Run a Nova benchmark file multiple times, return timing data."""
    times = []
    for i in range(runs):
        start = time.perf_counter()
        result = subprocess.run(
            [NOVA, "run", filepath],
            capture_output=True, text=True, timeout=120
        )
        elapsed_ms = (time.perf_counter() - start) * 1000
        if result.returncode != 0:
            return {"error": result.stderr[:500], "times": []}
        times.append(elapsed_ms)
    
    return {
        "times": times,
        "mean_ms": statistics.mean(times),
        "median_ms": statistics.median(times),
        "min_ms": min(times),
        "max_ms": max(times),
        "stdev_ms": statistics.stdev(times) if len(times) > 1 else 0.0,
    }


def run_gc_pause_benchmark(filepath: str, runs: int = RUNS) -> dict:
    """
    Special handler for bench_11_gc_pause which prints per-iteration ms.
    We collect all 200 latency readings across runs.
    """
    all_latencies_us = []
    wall_times = []
    for _ in range(runs):
        start = time.perf_counter()
        result = subprocess.run(
            [NOVA, "run", filepath],
            capture_output=True, text=True, timeout=120
        )
        wall_ms = (time.perf_counter() - start) * 1000
        wall_times.append(wall_ms)
        if result.returncode != 0:
            return {"error": result.stderr[:500]}
        for line in result.stdout.strip().split("\n"):
            line = line.strip()
            if line:
                try:
                    all_latencies_us.append(float(line))
                except ValueError:
                    pass
    
    if not all_latencies_us:
        return {"error": "no latency data collected"}
    
    all_latencies_us.sort()
    n = len(all_latencies_us)
    return {
        "wall_times": wall_times,
        "wall_mean_ms": statistics.mean(wall_times),
        "latency_count": n,
        "latency_mean_us": statistics.mean(all_latencies_us),
        "latency_median_us": statistics.median(all_latencies_us),
        "latency_min_us": min(all_latencies_us),
        "latency_max_us": max(all_latencies_us),
        "latency_p50_us": all_latencies_us[int(n * 0.50)],
        "latency_p90_us": all_latencies_us[int(n * 0.90)],
        "latency_p95_us": all_latencies_us[int(n * 0.95)],
        "latency_p99_us": all_latencies_us[int(n * 0.99)],
        "latency_stdev_us": statistics.stdev(all_latencies_us) if n > 1 else 0.0,
        "all_latencies_us": all_latencies_us,
    }


def main():
    print(f"Nova GC Benchmark Harness — {RUNS} runs per benchmark")
    print(f"Nova binary: {os.path.abspath(NOVA)}")
    print("=" * 70)
    
    results = {}
    for name, info in BENCHMARKS.items():
        filepath = os.path.join(BENCH_DIR, info["file"])
        print(f"\n[{name}] {info['desc']} ...", flush=True)
        
        if name == "11_gc_pause":
            data = run_gc_pause_benchmark(filepath)
            if "error" in data:
                print(f"  ERROR: {data['error']}")
            else:
                print(f"  Wall: {data['wall_mean_ms']:.1f}ms avg")
                print(f"  Latency (µs): mean={data['latency_mean_us']:.0f} "
                      f"p50={data['latency_p50_us']:.0f} "
                      f"p90={data['latency_p90_us']:.0f} "
                      f"p95={data['latency_p95_us']:.0f} "
                      f"p99={data['latency_p99_us']:.0f} "
                      f"max={data['latency_max_us']:.0f}")
        else:
            data = run_benchmark(filepath)
            if "error" in data:
                print(f"  ERROR: {data['error']}")
            else:
                print(f"  Mean: {data['mean_ms']:.1f}ms  "
                      f"Min: {data['min_ms']:.1f}ms  "
                      f"Max: {data['max_ms']:.1f}ms  "
                      f"Stdev: {data['stdev_ms']:.1f}ms")
        
        results[name] = {**info, **data}
    
    # Write results
    with open(RESULTS_FILE, "w") as f:
        json.dump(results, f, indent=2)
    
    print(f"\n{'=' * 70}")
    print(f"Results written to {RESULTS_FILE}")


if __name__ == "__main__":
    main()
