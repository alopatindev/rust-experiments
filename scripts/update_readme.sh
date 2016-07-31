#!/bin/sh

echo '[![Build Status](https://api.travis-ci.org/alopatindev/rust_experiments.svg?branch=master)](https://travis-ci.org/alopatindev/rust_experiments)'
echo
echo '"Hello Worlds" in Rust'
echo

find src -type f -name '*.rs' | egrep -v '/(lib|mod).rs$' | sed 's!^src/!* !' | sort -u
