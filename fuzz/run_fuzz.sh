#!/usr/bin/env bash
# Nova Fuzzer Runner
# Requires: cargo-fuzz (install with: cargo +nightly install cargo-fuzz)
#
# Usage:
#   ./fuzz/run_fuzz.sh lexer              # Fuzz the lexer
#   ./fuzz/run_fuzz.sh parser             # Fuzz the parser
#   ./fuzz/run_fuzz.sh full               # Fuzz the full pipeline
#   ./fuzz/run_fuzz.sh all                # Fuzz all targets in parallel
#   ./fuzz/run_fuzz.sh lexer 60           # Fuzz lexer for 60 seconds
#
# The fuzzer will report any panics as crashes.
# Crash inputs are saved to fuzz/artifacts/<target>/

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

TARGET="${1:-lexer}"
DURATION="${2:-30}"   # seconds per target

# Map short name to cargo-fuzz target name
case "$TARGET" in
    lexer)  TARGETS=("fuzz_lexer") ;;
    parser) TARGETS=("fuzz_parser") ;;
    full)   TARGETS=("fuzz_full_pipeline") ;;
    all)    TARGETS=("fuzz_lexer" "fuzz_parser" "fuzz_full_pipeline") ;;
    *)
        echo "Usage: $0 [lexer|parser|full|all] [seconds=30]"
        exit 1
        ;;
esac

cd "$PROJECT_DIR"

# Ensure nightly is available
if ! rustup toolchain list | grep -q nightly; then
    echo "ERROR: Rust nightly toolchain required."
    echo "       Install with: rustup toolchain install nightly"
    exit 1
fi

# Ensure cargo-fuzz is installed
if ! cargo +nightly fuzz --version &>/dev/null; then
    echo "Installing cargo-fuzz..."
    cargo +nightly install cargo-fuzz
fi

for target in "${TARGETS[@]}"; do
    corpus_dir="fuzz/corpus/$target"
    mkdir -p "$corpus_dir"

    echo "========================================"
    echo "  Fuzzing: $target"
    echo "  Duration: ${DURATION}s"
    echo "  Corpus: $corpus_dir"
    echo "========================================"

    cargo +nightly fuzz run \
        --fuzz-dir fuzz \
        "$target" \
        "$corpus_dir" \
        -- -max_total_time="$DURATION" \
           -print_final_stats=1 \
        2>&1 | grep -v "^INFO: " || true

    echo ""
    echo "Done fuzzing $target."
    echo ""
done

echo "Fuzzing complete."
echo "Crash inputs (if any) are in fuzz/artifacts/"
