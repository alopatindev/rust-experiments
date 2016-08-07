#!/bin/sh

cat README.md.template
find src -type f -name '*.rs' | egrep -v '/(lib|mod).rs$' | sed 's!^src/!* !' | sort -u
