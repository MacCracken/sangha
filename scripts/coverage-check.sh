#!/usr/bin/env bash
set -euo pipefail

# Run cargo-llvm-cov and fail if line coverage drops below the threshold.
#
# Usage:
#   ./scripts/coverage-check.sh          # default threshold: 70%
#   ./scripts/coverage-check.sh 75       # custom threshold
#
# Requires: cargo-llvm-cov (cargo install cargo-llvm-cov)

THRESHOLD="${1:-70}"

echo "Running coverage check (threshold: ${THRESHOLD}%)"
echo ""

OUTPUT=$(cargo llvm-cov --all-features 2>&1)

COVERAGE_LINE=$(echo "$OUTPUT" | grep 'TOTAL' | tail -1)
COVERAGE_PCT=$(echo "$COVERAGE_LINE" | awk '{for(i=1;i<=NF;i++) if($i ~ /[0-9]+\.[0-9]+%/) {gsub(/%/,"",$i); print $i; exit}}')

if [ -z "$COVERAGE_PCT" ]; then
    echo "Could not parse coverage from output"
    echo "$OUTPUT"
    exit 1
fi

echo "Coverage: ${COVERAGE_PCT}%"
echo ""

PASS=$(echo "$COVERAGE_PCT $THRESHOLD" | awk '{print ($1 >= $2) ? 1 : 0}')

if [ "$PASS" -eq 1 ]; then
    echo "PASS: ${COVERAGE_PCT}% >= ${THRESHOLD}% threshold"
    exit 0
else
    echo "FAIL: ${COVERAGE_PCT}% < ${THRESHOLD}% threshold"
    exit 1
fi
