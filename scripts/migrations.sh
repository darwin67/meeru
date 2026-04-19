#!/usr/bin/env bash

set -euo pipefail

if [ "$#" -eq 0 ]; then
  cargo run --quiet -p meeru-storage --bin migrate -- help
  exit 0
fi

cargo run --quiet -p meeru-storage --bin migrate -- "$@"
