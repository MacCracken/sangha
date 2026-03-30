#!/usr/bin/env bash
set -euo pipefail

HISTORY_FILE="${1:-bench-history.csv}"
BENCHMARKS_MD="benchmarks.md"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")

if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,commit,branch,benchmark,estimate_ns" > "$HISTORY_FILE"
fi

echo "Running sangha benchmarks..."
BENCH_OUTPUT=$(cargo bench --all-features 2>&1 | sed 's/\x1b\[[0-9;]*m//g')
echo "$BENCH_OUTPUT"

declare -a BENCH_NAMES=()
declare -a BENCH_NS=()

PREV_LINE=""
while IFS= read -r line; do
    if [[ "$line" == *"time:"*"["* ]]; then
        BENCH_NAME=$(echo "$line" | sed -E 's/[[:space:]]*time:.*//' | xargs)
        if [ -z "$BENCH_NAME" ]; then
            BENCH_NAME=$(echo "$PREV_LINE" | xargs)
        fi
        VALS=$(echo "$line" | sed -E 's/.*\[(.+)\]/\1/')
        MEDIAN=$(echo "$VALS" | awk '{print $3}')
        UNIT=$(echo "$VALS" | awk '{print $4}')
        case "$UNIT" in
            ps)  NS=$(echo "$MEDIAN" | awk '{printf "%.4f", $1 / 1000}') ;;
            ns)  NS="$MEDIAN" ;;
            µs|us)  NS=$(echo "$MEDIAN" | awk '{printf "%.4f", $1 * 1000}') ;;
            ms)  NS=$(echo "$MEDIAN" | awk '{printf "%.4f", $1 * 1000000}') ;;
            s)   NS=$(echo "$MEDIAN" | awk '{printf "%.4f", $1 * 1000000000}') ;;
            *)   NS="$MEDIAN" ;;
        esac
        echo "${TIMESTAMP},${COMMIT},${BRANCH},${BENCH_NAME},${NS}" >> "$HISTORY_FILE"
        BENCH_NAMES+=("$BENCH_NAME")
        BENCH_NS+=("$NS")
    fi
    PREV_LINE="$line"
done <<< "$BENCH_OUTPUT"

COUNT=${#BENCH_NAMES[@]}
echo "${COUNT} benchmarks recorded"
