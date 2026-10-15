#!/bin/sh
# check-field-diff.sh: F12, the Unicorn differential of the field session.
# Runs the scripted session of crates/black-field (`integration::session::f12_field_session`) through the
# ports and writes its per-frame trace, then replays the same keys entirely in the original code under the
# emulator (tools/rom-diff/field_diff.py, the same boot / host models as integration/host.rs), and compares
# the two traces line by line and the printed texts.
# Outputs go to $BLK_OUT/f12 (else $CARGO_TARGET_DIR/f12, else target/f12).
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
PY=${PYTHON:-/tmp/blk/venv/bin/python}
D=${CARGO_TARGET_DIR:-target}
D=${BLK_OUT:-$D}
OUT=$D/f12
mkdir -p "$OUT"
rm -f "$OUT"/session.*.trace "$OUT"/*.text
# the test writes its trace as it plays; a failure of its own checks (RECORDED hashes) still leaves the whole
# trace for the comparison, and fails this check at the end
TEST=ok
if ! F12_TRACE_OUT="$OUT/session.rust.trace" F12_TEXT_OUT="$OUT/rust.text" REQUIRE_ROM=1 \
  cargo test -q -p black-field --lib integration::session::f12_field_session >"$OUT/test.log" 2>&1; then
  TEST=
  grep -a -A3 "panicked" "$OUT/test.log" | head -8
  if [ ! -s "$OUT/rust.text" ]; then
    echo "Rust session failed (log in $OUT/test.log)"
    exit 1
  fi
fi
if ! $PY -I tools/rom-diff/field_diff.py --keys "$OUT/session.rust.trace" --out "$OUT/session.rom.trace" \
  --text "$OUT/rom.text"; then
  echo "ROM run failed (partial trace in $OUT/session.rom.trace)"
  exit 1
fi
if ! cmp -s "$OUT/session.rust.trace" "$OUT/session.rom.trace"; then
  N=$(wc -l <"$OUT/session.rom.trace")
  head -n "$N" "$OUT/session.rust.trace" | diff - "$OUT/session.rom.trace" | head -8 | cut -c1-400
  echo "field trace differs ($OUT/session.rust.trace vs $OUT/session.rom.trace)"
  exit 1
fi
if ! cmp -s "$OUT/rust.text" "$OUT/rom.text"; then
  diff "$OUT/rust.text" "$OUT/rom.text" | head -8
  echo "printed texts differ"
  exit 1
fi
echo "field diff: $(wc -l <"$OUT/session.rom.trace") frames match, texts match"
if [ -z "$TEST" ]; then
  echo "but the Rust session's own checks failed (log in $OUT/test.log)"
  exit 1
fi
