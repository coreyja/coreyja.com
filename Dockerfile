FROM rust as builder

WORKDIR /home/rust/

USER root

RUN rustc --version; cargo --version; rustup --version

RUN apt-get update && apt-get install -y \
  protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*

# Avoid having to install/build all dependencies by copying
# the Cargo files and making a dummy src/main.rs
COPY Cargo.toml .
COPY Cargo.lock .

COPY server/Cargo.toml ./server/
RUN mkdir -p ./server/src/ && echo "fn main() {}" > ./server/src/main.rs

RUN cargo build --release --locked --bin server

# We need to touch our real main.rs file or else docker will use
# the cached one.
COPY server/ server/
RUN touch server/src/main.rs

RUN cargo build --release --locked --bin server

# Download the static build of Litestream directly into the path & make it executable.
# This is done in the builder and copied as the chmod doubles the size.
ADD https://github.com/benbjohnson/litestream/releases/download/v0.3.9/litestream-v0.3.9-linux-amd64-static.tar.gz /tmp/litestream.tar.gz
RUN tar -C /usr/local/bin -xzf /tmp/litestream.tar.gz

# Start building the final image
FROM debian:bullseye-slim as final
WORKDIR /home/rust/

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && update-ca-certificates

COPY --from=builder /home/rust/target/release/server .

COPY --from=builder /usr/local/bin/litestream /usr/local/bin/litestream

COPY ./server/litestream.yaml /etc/litestream.yaml

EXPOSE 3000

ENTRYPOINT ["litestream", "replicate", "--config", "/etc/litestream.yaml", "--exec", "./server"]
