#!/bin/sh

version=31
wget "https://github.com/SimonKagstrom/kcov/archive/v${version}.tar.gz"
tar xzf "v${version}.tar.gz"
mv "kcov-${version}" kcov
cd kcov
mkdir build
cd build
cmake ..
make -j2
cd ../..
