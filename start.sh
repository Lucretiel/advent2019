set -ex

rm src/main.rs
cp src/template.rs src/"$1".rs
ln -s src/"$1".rs src/main.rs
