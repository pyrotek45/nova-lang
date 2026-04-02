#!/usr/bin/env bash
# Nova Performance Benchmark Runner
# ==================================
# Runs all perf_*.nv files in tests/perf/ and reports timing.
#
# Usage:
#   bash tests/perf/run_perf.sh           # run all benchmarks
#   bash tests/perf/run_perf.sh quick     # run only (skip slow ones)
#
# Each benchmark file prints timing data and exits 0 on success.
# This script collects results and produces a summary table.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
NOVA="$PROJECT_DIR/target/release/nova"

if [ ! -f "$NOVA" ]; then
    echo "ERROR: Nova binary not found at $NOVA"
    echo "       Run 'cargo build --release' first."
    exit 1
fi

PASS=0
FAIL=0
FAILURES=()

BENCH_FILES=("$SCRIPT_DIR"/perf_*.nv)

echo "════════════════════════════════════════════════"
echo "  Nova Performance Benchmark Suite"
echo "════════════════════════════════════════════════"
echo "Running ${#BENCH_FILES[@]} benchmark files..."
echo ""

total_start=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')

for bench_file in "${BENCH_FILES[@]}"; do
    bench_name="$(basename "$bench_file" .nv)"

    echo "── $bench_name ──"
    output=$("$NOVA" run "$bench_file" 2>&1) && exit_code=0 || exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo "$output" | grep -E "^\s+(BENCH|TOTAL)" | head -30
        PASS=$((PASS + 1))
    else
        echo "  ✗ FAILED"
        echo "$output" | head -15 | sed 's/^/    /'
        FAILURES+=("$bench_name")
        FAIL=$((FAIL + 1))
    fi
    echo ""
done

total_end=$(date +%s%N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1e9))')
total_ms=$(( (total_end - total_start) / 1000000 ))

echo "════════════════════════════════════════════════"
echo "  Results: $PASS passed, $FAIL failed"
echo "  Total wall time: ${total_ms}ms"
echo "════════════════════════════════════════════════"

if [ ${#FAILURES[@]} -gt 0 ]; then
    echo ""
    echo "Failed benchmarks:"
    for f in "${FAILURES[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

echo ""
echo "All benchmarks passed! ✓"
exit 0
