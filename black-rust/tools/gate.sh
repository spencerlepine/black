#!/bin/sh
# The CI gate: all four must pass before a commit.
set -e
. "$HOME/.cargo/env"
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo clippy -q --workspace --all-targets -- -D warnings
cargo test -q --workspace >/tmp/black-gate-test.log 2>&1 || { grep -E "FAILED|panicked|left|right" /tmp/black-gate-test.log | head -20; exit 1; }
cargo build -q --workspace --release
python3 -I tools/sync-docs.py >/dev/null
echo "gate: fmt ok, clippy ok, test ok, build ok"
