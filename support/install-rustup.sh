#!/bin/bash
# Usage: install-rustup.sh <toolchain>

set -e

rustup_file=$(mktemp -p .)

curl https://sh.rustup.rs -sSf -o $rustup_file && chmod +x $rustup_file
$rustup_file -y

rustup override set $1
rustup component add rust-src

cargo install xargo

# Cleanup
rm $rustup_file
