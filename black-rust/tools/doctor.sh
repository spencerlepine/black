#!/bin/sh
# doctor.sh: check (and where cheap, repair) everything a session needs. Run at session start.
# Recreates the ephemeral /tmp pieces (Unicorn venv, decompile cache). Exit 1 = a human must fix.
R=$(cd "$(dirname "$0")/../.." && pwd)   # input/game-a-mechanics
: "${BLACK_ROM_BUILD:=$R/black-rom/build/black.us}"
: "${RE_OUT:=/tmp/blk/re}"
: "${GHIDRA_INSTALL_DIR:=$HOME/tools/ghidra_12.1.4_PUBLIC}"
: "${JAVA_HOME:=$HOME/.local/share/mise/installs/java/temurin-21.0.12+8.0.LTS}"
export BLACK_ROM_BUILD RE_OUT GHIDRA_INSTALL_DIR JAVA_HOME
bad=0
ok() { echo "ok    $1"; }
fail() { echo "FAIL  $1 -- $2"; bad=1; }

[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
command -v cargo >/dev/null && ok "cargo $(cargo --version | cut -d' ' -f2)" || fail cargo "install rustup"
rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown && ok "wasm32 target" || fail wasm32 "rustup target add wasm32-unknown-unknown"
[ -f "$BLACK_ROM_BUILD/main.elf" ] && [ -f "$BLACK_ROM_BUILD/OVY_93.sbin" ] && ok "ROM build" || fail "ROM build" "missing $BLACK_ROM_BUILD (rebuild with the pokeblack-wine image)"
[ -x "$JAVA_HOME/bin/java" ] && ok "JDK 21" || fail JDK "mise install java@temurin-21.0.12+8.0.LTS"
[ -x "$GHIDRA_INSTALL_DIR/support/analyzeHeadless" ] && ok "Ghidra" || fail Ghidra "unzip ghidra_12.1.4 to ~/tools"
command -v rizin >/dev/null && ok "rizin" || fail rizin "install rizin"

if ! /tmp/blk/venv/bin/python -c "import unicorn" 2>/dev/null; then
  PY=$(ls -d "$HOME"/.local/share/mise/installs/python/3.12*/bin/python3 2>/dev/null | head -1)
  if [ -n "$PY" ]; then
    mkdir -p /tmp/blk && "$PY" -m venv /tmp/blk/venv && /tmp/blk/venv/bin/pip -q install unicorn
  fi
fi
/tmp/blk/venv/bin/python -c "import unicorn" 2>/dev/null && ok "unicorn venv" || fail venv "see MIGRATION-AGENT.md §3"

# decompile cache for every known overlay set (MIGRATION-AGENT.md §3b "Overlay sets")
while read -r name mods; do
  [ -z "$name" ] && continue
  if [ ! -f "$RE_OUT/$name/pc/index.jsonl" ]; then
    echo "..    decompiling $name ($mods), ~3 min"
    # shellcheck disable=SC2086
    sh "$R/black-rust/tools/re/decompile.sh" "$name" $mods >/dev/null || fail "decompile $name" "see $RE_OUT/$name/ghidra.log"
  fi
  [ -f "$RE_OUT/$name/pc/index.jsonl" ] && ok "pseudo-C $name ($(wc -l < "$RE_OUT/$name/pc/index.jsonl") fns)"
done < "$R/black-rust/tools/re/sets.txt"

exit $bad
