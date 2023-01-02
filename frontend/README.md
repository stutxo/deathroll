# deathroll frontend

Project to learn yew/rust/wasm for frontend

To build, install rust/trunk and add wasm target, 

```
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```
Then build with the commands below

```
RUSTFLAGS=--cfg=web_sys_unstable_apis CARGO_TARGET_DIR=../target-trunk trunk build --release --public-url ./assets/
```

To run with the server/frontend, you can use the prod.sh script in the root of the repository. or build the docker image using the dockerfile in the root repository. 