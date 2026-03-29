#!/usr/bin/env bash
# Nova Language Test Runner
# Runs all test_*.nv files and reports pass/fail

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
NOVA="$PROJECT_DIR/target/release/nova"

if [ ! -f "$NOVA" ]; then
    echo "ERROR: Nova binary not found at $NOVA"
    echo "       Run 'cargo build --release' first."
    exit 1
fi

PASS=0
FAIL=0
FAILURES=()

# Collect all test files
TEST_FILES=("$SCRIPT_DIR"/test_*.nv)

echo "========================================"
echo "  Nova Language Test Suite"
echo "========================================"
echo "Running ${#TEST_FILES[@]} test files..."
echo ""

for test_file in "${TEST_FILES[@]}"; do
    test_name="$(basename "$test_file" .nv)"
    
    # Run the test and capture output + exit code
    output=$("$NOVA" run "$test_file" 2>&1) && exit_code=0 || exit_code=$?
    
    if [ $exit_code -eq 0 ] && echo "$output" | grep -q "^PASS:"; then
        echo "  ✓ $test_name"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $test_name"
        FAILURES+=("$test_name")
        FAIL=$((FAIL + 1))
        # Show first few lines of output for debugging
        if [ -n "$output" ]; then
            echo "    Output:"
            echo "$output" | head -20 | sed 's/^/      /'
            lines=$(echo "$output" | wc -l)
            if [ "$lines" -gt 20 ]; then
                echo "      ... ($((lines - 20)) more lines)"
            fi
        fi
        echo ""
    fi
done

echo ""
echo "========================================"
echo "  Results: $PASS passed, $FAIL failed"
echo "  Total:   $((PASS + FAIL)) tests"
echo "========================================"

if [ $FAIL -gt 0 ]; then
    echo ""
    echo "Failed tests:"
    for f in "${FAILURES[@]}"; do
        echo "  - $f"
    done
    exit 1
fi

echo ""
echo "All tests passed! ✓"
exit 0
