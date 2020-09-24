# Dockerfile for our SDK worker and data generator

FROM rust:1.46

RUN apt-get update && apt-get install -y \
    cmake protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/statefun
COPY . .

RUN cargo install --path statefun-example

ENV RUST_LOG debug

CMD ["datagen"]