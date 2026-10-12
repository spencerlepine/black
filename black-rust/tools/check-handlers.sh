#!/bin/sh
# check-handlers.sh MODULE [--per-function]: regenerate the handler list, run the original code
# under the emulator and the Rust port, and record the hash only if both agree.
# --per-function: also compare a hash per handler (JSON maps); on mismatch name the function and
# its first failing input (diverging section/word and stub-log entries). Recording is unchanged:
# only on a full module match.
# --coverage: afterwards print the handlers of MODULE below 70% instruction coverage under the
# harness (tools/rom-diff/coverage.py; informational, never changes the exit status).
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
MOD=$1
PERFN=
COV=
for a in "$@"; do
  [ "$a" = "--per-function" ] && PERFN=1
  [ "$a" = "--coverage" ] && COV=1
done
PY=${PYTHON:-/tmp/blk/venv/bin/python}
python3 -I tools/sync-handlers.py "$MOD"
# Z18b: regenerate the Rust mirror of the Python harness hints (handlers/hints.rs)
$PY -I tools/rom-diff/genhints.py
cargo fmt --all
T=crates/black-battle/src/handlers/$MOD/tests.rs
if [ -n "$PERFN" ]; then
  D=${CARGO_TARGET_DIR:-target}
  D=${BLK_OUT:-$D}  # per-worker output dir (shared target dir is safe)
  mkdir -p "$D"
  mkdir -p "$D"
  RJ=$D/handler-hashes-$MOD.json
  PJ=$D/handler-hashes-$MOD.rom.json
  rm -f "$RJ"
  $PY -I tools/rom-diff/handlers.py "$MOD" --json --jobs "${JOBS:-8}" >"$PJ"
  HANDLER_JSON=$RJ cargo test -q -p black-battle "${MOD}_match_rom" >"$D/handler-test-$MOD.log" 2>&1 || true
  RUST=$(sed -n 's/^{"hash": "\([0-9a-f]*\)".*/\1/p' "$RJ" | head -1)
  # per-function hashes are folded into the module hash in order, so all-equal implies equal
  if [ -n "$RUST" ] && python3 -I tools/rom-diff/diffmaps.py maps "$PJ" "$RJ" >"$D/handler-bad-$MOD.txt"; then ROM=$RUST; else ROM=per-function-mismatch; fi
else
  ROM=$($PY -I tools/rom-diff/handlers.py "$MOD" | awk '{print $2}')
  RUST=$(cargo test -q -p black-battle "${MOD}_match_rom" 2>&1 | sed -n 's/.*left: "\([0-9a-f]*\)".*/\1/p')
  if [ -z "$RUST" ]; then RUST=$(sed -n 's/.*assert_eq!(h, "\([0-9a-f]*\)").*/\1/p' "$T"); fi
fi
echo "rom=$ROM rust=$RUST"
if [ -n "$COV" ]; then
  $PY -I tools/rom-diff/coverage.py "$MOD" --jobs "${JOBS:-8}" || true
fi
if [ "$ROM" = "$RUST" ]; then
  sed -i.bak "s/assert_eq!(h, \"[0-9a-fA-Z_]*\");/assert_eq!(h, \"$ROM\");/" "$T" && rm -f "$T.bak"
  echo "MATCH: recorded"
else
  echo "MISMATCH"
  if [ -n "$PERFN" ]; then
    head -20 "$D/handler-bad-$MOD.txt" | while read -r A S; do
      case "$S" in MISSING*) echo "function 0x$A: $S"; continue;; esac
      echo "function 0x$A: first failing input = seed $S (of 200)"
      $PY -I tools/rom-diff/handlers.py "$MOD" --detail "$A:$S" >"$D/detail-rom.json"
      HANDLER_DETAIL="$A:$S" HANDLER_DETAIL_OUT="$D/detail-rust.json" \
        cargo test -q -p black-battle "${MOD}_match_rom" >/dev/null 2>&1 || true
      python3 -I tools/rom-diff/diffmaps.py stream "$D/detail-rom.json" "$D/detail-rust.json" || true
    done
  fi
  exit 1
fi
