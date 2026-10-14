#!/usr/bin/env python3
"""Capture the port's frames for one UI-diff case (headless; stdlib only).

    python3 -I tools/ui-diff/capture_port.py <case-id> [<case-id> ...]
    python3 -I tools/ui-diff/capture_port.py --all

Reads ui-ref/manifest.json, runs `cargo run -p black-game` with
  BLACK_SCRIPT=<port.script>  BLACK_FADE_FRAMES=<port.fade_frames>  BLACK_SCREENSHOT_FRAMES=<port.frames>
  BLACK_SCREENSHOT=/tmp/blk/ui/<case>/shot.png
and crops each `shot_<frame>.png` (the composite window: two stacked 256x192 screens, integer-scaled,
letterboxed) back to a 256x384 top|bottom image at scale 1 in ui-ref/<case>/port_<frame>.png.

Env: CARGO_TARGET_DIR (recommended /tmp/blk/target-coord), BLACK_ASSETS (extracted assets, for the real font),
BLACK_GAME_BIN (skip cargo and run this binary). On Linux without DISPLAY the game runs under `xvfb-run -a`
(doctor.sh checks it); on macOS a logged-in session is enough. Frame windows `[lo, hi]` capture every frame in
the window. The PNG decoder here is minimal (8-bit RGB/RGBA, non-interlaced) to stay stdlib-only.
"""
import json
import os
import platform
import shutil
import struct
import subprocess
import sys
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.abspath(os.path.join(HERE, "..", ".."))
UI_REF = os.path.join(RUST, "ui-ref")
MANIFEST = os.path.join(UI_REF, "manifest.json")
TMP = "/tmp/blk/ui"
W, H = 256, 384


# ---- minimal PNG I/O (8-bit RGB / RGBA, no interlace) -------------------------------------------------------
def read_png(path):
    data = open(path, "rb").read()
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG: " + path
    pos = 8
    idat = []
    w = h = None
    ctype = None
    while pos < len(data):
        ln, = struct.unpack(">I", data[pos: pos + 4])
        typ = data[pos + 4: pos + 8]
        body = data[pos + 8: pos + 8 + ln]
        pos += 12 + ln
        if typ == b"IHDR":
            w, h, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", body)
            assert depth == 8 and interlace == 0 and ctype in (2, 6), "unsupported PNG (%d-bit, colour type %d)" % (depth, ctype)
        elif typ == b"IDAT":
            idat.append(body)
        elif typ == b"IEND":
            break
    raw = zlib.decompress(b"".join(idat))
    bpp = 3 if ctype == 2 else 4
    stride = w * bpp
    rows = []
    prev = bytearray(stride)
    p = 0
    for _ in range(h):
        f = raw[p]
        line = bytearray(raw[p + 1: p + 1 + stride])
        p += 1 + stride
        if f == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 255
        elif f == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 255
        elif f == 3:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 255
        elif f == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if pa <= pb and pa <= pc else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        rows.append(bytes(line))
        prev = line
    return w, h, bpp, rows


def write_png_rgb(path, w, h, rows):
    """rows: list of h bytes objects of length 3*w."""
    raw = b"".join(b"\x00" + r for r in rows)

    def chunk(t, b):
        return struct.pack(">I", len(b)) + t + b + struct.pack(">I", zlib.crc32(t + b) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def crop_composite(src, dst):
    """Find the integer scale and letterbox offset of the 256x384 stack inside the window capture, sample it
    back to scale 1 (centre pixel of each scale x scale block) and write an RGB PNG."""
    w, h, bpp, rows = read_png(src)
    scale = max(1, min(w // 256, h // 384))
    cw, ch = 256 * scale, 384 * scale
    ox, oy = (w - cw) // 2, (h - ch) // 2
    out = []
    for y in range(384):
        sy = oy + y * scale + scale // 2
        line = rows[sy]
        px = bytearray()
        for x in range(256):
            sx = ox + x * scale + scale // 2
            px += line[sx * bpp: sx * bpp + 3]
        out.append(bytes(px))
    write_png_rgb(dst, 256, 384, out)
    return scale


# ---- driver ----------------------------------------------------------------------------------------------------
def frames_of(case):
    out = []
    for f in case["port"]["frames"]:
        if isinstance(f, list):
            out += list(range(f[0], f[1] + 1))
        else:
            out.append(f)
    return sorted(set(out))


def game_command():
    b = os.environ.get("BLACK_GAME_BIN")
    if b:
        return [b]
    cargo = shutil.which("cargo") or os.path.expanduser("~/.cargo/bin/cargo")
    return [cargo, "run", "-q", "-p", "black-game", "--"]


def run_case(case):
    cid = case["id"]
    frames = frames_of(case)
    tmp = os.path.join(TMP, cid)
    os.makedirs(tmp, exist_ok=True)
    for fn in os.listdir(tmp):
        os.remove(os.path.join(tmp, fn))
    env = dict(os.environ)
    env["BLACK_SCRIPT"] = case["port"].get("script", "")
    env["BLACK_FADE_FRAMES"] = str(case["port"].get("fade_frames", 16))
    env["BLACK_SCREENSHOT_FRAMES"] = ",".join(str(f) for f in frames)
    env["BLACK_SCREENSHOT"] = os.path.join(tmp, "shot.png")
    cmd = game_command()
    if platform.system() == "Linux" and not env.get("DISPLAY") and not env.get("WAYLAND_DISPLAY"):
        if not shutil.which("xvfb-run"):
            print("capture_port: no DISPLAY and no xvfb-run (install xorg-x11-server-Xvfb)", file=sys.stderr)
            return 1
        cmd = ["xvfb-run", "-a", "-s", "-screen 0 1024x768x24"] + cmd
    print("capture_port: %s frames %s" % (cid, env["BLACK_SCREENSHOT_FRAMES"]))
    r = subprocess.run(cmd, cwd=RUST, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True, timeout=1800)
    if r.returncode != 0:
        print(r.stderr[-2000:], file=sys.stderr)
        print("capture_port: game exited %d" % r.returncode, file=sys.stderr)
        return 1
    out_dir = os.path.join(UI_REF, cid)
    os.makedirs(out_dir, exist_ok=True)
    missing = []
    for f in frames:
        src = os.path.join(tmp, "shot_%d.png" % f)
        if not os.path.exists(src):
            missing.append(f)
            continue
        scale = crop_composite(src, os.path.join(out_dir, "port_%d.png" % f))
    if missing:
        print("capture_port: frames not captured: %s" % missing, file=sys.stderr)
        return 1
    print("capture_port: wrote %d frames to %s (window scale %d)" % (len(frames), os.path.relpath(out_dir, RUST), scale))
    return 0


def main(argv):
    if not os.path.exists(MANIFEST):
        print("capture_port: no ui-ref/manifest.json", file=sys.stderr)
        return 1
    man = json.load(open(MANIFEST))
    if argv == ["--all"]:
        ids = [c["id"] for c in man["cases"]]
    else:
        ids = argv
    if not ids:
        print(__doc__)
        return 2
    rc = 0
    by_id = {c["id"]: c for c in man["cases"]}
    for cid in ids:
        if cid not in by_id:
            print("capture_port: unknown case %s" % cid, file=sys.stderr)
            rc = 1
            continue
        rc |= run_case(by_id[cid])
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
