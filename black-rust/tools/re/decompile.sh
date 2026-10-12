#!/bin/sh
# decompile.sh NAME MODULE... : synthesize an ELF for one co-resident overlay set, run Ghidra
# headless analysis, apply signatures.tsv, dump pseudo-C for every function.
#   e.g. tools/re/decompile.sh battle main 93 94 95 96
# out: $RE_OUT/NAME/{NAME.elf, proj/, pc/<addr>_<name>.c, pc/index.jsonl, clusters.txt}
# env: REUSE=1   skip import + analysis (after editing signatures.tsv; ~20 s instead of ~3 min)
#      RANGE="lo hi"   only dump functions in [lo, hi) (hex, no 0x)
#      SIGS="a.tsv b.tsv"   signature files, later lines win (default: signatures.tsv)
#      TYPES=types.h        C header parsed into Ghidra data types first (default: types.h; TYPES= for none)
set -e
HERE=$(cd "$(dirname "$0")" && pwd -P)
NAME=$1; shift
: "${BLACK_ROM_BUILD:=$HERE/../../../black-rom/build/black.us}"
: "${RE_OUT:=/tmp/blk/re}"
: "${GHIDRA_INSTALL_DIR:=$HOME/tools/ghidra_12.1.4_PUBLIC}"
: "${JAVA_HOME:=$HOME/.local/share/mise/installs/java/temurin-21.0.12+8.0.LTS}"
export JAVA_HOME PATH="$JAVA_HOME/bin:$PATH"
D=$RE_OUT/$NAME
mkdir -p "$D/proj"
rm -rf "$D/pc.new"
if [ -z "$REUSE" ]; then
  python3 -I "$HERE/mkelf.py" "$BLACK_ROM_BUILD" "$D/$NAME.elf" "$@"
  rm -rf "$D/proj/$NAME.gpr" "$D/proj/$NAME.rep"
  SRC="-import $D/$NAME.elf"
else
  SRC="-process $NAME.elf -noanalysis"
fi
# signatures.tsv addresses in overlay RAM (>= 0x021542C0) and @pool lines describe the battle
# overlays; other sets overlap that range with different code, so they get main-only lines.
if [ -z "$SIGS" ] && [ "$NAME" != battle ]; then
  awk -F'\t' '/^#/ || $1 !~ /^[0-9a-fA-F]+$/ && $1 != "@pool" || ($1 ~ /^[0-9a-fA-F]+$/ && length($1) == 8 && tolower($1) < "021542c0")' \
    "$HERE/signatures.tsv" > "$D/signatures.main.tsv"
  SIGS=$D/signatures.main.tsv
fi
# shellcheck disable=SC2086
"$GHIDRA_INSTALL_DIR/support/analyzeHeadless" "$D/proj" "$NAME" $SRC -scriptPath "$HERE" \
  -postScript ApplySignatures.java ${TYPES-"$HERE/types.h"} ${SIGS:-"$HERE/signatures.tsv"} \
  -postScript DecompileAll.java "$D/pc.new" $RANGE > "$D/ghidra.log" 2>&1 || { tail -20 "$D/ghidra.log"; exit 1; }
# swap in one step so readers (queue.py, packet.py, workers) never see a half-written pc/
rm -rf "$D/pc.old"; [ -d "$D/pc" ] && mv "$D/pc" "$D/pc.old"; mv "$D/pc.new" "$D/pc"; rm -rf "$D/pc.old"
grep -h 'ApplySignatures:' "$D/ghidra.log" | sed 's/.*> //; s/ (GhidraScript).*//'
python3 -I "$HERE/clusters.py" "$D/pc" > "$D/clusters.txt"
head -1 "$D/clusters.txt"
echo "pseudo-C: $D/pc  ($(wc -l < "$D/pc/index.jsonl") functions)"
