#!/bin/sh

set -e

export RUST_BACKTRACE=1

cargo fmt -- --write-mode=diff

for i in {1..100}; do
    echo
    echo "Running tests (attempt #${i})"
    echo
    cargo test --verbose
done
