#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --locked -q -- -D warnings
cargo test --locked -q
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps -q
