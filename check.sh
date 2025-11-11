#!/bin/bash
# CI-style checks: format, lint, test
set -e

echo "Running fmt..."
cargo fmt --all

echo "Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings

echo "Running tests..."
cargo test --workspace

echo "All checks passed!"
