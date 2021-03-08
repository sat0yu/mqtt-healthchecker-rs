FROM rust:1.50.0

RUN apt-get update && apt-get install -y cmake

RUN cargo install cargo-edit
RUN rustup component add rustfmt

