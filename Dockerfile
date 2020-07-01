FROM rust:1.44.1

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/statefun
COPY . .

RUN cargo install --path statefun-example

ENV RUST_LOG debug

CMD ["statefun-example"]
