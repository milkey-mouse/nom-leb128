#!/bin/sh
set -euo pipefail

cargo fuzz build
find fuzz_targets -type f -exec basename -s.rs {} + | \
    grep -v macros | \
    nice -n19 xargs -n1 -P0 \
        cargo fuzz run "$@"
