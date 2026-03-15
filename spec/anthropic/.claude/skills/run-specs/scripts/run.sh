#!/usr/bin/env bash
# Run RSpec and exit with the same code RSpec does.
# Usage: run.sh [rspec arguments...]
#
# Passes all arguments directly to `bundle exec rspec`.
# If no arguments are given, defaults to spec/.

set -euo pipefail

TARGET="${*:-spec/}"

exec bundle exec rspec $TARGET \
  --format progress \
  --format json --out tmp/rspec-results.json
