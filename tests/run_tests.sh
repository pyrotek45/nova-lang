#!/usr/bin/env bash
# Nova Language Test Runner
# Runs all test_*.nv files and reports pass/fail
# Also runs "should_fail" tests that must be rejected by the compiler

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

# ============================================================
# POSITIVE TESTS: programs that must compile and print PASS:
# ============================================================
TEST_FILES=("$SCRIPT_DIR"/test_*.nv)

echo "========================================"
echo "  Nova Language Test Suite"
echo "========================================"
echo "Running ${#TEST_FILES[@]} positive test files..."
echo ""

for test_file in "${TEST_FILES[@]}"; do
    test_name="$(basename "$test_file" .nv)"

    output=$("$NOVA" run "$test_file" 2>&1) && exit_code=0 || exit_code=$?

    if [ $exit_code -eq 0 ] && echo "$output" | grep -q "^PASS:"; then
        echo "  ✓ $test_name"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $test_name"
        FAILURES+=("$test_name")
        FAIL=$((FAIL + 1))
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

# ============================================================
# NEGATIVE TESTS: programs that MUST be rejected (non-zero exit)
# These validate that the type system/parser correctly rejects
# ill-typed or syntactically invalid programs.
# ============================================================
SF_DIR="$SCRIPT_DIR/should_fail"
SF_PASS=0
SF_FAIL=0
SF_FAILURES=()

if [ -d "$SF_DIR" ]; then
    SF_FILES=("$SF_DIR"/*.nv)
    echo ""
    echo "Running ${#SF_FILES[@]} type-rejection tests (should_fail/)..."
    echo ""

    for test_file in "${SF_FILES[@]}"; do
        test_name="$(basename "$test_file" .nv)"

        # Run and capture — we expect a NON-zero exit code
        output=$("$NOVA" run "$test_file" 2>&1) && exit_code=0 || exit_code=$?

        if [ $exit_code -ne 0 ]; then
            echo "  ✓ $test_name  (correctly rejected)"
            SF_PASS=$((SF_PASS + 1))
        else
            echo "  ✗ $test_name  (SHOULD HAVE FAILED but passed!)"
            SF_FAILURES+=("$test_name")
            SF_FAIL=$((SF_FAIL + 1))
            if [ -n "$output" ]; then
                echo "    Output:"
                echo "$output" | head -10 | sed 's/^/      /'
            fi
            echo ""
        fi
    done
fi

# ============================================================
# Summary
# ============================================================
TOTAL_PASS=$((PASS + SF_PASS))
TOTAL_FAIL=$((FAIL + SF_FAIL))

echo ""
echo "========================================"
echo "  Positive tests: $PASS passed, $FAIL failed"
if [ -d "$SF_DIR" ]; then
    echo "  Rejection tests: $SF_PASS passed, $SF_FAIL failed"
fi
echo "  Total: $TOTAL_PASS passed, $TOTAL_FAIL failed"
echo "========================================"

if [ ${#FAILURES[@]} -gt 0 ]; then
    echo ""
    echo "Failed positive tests:"
    for f in "${FAILURES[@]}"; do
        echo "  - $f"
    done
fi

if [ ${#SF_FAILURES[@]} -gt 0 ]; then
    echo ""
    echo "Rejection tests that incorrectly PASSED:"
    for f in "${SF_FAILURES[@]}"; do
        echo "  - $f"
    done
fi

echo ""
if [ $TOTAL_FAIL -eq 0 ]; then
    echo "All tests passed! ✓"
    exit 0
else
    echo "$TOTAL_FAIL test(s) failed."
    exit 1
fi


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
