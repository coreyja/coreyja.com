#!/usr/bin/env bash

pushd "$(git rev-parse --show-toplevel)" || exit
  ./scripts/build-frontend.sh
  overmind s
popd || exit
