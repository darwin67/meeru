#!/usr/bin/env bash

set -euo pipefail

output_path="${1:-docs/generated/schema.sql}"

cargo run --quiet -p meeru-storage --bin dump_schema -- "$output_path"
