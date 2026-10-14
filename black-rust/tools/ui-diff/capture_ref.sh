#!/bin/sh
# capture_ref.sh <case-id>: the human captures the reference frames of one UI-diff case from melonDS.
#
# One-time per case, on the human's machine (the agents never run the emulator). Prints the case's recipe
# (savestate + inputs + frames) from ui-ref/manifest.json, waits for you to load the savestate and get ready,
# then grabs the melonDS window once per listed frame while you perform the recipe, crops the two 256x192
# screens out of the window (scale detected from the window size, like black-game's integer_scale), writes
# ui-ref/<case>/ref_<frame>.png (256x384, top above bottom) and records each frame's perceptual hash in the
# manifest so drift of the golden set itself is detectable (compare.py --hash).
#
# Grabbing: macOS `screencapture -l <windowid>`; Linux `xdotool` + `import` (ImageMagick) or `grim` on
# Wayland. Timing is manual: the script counts frames by wall clock (1/60 s) from the moment you press Enter,
# so use frame windows ([lo, hi]) in the manifest for anything timing-sensitive. If your melonDS build has a
# frame-step hotkey or scripting, prefer that for frame-exact grabs: step to each frame and press Enter at
# the prompt instead of relying on the timer (MELONDS_FRAMESTEP=1).
#
# Requires /tmp/blk/venv (pillow + numpy; doctor.sh creates it). Never commit ui-ref/*.png (D5).
set -e
case_id="$1"
[ -n "$case_id" ] || { echo "usage: $0 <case-id>"; exit 2; }
R=$(cd "$(dirname "$0")/../.." && pwd)
REF="$R/ui-ref"
MAN="$REF/manifest.json"
PY=/tmp/blk/venv/bin/python
[ -x "$PY" ] || { echo "no $PY (run tools/doctor.sh)"; exit 1; }
[ -f "$MAN" ] || { echo "no $MAN"; exit 1; }

recipe=$("$PY" -I - "$MAN" "$case_id" <<'EOF'
import json, sys
man = json.load(open(sys.argv[1]))
c = next((c for c in man["cases"] if c["id"] == sys.argv[2]), None)
if not c:
    sys.exit("unknown case " + sys.argv[2])
frames = []
for f in c["ref"]["frames"]:
    frames += list(range(f[0], f[1] + 1)) if isinstance(f, list) else [f]
print("savestate:", c["ref"].get("savestate", "(none: power on)"))
print("inputs:   ", c["ref"].get("inputs", ""))
print("frames:   ", " ".join(str(f) for f in sorted(set(frames))))
EOF
)
echo "$recipe"
frames=$(echo "$recipe" | sed -n 's/^frames: *//p')
mkdir -p "$REF/$case_id"

# find the emulator window
os=$(uname -s)
if [ "$os" = "Darwin" ]; then
  wid=$("$PY" -I - <<'EOF'
import subprocess, json
out = subprocess.run(["osascript", "-e", 'tell application "System Events" to get id of first window of (first process whose name contains "melonDS")'], capture_output=True, text=True)
print(out.stdout.strip())
EOF
)
  [ -n "$wid" ] || { echo "melonDS window not found (is it running?)"; exit 1; }
  grab() { screencapture -x -l "$wid" "$1"; }
else
  wid=$(xdotool search --name melonDS | head -1 2>/dev/null || true)
  if [ -n "$wid" ] && command -v import >/dev/null; then
    grab() { import -window "$wid" "$1"; }
  elif command -v grim >/dev/null; then
    grab() { grim "$1"; }   # whole output; the crop below finds the screens by scale
  else
    echo "need xdotool + import (ImageMagick) or grim"; exit 1
  fi
fi

echo "Load the savestate, put melonDS where you can see it, then press Enter and perform the inputs."
echo "(MELONDS_FRAMESTEP=1: the script prompts for each frame instead of timing them.)"
read -r _
start=$("$PY" -c 'import time; print(time.time())')
for f in $frames; do
  if [ -n "$MELONDS_FRAMESTEP" ]; then
    printf 'step to frame %s, then Enter: ' "$f"; read -r _
  else
    "$PY" -c "import time,sys; t=$start + $f/60.0; d=t-time.time(); time.sleep(max(0,d))"
  fi
  grab "/tmp/blk/ui/ref_${case_id}_$f.png"
done

# crop + hash + record
"$PY" -I - "$MAN" "$case_id" "$REF/$case_id" $frames <<'EOF'
import json, os, sys
from PIL import Image
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(sys.argv[1])), "..", "tools", "ui-diff"))
import compare
man_path, cid, out_dir = sys.argv[1:4]
frames = [int(x) for x in sys.argv[4:]]
man = json.load(open(man_path))
case = next(c for c in man["cases"] if c["id"] == cid)
case.setdefault("phash", {})
for f in frames:
    src = "/tmp/blk/ui/ref_%s_%d.png" % (cid, f)
    im = Image.open(src).convert("RGB")
    w, h = im.size
    scale = max(1, min(w // 256, h // 384))
    cw, ch = 256 * scale, 384 * scale
    ox, oy = (w - cw) // 2, (h - ch) // 2
    crop = im.crop((ox, oy, ox + cw, oy + ch)).resize((256, 384), Image.NEAREST)
    dst = os.path.join(out_dir, "ref_%d.png" % f)
    crop.save(dst)
    case["phash"][str(f)] = compare.phash(compare.load_rgb(dst))
    print("wrote", dst, "scale", scale, "phash", case["phash"][str(f)])
json.dump(man, open(man_path, "w"), indent=1)
open(man_path, "a").write("\n")
EOF
echo "done: ui-ref/$case_id/ref_*.png written; phashes recorded in manifest.json (commit the manifest, never the PNGs)."
