#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

pushd frontend
RUSTFLAGS=--cfg=web_sys_unstable_apis CARGO_TARGET_DIR=../target-trunk trunk build --release --public-url ./assets/
popd

pushd server
cargo run --release
popd