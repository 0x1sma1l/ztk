#!/usr/bin/env bash
set -euo pipefail

cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo package --locked --allow-dirty
