#!/bin/bash
set -e

cargo test
$(dirname "$0")/clippy-pedantic.sh
cargo doc --no-deps
cargo deadlinks
