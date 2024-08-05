FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
RUN rustc --version; cargo --version; rustup --version

RUN apt-get update && apt-get install -y \
  protobuf-compiler libasound2-dev clang cmake \
  && rm -rf /var/lib/apt/lists/*

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64 && \
  chmod +x tailwindcss-linux-x64 && \
  mv tailwindcss-linux-x64 tailwindcss

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --bin server --recipe-path recipe.json
COPY . .

COPY tailwind.config.js .
RUN ./tailwindcss -i server/src/styles/tailwind.css -o target/tailwind.css

RUN cargo build --release --locked --bin server

# Start building the final image
FROM debian:stable-slim as final
WORKDIR /app

RUN apt-get update && apt-get install -y \
  ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && update-ca-certificates

COPY --from=builder /app/target/release/server .

EXPOSE 3000

ENTRYPOINT ["./server"]
