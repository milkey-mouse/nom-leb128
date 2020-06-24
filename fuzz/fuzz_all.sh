#!/bin/sh
set -euo pipefail

cargo fuzz build
cargo fuzz list | nice -n19 xargs -n1 -P0 cargo fuzz run "$@"
