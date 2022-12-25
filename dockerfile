FROM rust:latest as build

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk wasm-bindgen-cli

WORKDIR /src/
COPY . .

RUN cd frontend && CARGO_TARGET_DIR=../target-trunk trunk build --release --public-url ./assets/
RUN cd server && cargo build --release

FROM gcr.io/distroless/cc-debian10

COPY --from=build /src/target/release/server /usr/local/bin/server
COPY --from=build /src/dist /usr/local/dist

WORKDIR /usr/local/bin

CMD ["server"]
