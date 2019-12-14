#!/bin/sh

set -ex

rm -f src/main.rs
cp -nv src/template.rs src/"$1".rs
ln -s "$1".rs src/main.rs
