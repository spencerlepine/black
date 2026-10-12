#!/bin/sh
# check-svflow.sh AREA [--per-function] [--fn HEXADDR]: run every PORTED function of a server-flow
# area (crates/black-battle/src/server_flow/AREA.rs) on the original code under the emulator and
# on the Rust port, and record the per-function hashes in AREA/tests.rs only if all of them agree.
# --per-function: on a mismatch name each differing function and its first failing input (diverging
#                 section/word and +-3 words of context from both sides).
# --fn ADDR:      check just that one function (ROM vs Rust); never records.
# Recording writes the RECORDED table only from a full match and refuses empty/short hashes.
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
AREA=$1
[ -n "$AREA" ] || { sed -n '2,9p' "$0"; exit 2; }
shift
PERFN=
FN=
while [ $# -gt 0 ]; do
  case "$1" in
    --per-function) PERFN=1 ;;
    --fn) FN=$(echo "$2" | sed 's/^0x//'); shift ;;
    *) echo "unknown option $1"; exit 2 ;;
  esac
  shift
done
PY=${PYTHON:-/tmp/blk/venv/bin/python}
D=${CARGO_TARGET_DIR:-target}
D=${BLK_OUT:-$D}  # per-worker output dir (shared target dir is safe)
mkdir -p "$D"
mkdir -p "$D"
MOD=server_flow
case "$AREA" in client[0-9]*) MOD=client ;; esac  # B13-spec1: btl_client.c areas live in crates/black-battle/src/client/
T=crates/black-battle/src/$MOD/$AREA/tests.rs
[ -f "$T" ] || { echo "no $T: nothing is ported in $AREA yet"; exit 2; }
TEST=$MOD::$AREA::tests::svflow_match_rom
RJ=$D/svflow-$AREA.rust.json
PJ=$D/svflow-$AREA.rom.json
rm -f "$RJ" "$PJ"
if [ -n "$FN" ]; then
  $PY -I tools/rom-diff/svflow.py "$AREA" --json --fn "$FN" >"$PJ"
  SVFLOW_ONLY=$FN SVFLOW_JSON=$RJ cargo test -q -p black-battle "$TEST" >"$D/svflow-test-$AREA.log" 2>&1 || true
else
  $PY -I tools/rom-diff/svflow.py "$AREA" --json --jobs "${JOBS:-8}" >"$PJ"
  SVFLOW_JSON=$RJ cargo test -q -p black-battle "$TEST" >"$D/svflow-test-$AREA.log" 2>&1 || true
fi
if [ ! -s "$RJ" ]; then
  echo "Rust run wrote no JSON (compile error or panic); see $D/svflow-test-$AREA.log"
  tail -20 "$D/svflow-test-$AREA.log"
  exit 1
fi
ROM=$(sed -n 's/^{"module": "[a-z0-9_]*", "hash": "\([0-9a-f]*\)".*/\1/p' "$PJ" | head -1)
RUST=$(sed -n 's/^{"module": "[a-z0-9_]*", "hash": "\([0-9a-f]*\)".*/\1/p' "$RJ" | head -1)
if python3 -I tools/rom-diff/diffmaps.py maps "$PJ" "$RJ" >"$D/svflow-bad-$AREA.txt"; then
  echo "rom=$ROM rust=$RUST"
  if [ -n "$FN" ]; then
    echo "MATCH: $FN (not recorded: --fn)"
  else
    python3 -I tools/rom-diff/svrecord.py "$T" "$PJ"
    cargo fmt --all  # svrecord writes the RECORDED table unformatted
    echo "MATCH: recorded"
  fi
else
  echo "rom=$ROM rust=mismatch"
  echo "MISMATCH"
  if [ -n "$PERFN" ] || [ -n "$FN" ]; then
    head -20 "$D/svflow-bad-$AREA.txt" | while read -r A S; do
      case "$S" in MISSING*) echo "function 0x$A: $S"; continue ;; esac
      echo "function 0x$A: first failing input = seed $S"
      $PY -I tools/rom-diff/svflow.py "$AREA" --detail "$A:$S" >"$D/svflow-detail-rom.json"
      SVFLOW_DETAIL="$A:$S" SVFLOW_DETAIL_OUT="$D/svflow-detail-rust.json" \
        cargo test -q -p black-battle "$TEST" >/dev/null 2>&1 || true
      python3 -I tools/rom-diff/diffmaps.py stream "$D/svflow-detail-rom.json" "$D/svflow-detail-rust.json" || true
    done
  fi
  exit 1
fi
