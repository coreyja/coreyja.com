#!/usr/bin/env bash

set -e

pushd "$(git rev-parse --show-toplevel)" || exit
  cargo sqlx prepare --all --workspace -- --all-targets
  cargo clippy --all-targets --all-features --workspace --tests --allow-dirty --fix
  cargo fmt
popd || exit
