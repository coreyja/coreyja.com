#!/usr/bin/env bash



pushd "$(git rev-parse --show-toplevel)" || exit
  pushd thread-frontend || exit
    npm run lint:fix
    npm run format
  popd || exit
  
  
  cargo sqlx prepare --all --workspace -- --all-targets
  cargo clippy --all-targets --all-features --workspace --tests --allow-dirty --fix
  cargo fmt
popd || exit
