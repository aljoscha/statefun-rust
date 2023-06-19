# Dockerfile for our SDK worker and data generator

FROM rust:1.70

RUN apt-get update && apt-get install -y \
    cmake protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/statefun
COPY . .

RUN cargo install --path statefun-advanced-example

ENV RUST_LOG debug
EXPOSE 1108

CMD ["statefun-advanced-example"]
