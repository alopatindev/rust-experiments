#!/bin/sh

cat README.md.template
find src -type f -name '*.rs' | \
    egrep -v '/(lib|mod).rs$' | \
    grep -v '/cli/' | \
    sed 's!^src/!* !' | \
    sort -u
