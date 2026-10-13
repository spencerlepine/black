#!/bin/sh
# The CI gate: all four must pass before a commit.
set -e
. "$HOME/.cargo/env"
cd "$(dirname "$0")/.."
# GATE_NO_GAME=1: workers that don't touch black-game skip Bevy (saves ~10 GB of target dir);
# the coordinator always runs the full gate on merged main.
X=""; [ -n "$GATE_NO_GAME" ] && X="--exclude black-game"
cargo fmt --all --check
cargo clippy -q --workspace $X --all-targets -- -D warnings
cargo clippy -q -p black-data --features rom --all-targets -- -D warnings
# Real-ROM tests fail instead of skipping when the ROM files or build are missing (REQUIRE_ROM=0
# to opt out). The build is untracked, so without BLACK_ROM_BUILD a worktree uses the main
# checkout's (git's common dir is the main checkout's .git).
: "${REQUIRE_ROM:=1}"
if [ -z "$BLACK_ROM_BUILD" ]; then
  BLACK_ROM_BUILD="$(cd "$(git rev-parse --path-format=absolute --git-common-dir)/.." && pwd)/black-rom/build/black.us"
fi
export REQUIRE_ROM BLACK_ROM_BUILD
log="${BLK_OUT:-${CARGO_TARGET_DIR:-target}}/gate-test.log"
mkdir -p "$(dirname "$log")"
cargo test -q --workspace $X >"$log" 2>&1 || { grep -E "FAILED|panicked|left|right" "$log" | head -20; exit 1; }
cargo test -q -p black-data --features rom >>"$log" 2>&1 || { grep -E "FAILED|panicked|left|right" "$log" | head -20; exit 1; }
cargo build -q --workspace $X --release
[ -n "$GATE_NO_GAME" ] || cargo build -q -p black-game --target wasm32-unknown-unknown
python3 -I tools/sync-docs.py >/dev/null
# V-0021: every address in a `pub const PORTED` must have a SYMBOLS.md row (a `/// C:` head sync-docs can read)
python3 -I tools/check-symbols.py >/dev/null
if [ -n "$(git ls-files assets)" ]; then echo "gate: assets/ is tracked (D5)"; exit 1; fi
if [ -n "$GATE_NO_GAME" ]; then echo "gate (no black-game): fmt ok, clippy ok, test ok, build ok"; else echo "gate: fmt ok, clippy ok, test ok, build ok, wasm ok"; fi
