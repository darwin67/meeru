#!/usr/bin/env bash

set -euo pipefail

output_path="${1:-docs/generated/schema.sql}"

bash "$(dirname "$0")/migrations.sh" dump "$output_path"
