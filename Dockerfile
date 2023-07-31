FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /home/rust/

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

RUN rustc --version; cargo --version; rustup --version

RUN apt-get update && apt-get install -y \
  protobuf-compiler \
  && rm -rf /var/lib/apt/lists/*

COPY --from=planner /home/rust/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

USER root

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
  chmod +x tailwindcss-linux-x64 && \
  mv tailwindcss-linux-x64 tailwindcss

COPY . .

COPY tailwind.config.js .
RUN ./tailwindcss -i server/src/styles/tailwind.css -o target/tailwind.css

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
COPY ./server/litefs.yaml /etc/litefs.yaml

EXPOSE 3000

ENTRYPOINT ["litefs", "mount"]
