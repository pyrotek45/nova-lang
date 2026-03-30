#!/usr/bin/env python3
"""
Nova GC/RC Benchmark Analysis
==============================
Reads results.json, generates charts as PNG files, and prints a summary.

Run:  nix-shell -p 'python3.withPackages(ps: [ps.matplotlib])' --run 'python3 plot_gc_results.py'
"""

import json
import os
import sys
import math

# ---------------------------------------------------------------------------
# Load data
# ---------------------------------------------------------------------------
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
RESULTS_FILE = os.path.join(SCRIPT_DIR, "results.json")
OUT_DIR = os.path.join(SCRIPT_DIR, "charts")
os.makedirs(OUT_DIR, exist_ok=True)

with open(RESULTS_FILE) as f:
    data = json.load(f)

import matplotlib
matplotlib.use("Agg")  # headless
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker

# house style
plt.rcParams.update({
    "figure.facecolor": "#1e1e2e",
    "axes.facecolor":   "#1e1e2e",
    "axes.edgecolor":   "#585b70",
    "axes.labelcolor":  "#cdd6f4",
    "text.color":       "#cdd6f4",
    "xtick.color":      "#a6adc8",
    "ytick.color":      "#a6adc8",
    "grid.color":       "#313244",
    "figure.dpi":       150,
    "font.size":        10,
    "axes.titlesize":   13,
    "axes.titleweight": "bold",
})

PALETTE = {
    "allocation": "#89b4fa",
    "retention":  "#f9e2af",
    "gc_pressure":"#f38ba8",
    "mutation":   "#a6e3a1",
    "latency":    "#cba6f7",
    "baseline":   "#9399b2",
}


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
def save(fig, name):
    path = os.path.join(OUT_DIR, name)
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)
    print(f"  -> {path}")


# collect simple benchmark data (everything except bench_11)
simple = {k: v for k, v in data.items() if k != "11_gc_pause"}
bench11 = data.get("11_gc_pause", {})

# sort by mean time
ordered = sorted(simple.items(), key=lambda kv: kv[1]["mean_ms"])
labels   = [v["desc"] for _, v in ordered]
means    = [v["mean_ms"] for _, v in ordered]
mins     = [v["min_ms"] for _, v in ordered]
maxs     = [v["max_ms"] for _, v in ordered]
stdevs   = [v["stdev_ms"] for _, v in ordered]
cats     = [v["category"] for _, v in ordered]
colors   = [PALETTE.get(c, "#89b4fa") for c in cats]

# ops counts per benchmark (from source)
OPS = {
    "01_mass_alloc":     100_000,
    "02_list_grow":      100_000,
    "03_string_concat":   10_000,
    "04_struct_churn":   100_000,
    "05_closure_capture": 50_000,
    "06_deep_nesting":    10_000,
    "07_long_lived":     100_000,
    "08_clone_deep":      50_000,
    "09_list_replace":   1_000_000,  # 100 passes * 10k
    "10_enum_alloc":     100_000,
    "12_tuple_throughput":100_000,
    "13_baseline":       1_000_000,
}

# ---------------------------------------------------------------------------
# CHART 1 — Horizontal bar: mean wall-clock time per benchmark
# ---------------------------------------------------------------------------
print("Generating charts...")
fig, ax = plt.subplots(figsize=(10, 7))
y_pos = range(len(labels))
bars = ax.barh(y_pos, means, color=colors, edgecolor="#45475a", height=0.65)
# error bars (min/max range)
xerr_low  = [m - mn for m, mn in zip(means, mins)]
xerr_high = [mx - m for m, mx in zip(means, maxs)]
ax.errorbar(means, y_pos, xerr=[xerr_low, xerr_high], fmt="none",
            ecolor="#a6adc8", capsize=3, linewidth=1)
ax.set_yticks(list(y_pos))
ax.set_yticklabels(labels, fontsize=9)
ax.set_xlabel("Wall-clock time (ms)")
ax.set_title("Nova GC Benchmarks — Mean Execution Time (5 runs)")
ax.invert_yaxis()
ax.grid(axis="x", alpha=0.3)
# value labels
for bar, m in zip(bars, means):
    ax.text(bar.get_width() + 0.5, bar.get_y() + bar.get_height()/2,
            f"{m:.1f} ms", va="center", fontsize=8, color="#cdd6f4")
# legend for categories
from matplotlib.patches import Patch
legend_items = [Patch(facecolor=PALETTE[c], label=c) for c in
                ["allocation", "retention", "gc_pressure", "mutation", "baseline"]]
ax.legend(handles=legend_items, loc="lower right", fontsize=8,
          facecolor="#313244", edgecolor="#585b70")
save(fig, "01_wall_clock.png")

# ---------------------------------------------------------------------------
# CHART 2 — Throughput (ops/ms)
# ---------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(10, 7))
throughputs = []
tp_labels = []
tp_colors = []
for key, v in ordered:
    if key in OPS:
        tp = OPS[key] / v["mean_ms"]
        throughputs.append(tp)
        tp_labels.append(v["desc"])
        tp_colors.append(PALETTE.get(v["category"], "#89b4fa"))

y_pos = range(len(tp_labels))
bars = ax.barh(y_pos, throughputs, color=tp_colors, edgecolor="#45475a", height=0.65)
ax.set_yticks(list(y_pos))
ax.set_yticklabels(tp_labels, fontsize=9)
ax.set_xlabel("Operations per millisecond (ops/ms)")
ax.set_title("Nova GC Benchmarks — Throughput")
ax.invert_yaxis()
ax.grid(axis="x", alpha=0.3)
for bar, tp in zip(bars, throughputs):
    ax.text(bar.get_width() + 50, bar.get_y() + bar.get_height()/2,
            f"{tp:,.0f}", va="center", fontsize=8, color="#cdd6f4")
save(fig, "02_throughput.png")

# ---------------------------------------------------------------------------
# CHART 3 — Coefficient of Variation (consistency / jitter)
# ---------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(10, 6))
cov_data = []
for key, v in ordered:
    cv = (v["stdev_ms"] / v["mean_ms"]) * 100 if v["mean_ms"] > 0 else 0
    cov_data.append((v["desc"], cv, v["category"]))

cov_data.sort(key=lambda x: x[1], reverse=True)
clabels = [d[0] for d in cov_data]
cvals   = [d[1] for d in cov_data]
ccols   = [PALETTE.get(d[2], "#89b4fa") for d in cov_data]

y_pos = range(len(clabels))
bars = ax.barh(y_pos, cvals, color=ccols, edgecolor="#45475a", height=0.65)
ax.set_yticks(list(y_pos))
ax.set_yticklabels(clabels, fontsize=9)
ax.set_xlabel("Coefficient of Variation (%)")
ax.set_title("Nova GC Benchmarks — Run-to-Run Consistency\n(lower = more consistent)")
ax.invert_yaxis()
ax.grid(axis="x", alpha=0.3)
# threshold line at 5%
ax.axvline(5.0, color="#f38ba8", linestyle="--", alpha=0.6, label="5% threshold")
ax.legend(loc="lower right", fontsize=8, facecolor="#313244", edgecolor="#585b70")
for bar, cv in zip(bars, cvals):
    ax.text(bar.get_width() + 0.1, bar.get_y() + bar.get_height()/2,
            f"{cv:.1f}%", va="center", fontsize=8, color="#cdd6f4")
save(fig, "03_consistency.png")

# ---------------------------------------------------------------------------
# CHART 4 — GC overhead: heap-intensive vs baseline
# ---------------------------------------------------------------------------
baseline_mean = data["13_baseline"]["mean_ms"]
fig, ax = plt.subplots(figsize=(10, 6))
overhead_data = []
for key, v in simple.items():
    if key == "13_baseline":
        continue
    if key not in OPS:
        continue
    # normalize: time per 1M ops
    t_per_Mops = (v["mean_ms"] / OPS[key]) * 1_000_000
    b_per_Mops = (baseline_mean / OPS["13_baseline"]) * 1_000_000
    overhead = t_per_Mops / b_per_Mops
    overhead_data.append((v["desc"], overhead, v["category"]))

overhead_data.sort(key=lambda x: x[1])
olabels = [d[0] for d in overhead_data]
ovals   = [d[1] for d in overhead_data]
ocols   = [PALETTE.get(d[2], "#89b4fa") for d in overhead_data]

y_pos = range(len(olabels))
bars = ax.barh(y_pos, ovals, color=ocols, edgecolor="#45475a", height=0.65)
ax.set_yticks(list(y_pos))
ax.set_yticklabels(olabels, fontsize=9)
ax.set_xlabel("Slowdown vs pure-integer baseline (×)")
ax.set_title("Nova GC Benchmarks — Heap Overhead per Operation\n(normalized to 1M pure-int adds)")
ax.invert_yaxis()
ax.grid(axis="x", alpha=0.3)
ax.axvline(1.0, color="#a6e3a1", linestyle="--", alpha=0.6, label="baseline = 1×")
ax.legend(loc="lower right", fontsize=8, facecolor="#313244", edgecolor="#585b70")
for bar, ov in zip(bars, ovals):
    ax.text(bar.get_width() + 0.05, bar.get_y() + bar.get_height()/2,
            f"{ov:.1f}×", va="center", fontsize=8, color="#cdd6f4")
save(fig, "04_gc_overhead.png")

# ---------------------------------------------------------------------------
# CHART 5 — Latency distribution (bench_11 all_latencies_us)
# ---------------------------------------------------------------------------
if bench11 and "all_latencies_us" in bench11:
    latencies = bench11["all_latencies_us"]

    fig, axes = plt.subplots(1, 2, figsize=(14, 5))

    # 5a — Histogram
    ax = axes[0]
    ax.hist(latencies, bins=80, color="#cba6f7", edgecolor="#45475a", alpha=0.85)
    ax.set_xlabel("Latency per 5k-alloc batch (µs)")
    ax.set_ylabel("Count")
    ax.set_title("Allocation Latency Distribution\n(200 batches × 5 runs = 2500 samples)")
    ax.axvline(bench11["latency_p50_us"], color="#a6e3a1", linestyle="--", linewidth=1.5, label=f'p50 = {bench11["latency_p50_us"]:.0f} µs')
    ax.axvline(bench11["latency_p95_us"], color="#f9e2af", linestyle="--", linewidth=1.5, label=f'p95 = {bench11["latency_p95_us"]:.0f} µs')
    ax.axvline(bench11["latency_p99_us"], color="#f38ba8", linestyle="--", linewidth=1.5, label=f'p99 = {bench11["latency_p99_us"]:.0f} µs')
    ax.legend(fontsize=8, facecolor="#313244", edgecolor="#585b70")
    ax.grid(axis="y", alpha=0.3)

    # 5b — CDF (sorted latencies)
    ax = axes[1]
    sorted_lat = sorted(latencies)
    n = len(sorted_lat)
    cdf = [(i + 1) / n * 100 for i in range(n)]
    ax.plot(sorted_lat, cdf, color="#cba6f7", linewidth=1.5)
    ax.set_xlabel("Latency (µs)")
    ax.set_ylabel("Cumulative %")
    ax.set_title("Latency CDF")
    ax.axhline(50, color="#a6e3a1", linestyle=":", alpha=0.5)
    ax.axhline(95, color="#f9e2af", linestyle=":", alpha=0.5)
    ax.axhline(99, color="#f38ba8", linestyle=":", alpha=0.5)
    ax.grid(alpha=0.3)
    # annotate tail
    ax.annotate(f'max = {bench11["latency_max_us"]:.0f} µs',
                xy=(sorted_lat[-1], 100), fontsize=8, color="#f38ba8",
                xytext=(-80, -20), textcoords="offset points",
                arrowprops=dict(arrowstyle="->", color="#f38ba8"))

    fig.suptitle("Nova GC — Allocation Latency (bench_11: 200 batches of 5k allocs)",
                 fontsize=13, fontweight="bold", color="#cdd6f4")
    fig.tight_layout()
    save(fig, "05_latency.png")

# ---------------------------------------------------------------------------
# CHART 6 — Per-benchmark box plots (raw run times)
# ---------------------------------------------------------------------------
fig, ax = plt.subplots(figsize=(12, 6))
all_times = []
bx_labels = []
bx_colors = []
for key, v in ordered:
    all_times.append(v.get("times", v.get("wall_times", [])))
    bx_labels.append(v["desc"][:25])
    bx_colors.append(PALETTE.get(v["category"], "#89b4fa"))

bp = ax.boxplot(all_times, vert=False, patch_artist=True,
                boxprops=dict(edgecolor="#585b70"),
                whiskerprops=dict(color="#585b70"),
                capprops=dict(color="#585b70"),
                medianprops=dict(color="#f9e2af", linewidth=2),
                flierprops=dict(marker="o", markerfacecolor="#f38ba8", markersize=4))
for patch, col in zip(bp["boxes"], bx_colors):
    patch.set_facecolor(col)
    patch.set_alpha(0.7)
ax.set_yticklabels(bx_labels, fontsize=8)
ax.set_xlabel("Wall-clock time (ms)")
ax.set_title("Nova GC Benchmarks — Run Distribution (5 runs each)")
ax.invert_yaxis()
ax.grid(axis="x", alpha=0.3)
save(fig, "06_box_plots.png")

# ---------------------------------------------------------------------------
# CHART 7 — Radar / spider chart: multi-dimensional comparison
# ---------------------------------------------------------------------------
# Dimensions: speed, consistency, heap-friendliness, scalability proxy
# We'll pick the key benchmarks for a spider chart
spider_benches = [
    ("01_mass_alloc",     "Mass Alloc"),
    ("03_string_concat",  "String Cat"),
    ("04_struct_churn",   "Struct Churn"),
    ("06_deep_nesting",   "Deep Nest"),
    ("08_clone_deep",     "Deep Clone"),
    ("09_list_replace",   "List Mutate"),
]

# normalize each metric 0-1 (1 = best)
max_tp = max(OPS.get(k, 1) / data[k]["mean_ms"] for k, _ in spider_benches)
max_cv = max((data[k]["stdev_ms"] / data[k]["mean_ms"]) * 100 for k, _ in spider_benches)
max_ms = max(data[k]["mean_ms"] for k, _ in spider_benches)

fig, ax = plt.subplots(figsize=(8, 8), subplot_kw=dict(polar=True))
dims = ["Throughput", "Consistency", "Speed", "Low Jitter\n(max/min ratio)"]
n_dims = len(dims)
angles = [n * 2 * math.pi / n_dims for n in range(n_dims)]
angles += angles[:1]  # close

for key, label in spider_benches:
    v = data[key]
    tp_norm = (OPS.get(key, 1) / v["mean_ms"]) / max_tp
    cv = (v["stdev_ms"] / v["mean_ms"]) * 100
    cv_norm = 1 - (cv / max_cv)  # invert: lower cv = better
    speed_norm = 1 - (v["mean_ms"] / max_ms)
    jitter_norm = 1 - ((v["max_ms"] / v["min_ms"] - 1) / 0.25)  # 25% spread = 0
    jitter_norm = max(0, min(1, jitter_norm))
    values = [tp_norm, cv_norm, speed_norm, jitter_norm]
    values += values[:1]
    ax.plot(angles, values, linewidth=1.5, label=label)
    ax.fill(angles, values, alpha=0.08)

ax.set_xticks(angles[:-1])
ax.set_xticklabels(dims, fontsize=9)
ax.set_ylim(0, 1.05)
ax.set_title("Nova GC — Multi-Dimension Quality Profile", pad=20)
ax.legend(loc="upper right", bbox_to_anchor=(1.3, 1.1), fontsize=8,
          facecolor="#313244", edgecolor="#585b70")
save(fig, "07_radar.png")

# ---------------------------------------------------------------------------
# CHART 8 — Latency timeline: per-batch latency over time (bench_11)
# ---------------------------------------------------------------------------
if bench11 and "all_latencies_us" in bench11:
    latencies = bench11["all_latencies_us"]
    # We have 2500 samples = 5 runs * 200 batches per run (500 per run)
    per_run = len(latencies) // 5

    fig, ax = plt.subplots(figsize=(12, 4))
    run_colors = ["#89b4fa", "#f9e2af", "#a6e3a1", "#f38ba8", "#cba6f7"]
    for run_i in range(5):
        start = run_i * per_run
        end = start + per_run
        run_lats = latencies[start:end]
        xs = list(range(len(run_lats)))
        ax.plot(xs, run_lats, color=run_colors[run_i], alpha=0.6, linewidth=0.8,
                label=f"Run {run_i+1}")
    ax.set_xlabel("Batch index (each = 5k allocations)")
    ax.set_ylabel("Latency (µs)")
    ax.set_title("Nova GC — Per-Batch Latency Over Time (5 runs overlaid)")
    ax.legend(fontsize=8, facecolor="#313244", edgecolor="#585b70")
    ax.grid(alpha=0.3)
    # highlight the target frame budget (16ms = 16000µs)
    ax.axhline(16000, color="#f38ba8", linestyle="--", alpha=0.4, label="16ms frame budget")
    save(fig, "08_latency_timeline.png")

# ---------------------------------------------------------------------------
# SUMMARY TABLE (printed to stdout)
# ---------------------------------------------------------------------------
print("\n" + "=" * 90)
print("  NOVA GC/RC BENCHMARK SUMMARY")
print("=" * 90)
print(f"  GC Architecture: Hybrid Reference Counting + Mark-Sweep Cycle Collector")
print(f"  Threshold tuning: adaptive ({5000}–{1_000_000} objects), targets 16ms frame budget")
print(f"  Heap: Vec<Option<HeapEntry>> with free-list reuse + tail shrinking")
print("=" * 90)
print()
print(f"  {'Benchmark':<35} {'Mean ms':>10} {'Stdev':>8} {'Min':>10} {'Max':>10} {'Ops/ms':>10}")
print(f"  {'-'*35} {'-'*10} {'-'*8} {'-'*10} {'-'*10} {'-'*10}")

for key, v in ordered:
    ops = OPS.get(key, 0)
    tp_str = f"{ops / v['mean_ms']:,.0f}" if ops else "—"
    print(f"  {v['desc']:<35} {v['mean_ms']:>10.2f} {v['stdev_ms']:>8.2f} {v['min_ms']:>10.2f} {v['max_ms']:>10.2f} {tp_str:>10}")

if bench11:
    print()
    print(f"  --- Latency Profile (bench_11: 200 batches × 5k allocs × 5 runs) ---")
    print(f"  Samples:  {bench11['latency_count']}")
    print(f"  Mean:     {bench11['latency_mean_us']:.1f} µs")
    print(f"  Median:   {bench11['latency_median_us']:.1f} µs")
    print(f"  p90:      {bench11['latency_p90_us']:.1f} µs")
    print(f"  p95:      {bench11['latency_p95_us']:.1f} µs")
    print(f"  p99:      {bench11['latency_p99_us']:.1f} µs")
    print(f"  Max:      {bench11['latency_max_us']:.1f} µs")
    print(f"  Stdev:    {bench11['latency_stdev_us']:.1f} µs")
    p99_ratio = bench11["latency_p99_us"] / bench11["latency_median_us"]
    max_ratio = bench11["latency_max_us"] / bench11["latency_median_us"]
    print(f"  p99/p50:  {p99_ratio:.2f}×  (tail spike ratio)")
    print(f"  max/p50:  {max_ratio:.2f}×  (worst-case spike)")

print()
print("=" * 90)
print("  KEY FINDINGS")
print("=" * 90)

# Identify best/worst
fastest = ordered[0]
slowest = ordered[-1]
print(f"  Fastest:   {fastest[1]['desc']} — {fastest[1]['mean_ms']:.1f} ms")
print(f"  Slowest:   {slowest[1]['desc']} — {slowest[1]['mean_ms']:.1f} ms")

# Most consistent vs jittery
best_cv = min(simple.items(), key=lambda kv: kv[1]["stdev_ms"] / kv[1]["mean_ms"] if kv[1]["mean_ms"] > 0 else 999)
worst_cv = max(simple.items(), key=lambda kv: kv[1]["stdev_ms"] / kv[1]["mean_ms"] if kv[1]["mean_ms"] > 0 else 0)
print(f"  Most consistent:  {best_cv[1]['desc']} (CV = {best_cv[1]['stdev_ms']/best_cv[1]['mean_ms']*100:.1f}%)")
print(f"  Most jittery:     {worst_cv[1]['desc']} (CV = {worst_cv[1]['stdev_ms']/worst_cv[1]['mean_ms']*100:.1f}%)")

# String concat pain
sc = data["03_string_concat"]
bl = data["13_baseline"]
sc_per_op = sc["mean_ms"] / 10_000
bl_per_op = bl["mean_ms"] / 1_000_000
slowdown = sc_per_op / bl_per_op
print(f"  String concat per-op cost: {sc_per_op*1000:.1f} µs/op ({slowdown:.0f}× vs int baseline)")

if bench11:
    if bench11["latency_max_us"] < 1000:
        print(f"  Latency verdict: ✅ Excellent — max spike {bench11['latency_max_us']:.0f} µs (< 1ms)")
    elif bench11["latency_max_us"] < 5000:
        print(f"  Latency verdict: ⚠️  Acceptable — max spike {bench11['latency_max_us']:.0f} µs (< 5ms)")
    else:
        print(f"  Latency verdict: ❌ Needs work — max spike {bench11['latency_max_us']:.0f} µs (> 5ms)")

# Memory overhead
list_replace = data["09_list_replace"]
if list_replace["stdev_ms"] / list_replace["mean_ms"] > 0.05:
    print(f"  ⚠️  list_replace has high variance ({list_replace['stdev_ms']:.1f}ms stdev) — possible GC pauses during mutation")

print()
print(f"  Charts saved to: {OUT_DIR}/")
print("=" * 90)
