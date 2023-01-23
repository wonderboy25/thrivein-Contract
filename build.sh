#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir -p ./out
copy target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm
