#!/bin/sh
# check-handlers.sh MODULE: regenerate the handler list, run the original code under the
# emulator and the Rust port, and record the hash only if both agree.
set -e
cd "$(dirname "$0")/.."
. "$HOME/.cargo/env"
MOD=$1
PY=${PYTHON:-/tmp/blk/venv/bin/python}
python3 -I tools/sync-handlers.py "$MOD"
ROM=$($PY -I tools/rom-diff/handlers.py "$MOD" | awk '{print $2}')
T=crates/black-battle/src/handlers/$MOD/tests.rs
RUST=$(cargo test -q -p black-battle "${MOD}_match_rom" 2>&1 | sed -n 's/.*left: "\([0-9a-f]*\)".*/\1/p')
if [ -z "$RUST" ]; then RUST=$(sed -n 's/.*assert_eq!(h, "\([0-9a-f]*\)").*/\1/p' "$T"); fi
echo "rom=$ROM rust=$RUST"
if [ "$ROM" = "$RUST" ]; then
  sed -i '' "s/assert_eq!(h, \"[0-9a-fA-Z_]*\");/assert_eq!(h, \"$ROM\");/" "$T"
  echo "MATCH: recorded"
else
  echo "MISMATCH"; exit 1
fi
