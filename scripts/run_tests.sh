#!/bin/bash

set -e

cargo fmt -- --write-mode=diff

for i in {1..20}; do
    echo
    echo "Running tests (attempt #${i})"
    echo
    cargo test ${CARGO_ARGS} --verbose
done
