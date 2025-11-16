@echo off
echo Building for wasm

cargo build --release --target wasm32-unknown-unknown
echo Generating bindings...
wasm-bindgen --out-dir ./out --target web ./target/wasm32-unknown-unknown/release/Voxxel-Engine.wasm

echo Finished!