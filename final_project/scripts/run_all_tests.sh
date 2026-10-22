#!/bin/bash

set -e

TRIALS=50
THREADS=4

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_DIR/results/raw_data"

MC_C="$PROJECT_DIR/montecarlo/c/montecarlo"
MC_RUST="$PROJECT_DIR/montecarlo/rust/target/release/montecarlo"
MAT_C="$PROJECT_DIR/matrix/c/matrix"
MAT_RUST="$PROJECT_DIR/matrix/rust/target/release/matrix"
DNS_C="$PROJECT_DIR/dns/c/dns"
DNS_RUST="$PROJECT_DIR/dns/rust/target/release/dns"

DNS_OUTFILE="/tmp/dns_bench_out.txt"

#extract elapsed time from program output
extract_time() {
    echo "$1" | grep "Elapsed time" | awk '{print $3}'
}

#run benchmark and write csv
run_benchmark() {
    local csv="$1"
    local name="$2"
    shift 2
    local cmd=("$@")

    echo "  Running: $name ($TRIALS trials)"
    echo "trial,time_seconds" > "$csv"

    for i in $(seq 1 $TRIALS); do
        output=$("${cmd[@]}" 2>/dev/null)
        t=$(extract_time "$output")

        if [ -z "$t" ]; then
            echo "  WARNING: trial $i returned empty time for $name"
            t="NA"
        fi

        echo "$i,$t" >> "$csv"

        if (( i % 10 == 0 )); then
            echo "    ... $i/$TRIALS done"
        fi
    done

    echo "  Saved -> $csv"
}

echo "============================================================"
echo " Benchmark Runner"
echo " Trials:  $TRIALS"
echo " Threads: $THREADS"
echo "============================================================"
echo ""

echo "[ Checking binaries ]"

MISSING=0

for bin in "$MC_C" "$MC_RUST" "$MAT_C" "$MAT_RUST" "$DNS_C" "$DNS_RUST"; do
    if [ ! -f "$bin" ]; then
        echo "  MISSING: $bin"
        MISSING=1
    else
        echo "  OK:      $bin"
    fi
done

if [ "$MISSING" -eq 1 ]; then
    echo ""
    echo "ERROR: build binaries first"
    exit 1
fi

mkdir -p "$RESULTS_DIR"
echo ""

# Monte Carlo
echo "[ Monte Carlo ]"

run_benchmark "$RESULTS_DIR/montecarlo_c_1m.csv" \
    "C / 1M" \
    "$MC_C" 1000000 $THREADS

run_benchmark "$RESULTS_DIR/montecarlo_rust_1m.csv" \
    "Rust / 1M" \
    "$MC_RUST" 1000000 $THREADS

run_benchmark "$RESULTS_DIR/montecarlo_c_10m.csv" \
    "C / 10M" \
    "$MC_C" 10000000 $THREADS

run_benchmark "$RESULTS_DIR/montecarlo_rust_10m.csv" \
    "Rust / 10M" \
    "$MC_RUST" 10000000 $THREADS

run_benchmark "$RESULTS_DIR/montecarlo_c_100m.csv" \
    "C / 100M" \
    "$MC_C" 100000000 $THREADS

run_benchmark "$RESULTS_DIR/montecarlo_rust_100m.csv" \
    "Rust / 100M" \
    "$MC_RUST" 100000000 $THREADS

echo ""

# Matrix
echo "[ Matrix ]"

run_benchmark "$RESULTS_DIR/matrix_c_256.csv" \
    "C / 256" \
    "$MAT_C" 256 $THREADS

run_benchmark "$RESULTS_DIR/matrix_rust_256.csv" \
    "Rust / 256" \
    "$MAT_RUST" 256 $THREADS

run_benchmark "$RESULTS_DIR/matrix_c_512.csv" \
    "C / 512" \
    "$MAT_C" 512 $THREADS

run_benchmark "$RESULTS_DIR/matrix_rust_512.csv" \
    "Rust / 512" \
    "$MAT_RUST" 512 $THREADS

run_benchmark "$RESULTS_DIR/matrix_c_1024.csv" \
    "C / 1024" \
    "$MAT_C" 1024 $THREADS

run_benchmark "$RESULTS_DIR/matrix_rust_1024.csv" \
    "Rust / 1024" \
    "$MAT_RUST" 1024 $THREADS

echo ""

# DNS
echo "[ DNS ]"

run_benchmark "$RESULTS_DIR/dns_c_50.csv" \
    "C / 50" \
    "$DNS_C" 50 $THREADS "$DNS_OUTFILE"

run_benchmark "$RESULTS_DIR/dns_rust_50.csv" \
    "Rust / 50" \
    "$DNS_RUST" 50 $THREADS "$DNS_OUTFILE"

run_benchmark "$RESULTS_DIR/dns_c_200.csv" \
    "C / 200" \
    "$DNS_C" 200 $THREADS "$DNS_OUTFILE"

run_benchmark "$RESULTS_DIR/dns_rust_200.csv" \
    "Rust / 200" \
    "$DNS_RUST" 200 $THREADS "$DNS_OUTFILE"

run_benchmark "$RESULTS_DIR/dns_c_500.csv" \
    "C / 500" \
    "$DNS_C" 500 $THREADS "$DNS_OUTFILE"

run_benchmark "$RESULTS_DIR/dns_rust_500.csv" \
    "Rust / 500" \
    "$DNS_RUST" 500 $THREADS "$DNS_OUTFILE"

echo ""
echo "Done. Results in: $RESULTS_DIR"