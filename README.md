# deathroll.gg 💀🎲 

https://deathroll.gg

learning rust/wasm by building Full stack rust game. Yew for frontend, Axum for webserver.

deathrolling is a game made famous by world of warcraft, where players  deathroll for gold. 

https://youtu.be/vshLQqwfnjc?t=1044 

## rules 

- Players take turns rolling a die.
- The first player selects a number, and then rolls the die. The number they roll becomes the maximum number for the next player's roll.
- If a player rolls a 1, they lose the game.

## build 

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



