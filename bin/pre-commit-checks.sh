#!/bin/bash
set -e

cargo test
$(dirname "$0")/clippy-pedantic.sh
cargo doc
cargo deadlinks
