#!/bin/sh

set -ex

cp -nv src/template.rs src/"$1".rs
rm -f src/main.rs
ln -s "$1".rs src/main.rs
