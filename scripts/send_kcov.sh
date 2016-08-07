#!/bin/bash

PKGID="$(cargo pkgid)"
[ -z "$PKGID" ] && exit 1
ORIGIN="${PKGID%#*}"
ORIGIN="${ORIGIN:7}"
PKGNAMEVER="${PKGID#*#}"
PKGNAME="${PKGNAMEVER%:*}"
shift
cargo test --no-run || exit $?
EXE=($ORIGIN/target/debug/$PKGNAME-*)
if [ ${#EXE[@]} -ne 1 ]; then
    echo 'Non-unique test file, retrying...' >2
    rm -f ${EXE[@]}
    cargo test --no-run || exit $?
fi
rm -rf $ORIGIN/target/cov

kcov/build/src/kcov $ORIGIN/target/cov $ORIGIN/target/debug/$PKGNAME-* "$@"
