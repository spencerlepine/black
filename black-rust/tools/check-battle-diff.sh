#!/bin/sh
# check-battle-diff.sh [--record] [BATTLE ...]: B19, the Unicorn differential of whole battles.
# Runs each battle of tools/rom-diff/battle_diff.py BATTLES (default: all) entirely in the original
# code under the emulator (ARM9 main + overlays 93-96 + 231 with a boot / OS / heap / file-system /
# view shim) and through the ports (`integration::b19::b19_battles_match_rom`, crates/black-battle),
# and compares their traces line by line (command frames, per-frame HP + battle RNG, MT calls).
# --record: if every trace matches, write the ROM hashes into the RECORDED table of
#           crates/black-battle/src/integration/b19.rs (all battles only), then re-run the test.
# Outputs go to $BLK_OUT/b19 (else $CARGO_TARGET_DIR/b19, else target/b19).
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
REC=
BATTLES=
while [ $# -gt 0 ]; do
  case "$1" in
    --record) REC=1 ;;
    -*) echo "unknown option $1"; exit 2 ;;
    *) BATTLES="$BATTLES $1" ;;
  esac
  shift
done
if [ -n "$REC" ] && [ -n "$BATTLES" ]; then
  echo "--record needs every battle (no BATTLE arguments)"
  exit 2
fi
PY=${PYTHON:-/tmp/blk/venv/bin/python}
D=${CARGO_TARGET_DIR:-target}
D=${BLK_OUT:-$D}
OUT=$D/b19
RS=crates/black-battle/src/integration/b19.rs
TEST=integration::b19::b19_battles_match_rom
mkdir -p "$OUT"
rm -f "$OUT"/*.trace
# shellcheck disable=SC2086
if ! $PY -I tools/rom-diff/battle_diff.py --out "$OUT" $BATTLES; then
  echo "ROM run failed (partial traces in $OUT)"
  exit 1
fi
LOG=$D/b19-test.log
if BLK_OUT=$D B19_ROM_TRACES=$OUT REQUIRE_ROM=1 cargo test -q -p black-battle --lib "$TEST" -- --nocapture >"$LOG" 2>&1; then
  grep '^B19' "$LOG" || true
  echo "MATCH (recorded hashes reproduce)"
  exit 0
fi
grep '^B19\|panicked' "$LOG" || tail -20 "$LOG"
if grep -q 'first difference' "$LOG"; then
  echo "MISMATCH: traces in $OUT (<name>.trace = ROM, <name>.rust.trace = ports)"
  exit 1
fi
if [ -z "$REC" ]; then
  echo "traces equal but RECORDED is stale; run with --record"
  exit 1
fi
$PY -I tools/rom-diff/battle_diff.py --record "$RS" --out "$OUT"
cargo fmt --all
if BLK_OUT=$D B19_ROM_TRACES=$OUT REQUIRE_ROM=1 cargo test -q -p black-battle --lib "$TEST" -- --nocapture >"$LOG" 2>&1; then
  grep '^B19' "$LOG" || true
  echo "MATCH: recorded"
else
  tail -20 "$LOG"
  echo "recorded, but the test still fails"
  exit 1
fi
