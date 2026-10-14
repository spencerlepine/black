#!/usr/bin/env python3
"""Capture the reference frames of UI-diff cases from the original game, headless and scripted (no human).

    /tmp/blk/venv-dsm/bin/python -I tools/ui-diff/capture_ref.py <case-id> [...]   # one or more cases
    /tmp/blk/venv-dsm/bin/python -I tools/ui-diff/capture_ref.py --all [--force]   # every case (skips ones that
                                                                                   # already have all ref PNGs)
    /tmp/blk/venv-dsm/bin/python -I tools/ui-diff/capture_ref.py --save-state NAME --script "400:A" --at 5000
        # run a script from power-on (or --from STATE) and write ui-ref/states/NAME.dst for cases to start from

Runs the DeSmuME core in-process through py-desmume (the venv is created by tools/doctor.sh): opens the ROM
(BLACK_ROM_NDS, default black-rom/build/black.us/pokeblack.us.nds), optionally loads a savestate, then steps
frame by frame, pressing the buttons of the case's `ref.script` ("frame:Button,frame:Button", each held for
`ref.hold` frames, default 4) and taking a 256x384 screenshot (top above bottom) at every frame listed in
`ref.frames` (ints or [lo, hi] windows). Output: ui-ref/<case>/ref_<frame>.png + the perceptual hash of each
frame recorded under `phash` in ui-ref/manifest.json (commit the manifest; never the PNGs or states, D5).

Frame 0 is power-on (DeSmuME boots the ROM directly: health screen first, `A` at ~400 skips it, the intro movie
then runs to the title screen at ~frame 4900) or the savestate if `ref.savestate` names one in ui-ref/states/.
The emulation is deterministic for everything that does not read the real-time clock; cases around clock-seeded
randomness should use frame windows. Save type is forced to FLASH 4 Mbit (6): with autodetect the game stays on a
white screen. ~160 frames/s on an M-series Mac.
"""
import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.abspath(os.path.join(HERE, "..", ".."))
ROOT = os.path.abspath(os.path.join(RUST, ".."))
UI_REF = os.path.join(RUST, "ui-ref")
STATES = os.path.join(UI_REF, "states")
MANIFEST = os.path.join(UI_REF, "manifest.json")
ROM = os.environ.get("BLACK_ROM_NDS", os.path.join(ROOT, "black-rom", "build", "black.us", "pokeblack.us.nds"))
SAVETYPE_FLASH_4MBIT = 6
DEFAULT_HOLD = 4

sys.path.insert(0, HERE)
import compare  # noqa: E402  (phash; pillow + numpy)

try:
    from desmume.controls import Keys, keymask
    from desmume.emulator import DeSmuME
except ImportError:
    sys.exit("capture_ref: py-desmume is not installed in this interpreter; run tools/doctor.sh and use /tmp/blk/venv-dsm/bin/python")

BUTTONS = {
    "a": Keys.KEY_A, "b": Keys.KEY_B, "x": Keys.KEY_X, "y": Keys.KEY_Y, "l": Keys.KEY_L, "r": Keys.KEY_R,
    "start": Keys.KEY_START, "select": Keys.KEY_SELECT,
    "up": Keys.KEY_UP, "down": Keys.KEY_DOWN, "left": Keys.KEY_LEFT, "right": Keys.KEY_RIGHT,
}


def parse_script(s, hold):
    """"400:A,520:Start" -> {frame: [(key, release_frame)]} (same syntax as the port's BLACK_SCRIPT)."""
    out = {}
    for item in (s or "").split(","):
        item = item.strip()
        if not item:
            continue
        f, b = item.split(":")
        key = BUTTONS[b.strip().lower()]
        out.setdefault(int(f), []).append((key, int(f) + hold))
    return out


def expand(frames):
    out = []
    for f in frames:
        out += list(range(f[0], f[1] + 1)) if isinstance(f, list) else [f]
    return sorted(set(out))


_CORE = None  # the DeSmuME core is a process-wide singleton: a second DeSmuME() aborts the process


class Emu:
    def __init__(self, savestate=None):
        global _CORE
        if not os.path.exists(ROM):
            sys.exit("capture_ref: ROM not found: %s (set BLACK_ROM_NDS)" % ROM)
        if _CORE is None:
            _CORE = DeSmuME()
            _CORE.set_savetype(SAVETYPE_FLASH_4MBIT)
            _CORE.open(ROM)
            _CORE.volume_set(0)
        else:
            _CORE.reset()  # power-on state again for the next case
        self.emu = _CORE
        if savestate:
            path = savestate if os.path.isabs(savestate) else os.path.join(STATES, savestate)
            if not os.path.exists(path):
                raise FileNotFoundError(path)
            self.emu.savestate.load_file(path)
        self.frame = 0

    def run(self, script, until, on_frame):
        """Advance to frame `until`, applying `script` (parse_script output); call on_frame(frame) after each frame."""
        releases = []  # (release_frame, key)
        while self.frame < until:
            self.frame += 1
            for key, rel in script.get(self.frame, []):
                self.emu.input.keypad_add_key(keymask(key))
                releases.append((rel, key))
            for rel, key in [r for r in releases if r[0] == self.frame]:
                self.emu.input.keypad_rm_key(keymask(key))
            releases = [r for r in releases if r[0] != self.frame]
            self.emu.cycle(with_joystick=False)
            on_frame(self.frame)

    def screenshot(self):
        return self.emu.screenshot()

    def save_state(self, path):
        self.emu.savestate.save_file(path)


def capture_case(case, man, force=False):
    cid = case["id"]
    ref = case.get("ref", {})
    frames = expand(ref["frames"])
    out_dir = os.path.join(UI_REF, cid)
    os.makedirs(out_dir, exist_ok=True)
    if not force and all(os.path.exists(os.path.join(out_dir, "ref_%d.png" % f)) for f in frames):
        print("capture_ref: %s already captured (%d frames); --force to redo" % (cid, len(frames)))
        return True
    try:
        emu = Emu(ref.get("savestate"))
    except FileNotFoundError as e:
        print("capture_ref: %s skipped: savestate missing: %s (make it with --save-state)" % (cid, e))
        return False
    script = parse_script(ref.get("script", ""), ref.get("hold", DEFAULT_HOLD))
    want = set(frames)
    phashes = case.setdefault("phash", {})

    def on_frame(f):
        if f in want:
            im = emu.screenshot()
            p = os.path.join(out_dir, "ref_%d.png" % f)
            im.save(p)
            phashes[str(f)] = compare.phash(compare.load_rgb(p))

    emu.run(script, max(frames), on_frame)
    print("capture_ref: %s wrote %d frames to %s" % (cid, len(frames), os.path.relpath(out_dir, RUST)))
    return True


def save_manifest(man):
    with open(MANIFEST, "w") as f:
        json.dump(man, f, indent=1)
        f.write("\n")


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("cases", nargs="*")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--force", action="store_true", help="re-capture even when the ref PNGs exist")
    ap.add_argument("--save-state", metavar="NAME", help="write ui-ref/states/NAME.dst after running --script to --at")
    ap.add_argument("--script", default="", help="with --save-state: frame:Button list from power-on (or --from)")
    ap.add_argument("--from", dest="from_state", help="with --save-state: start from this savestate")
    ap.add_argument("--at", type=int, help="with --save-state: frame at which to save")
    ap.add_argument("--hold", type=int, default=DEFAULT_HOLD)
    a = ap.parse_args(argv)
    if a.save_state:
        if a.at is None:
            ap.error("--save-state needs --at")
        os.makedirs(STATES, exist_ok=True)
        emu = Emu(a.from_state)
        emu.run(parse_script(a.script, a.hold), a.at, lambda f: None)
        path = os.path.join(STATES, a.save_state + ".dst")
        emu.save_state(path)
        emu.screenshot().save(os.path.join(STATES, a.save_state + ".png"))
        print("capture_ref: wrote %s (+ .png preview) at frame %d" % (os.path.relpath(path, RUST), a.at))
        return 0
    man = json.load(open(MANIFEST))
    by_id = {c["id"]: c for c in man["cases"]}
    ids = [c["id"] for c in man["cases"]] if a.all else a.cases
    if not ids:
        ap.error("case ids, --all, or --save-state")
    rc = 0
    for cid in ids:
        if cid not in by_id:
            print("capture_ref: unknown case %s" % cid, file=sys.stderr)
            rc = 1
            continue
        if not capture_case(by_id[cid], man, a.force):
            rc = 1
    save_manifest(man)
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
