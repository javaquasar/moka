#!/usr/bin/env sh
set -eu

operations="${1:-20000}"
repetitions="${2:-5}"
bench_root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH= cd -- "$bench_root/../.." && pwd)
source_revision=$(git -C "$repo_root" rev-parse HEAD)
if [ -n "$(git -C "$repo_root" status --porcelain)" ]; then source_dirty=true; else source_dirty=false; fi
results="$bench_root/results"
mkdir -p "$results"

cargo build --release --manifest-path "$bench_root/Cargo.toml"
binary="$bench_root/target/release/moka-post-removal-observer-bench"
csv="$results/raw.csv"
metadata="$results/metadata.txt"
printf '%s\n' 'repetition_index,mode,operation,operations,gross_bytes,allocations,bytes_per_operation,allocations_per_operation' >"$csv"

configurations='off insert
listener insert
observer insert
off remove
listener remove
observer remove'

repetition=1
while [ "$repetition" -le "$repetitions" ]; do
    rotation=$(( (repetition - 1) % 6 ))
    printf '%s\n' "$configurations" | awk -v rotation="$rotation" '{ line[NR]=$0 } END { for (i=1; i<=NR; i++) print line[((i + rotation - 1) % NR) + 1] }' |
    while read -r mode operation; do
        row=$($binary "$mode" "$operation" "$operations")
        printf '%s,%s\n' "$repetition" "$row" >>"$csv"
    done
    repetition=$((repetition + 1))
done

{
    printf 'source_revision=%s\n' "$source_revision"
    printf 'source_dirty=%s\n' "$source_dirty"
    rustc --version --verbose
    uname -a
    printf 'operations=%s\nrepetitions=%s\n' "$operations" "$repetitions"
} >"$metadata"

printf '%s\n%s\n' "$csv" "$metadata"
