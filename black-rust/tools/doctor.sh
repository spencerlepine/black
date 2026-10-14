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
    mkdir -p /tmp/blk && "$PY" -m venv /tmp/blk/venv && /tmp/blk/venv/bin/pip -q install unicorn pillow numpy
  fi
fi
/tmp/blk/venv/bin/python -c "import unicorn" 2>/dev/null && ok "unicorn venv" || fail venv "see docs/migration/ENV.md"
# imaging libs for tools/ui-diff (compare.py); scikit-image is not needed (pure numpy metrics)
/tmp/blk/venv/bin/python -c "import PIL, numpy" 2>/dev/null || /tmp/blk/venv/bin/pip -q install pillow numpy >/dev/null 2>&1
/tmp/blk/venv/bin/python -c "import PIL, numpy" 2>/dev/null && ok "ui-diff venv (pillow, numpy)" || fail "ui-diff venv" "/tmp/blk/venv/bin/pip install pillow numpy"
# scripted reference captures (tools/ui-diff/capture_ref.py): the DeSmuME core via py-desmume, in its own venv
# because its wheels exist only for CPython <= 3.13 (brew python@3.13 / mise 3.12), not for the 3.15 harness venv
if ! /tmp/blk/venv-dsm/bin/python -c "import desmume.emulator" 2>/dev/null; then
  for cand in python3.13 python3.12 python3.11 "$HOME"/.local/share/mise/installs/python/3.12*/bin/python3 "$HOME"/.local/share/mise/installs/python/3.13*/bin/python3; do
    P=$(command -v "$cand" 2>/dev/null || ls -d "$cand" 2>/dev/null | head -1)
    [ -n "$P" ] || continue
    "$P" -m venv /tmp/blk/venv-dsm 2>/dev/null && /tmp/blk/venv-dsm/bin/pip -q install py-desmume pillow numpy >/dev/null 2>&1 && break
    rm -rf /tmp/blk/venv-dsm
  done
fi
/tmp/blk/venv-dsm/bin/python -c "import desmume.emulator" 2>/dev/null && ok "ui-diff emulator venv (py-desmume)" || fail "ui-diff emulator venv" "python3.13 -m venv /tmp/blk/venv-dsm && /tmp/blk/venv-dsm/bin/pip install py-desmume pillow numpy"
[ -f "$R/black-rom/build/black.us/pokeblack.us.nds" ] && ok "ROM image for capture_ref.py" || fail "ROM .nds" "build the ROM (pokeblack-wine image) or set BLACK_ROM_NDS"
# headless window for tools/ui-diff/capture_port.py on Linux
if [ "$(uname -s)" = Linux ] && [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
  command -v xvfb-run >/dev/null && ok "xvfb-run (headless UI captures)" || fail xvfb-run "install xorg-x11-server-Xvfb (needed only for Phase G/U captures)"
fi

# decompile cache for every known overlay set (docs/migration/ENV.md "RE workflow", Overlay sets)
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
