#!/bin/bash

set -ex

cd $(dirname "${BASH_SOURCE[0]}")

rm -rf js
mkdir js

rm -rf rust
mkdir rust

rm -rf python
mkdir python



cargo run -p wit-bindgen-cli -- rust-wasm \
  --export ./interfaces/stringfuncs.wit \
  --import ./interfaces/printer.wit \
  --out-dir rust 

cargo build -p wit-bindgen-example --target wasm32-unknown-unknown --release
cp ../../../target/wasm32-unknown-unknown/release/wit_bindgen_example.wasm dist/rust.wasm

cargo run -p wit-bindgen-cli -- js \
  --export ./interfaces/printer.wit \
  --import ./interfaces/stringfuncs.wit \
  --out-dir dist \
&& yarn tsc

# cp crates/wit-bindgen-demo/{index.html,main.ts} dist/
# (cd crates/wit-bindgen-demo && npx tsc ../../dist/main.ts --target es6)

