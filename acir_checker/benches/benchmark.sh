#!/bin/bash

# This script runs the benchmarks for the ACIR checker and saves the results in a directory specified by the first argument. 
# The results are saved in JSON format and can be compared using the comparison.py script.
BENCH_TAG=$1
DIR="$(dirname $(realpath ${BASH_SOURCE[0]}))"
# Command to benchmark
CMD="${DIR}/../../../target/debug/nargo formal-verify"
TIMEOUT_SECS=90
BENCHES_PATH="${DIR}/../test_programs/benches"
RESULTS_PATH="${DIR}/${BENCH_TAG}"

if [ -d "$RESULTS_PATH" ]; then
    echo "Benchmarks directory already exists; aborting benchmark creation"
    exit 1
fi

mkdir -p "$RESULTS_PATH"

FILES="$BENCHES_PATH/*" 
echo FILES: $FILES
for bench_path in $FILES; do
    name=$(basename "$bench_path")
    if [ -d "$bench_path" ]; then
        cd "$bench_path"
        hyperfine "timeout $TIMEOUT_SECS $CMD" -i --warmup 1 --runs 5 --export-json "$RESULTS_PATH/${name}.json"
    fi
done



