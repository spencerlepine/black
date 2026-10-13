#!/bin/sh
# check-fn.sh CRATE AREA [--per-function] [--fn HEXADDR]: run every PORTED function of one area of
# any logic crate on the original code under the emulator and on the Rust port, and record the
# per-function hashes in the area's test module only if all of them agree (Z16).
# Layout (by convention; override with the env vars below):
#   crates/CRATE/src/AREA.rs          has `pub const PORTED: &[u32]` (the functions to check)
#   crates/CRATE/src/AREA/tests.rs    has `fn fndiff_match_rom` (black_fndiff::run_suite) and the
#                                     `// fndiff-recorded-begin` .. `-end` RECORDED table
#   tools/rom-diff/specs/CRATE/AREA.spec   the spec; a `set NAME` line picks the memory image
#                                     (battle | field | any tools/re/sets.txt set)
# --per-function: on a mismatch name each differing function and its first failing input (diverging
#                 section/word and +-3 words of context from both sides).
# --fn ADDR:      check just that one function (ROM vs Rust); never records.
# Recording writes the RECORDED table only from a full match and refuses empty/short hashes.
# Env: FNDIFF_SRC, FNDIFF_TESTS, FNDIFF_SPEC (paths), FNDIFF_FILTER (cargo test filter),
#      FNDIFF_PORTED_EXTRA (more files whose PORTED callees run as real code), JOBS, PYTHON.
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
CRATE=$1
AREA=$2
[ -n "$CRATE" ] && [ -n "$AREA" ] || { sed -n '2,19p' "$0"; exit 2; }
shift 2
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
SRC=${FNDIFF_SRC:-crates/$CRATE/src/$AREA.rs}
T=${FNDIFF_TESTS:-crates/$CRATE/src/$AREA/tests.rs}
SPEC=${FNDIFF_SPEC:-tools/rom-diff/specs/$CRATE/$AREA.spec}
TEST=${FNDIFF_FILTER:-$AREA::tests::fndiff_match_rom}
# The default filter is the full test path: match it exactly (a substring filter for `encount` also runs
# `effect_encount`, and both suites write the same FNDIFF_JSON).
EXACT=; [ -n "$FNDIFF_FILTER" ] || EXACT="-- --exact"
[ -f "$SRC" ] || { echo "no $SRC: nothing is ported in $CRATE/$AREA yet"; exit 2; }
[ -f "$T" ] || { echo "no $T (the test module with the RECORDED table)"; exit 2; }
[ -f "$SPEC" ] || { echo "no $SPEC: request a spec (genspec.py draft + hand review)"; exit 2; }
PSRC="--ported-src $SRC"
for X in $FNDIFF_PORTED_EXTRA; do PSRC="$PSRC --ported-src $X"; done
TAG=$CRATE-$AREA
RJ=$D/fndiff-$TAG.rust.json
PJ=$D/fndiff-$TAG.rom.json
rm -f "$RJ" "$PJ"
if [ -n "$FN" ]; then
  $PY -I tools/rom-diff/svflow.py "$AREA" --spec "$SPEC" $PSRC --json --fn "$FN" >"$PJ"
  FNDIFF_ONLY=$FN FNDIFF_JSON=$RJ cargo test -q -p "$CRATE" "$TEST" $EXACT >"$D/fndiff-test-$TAG.log" 2>&1 || true
else
  $PY -I tools/rom-diff/svflow.py "$AREA" --spec "$SPEC" $PSRC --json --jobs "${JOBS:-8}" >"$PJ"
  FNDIFF_JSON=$RJ cargo test -q -p "$CRATE" "$TEST" $EXACT >"$D/fndiff-test-$TAG.log" 2>&1 || true
fi
if [ ! -s "$RJ" ]; then
  echo "Rust run wrote no JSON (compile error or panic); see $D/fndiff-test-$TAG.log"
  tail -20 "$D/fndiff-test-$TAG.log"
  exit 1
fi
ROM=$(sed -n 's/^{"module": "[a-z0-9_]*", "hash": "\([0-9a-f]*\)".*/\1/p' "$PJ" | head -1)
RUST=$(sed -n 's/^{"module": "[a-z0-9_]*", "hash": "\([0-9a-f]*\)".*/\1/p' "$RJ" | head -1)
if python3 -I tools/rom-diff/diffmaps.py maps "$PJ" "$RJ" >"$D/fndiff-bad-$TAG.txt"; then
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
    head -20 "$D/fndiff-bad-$TAG.txt" | while read -r A S; do
      case "$S" in MISSING*) echo "function 0x$A: $S"; continue ;; esac
      echo "function 0x$A: first failing input = seed $S"
      $PY -I tools/rom-diff/svflow.py "$AREA" --spec "$SPEC" $PSRC --detail "$A:$S" >"$D/fndiff-detail-rom.json"
      FNDIFF_DETAIL="$A:$S" FNDIFF_DETAIL_OUT="$D/fndiff-detail-rust.json" \
        cargo test -q -p "$CRATE" "$TEST" $EXACT >/dev/null 2>&1 || true
      python3 -I tools/rom-diff/diffmaps.py stream "$D/fndiff-detail-rom.json" "$D/fndiff-detail-rust.json" || true
    done
  fi
  exit 1
fi
