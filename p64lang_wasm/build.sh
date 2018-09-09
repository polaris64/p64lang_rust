cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/p64lang_wasm.wasm --out-dir ./web
