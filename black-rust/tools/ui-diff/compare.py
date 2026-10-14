#!/usr/bin/env python3
"""Score the port's screens against the human-captured emulator goldens. Local compute only, no LLM.

    /tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --case <ID>      # one case
    /tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --all            # every case with goldens
    /tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --all --regress  # gate: fail if a previously passing case
                                                                         # (per docs/migration/PROGRESS.json) now
                                                                         # fails or dropped by > 0.05
    /tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --hash <png>     # perceptual hash of one image (capture_ref.sh)

Inputs (all under black-rust/ui-ref/, gitignored except manifest.json):
  manifest.json            cases: id, screen, port {script, frames, fade_frames}, ref {savestate, inputs, frames},
                           regions (optional per-screen rect + weights), timing (optional), phash (recorded by
                           capture_ref.sh)
  <case>/ref_<frame>.png   golden, 256x384 (top screen above bottom), captured by the human (capture_ref.sh)
  <case>/port_<frame>.png  port frame, 256x384, written by capture_port.py
Outputs:
  ui-ref/report/<case>.json                scores per screen per frame, timing, pass/fail
  ui-ref/report/<case>_<frame>.png         only on failure: reference | port | heat-map, 768 px wide
ref.frames[i] is compared with port.frames[i] (the reference counts frames from the savestate load, the port from
power-on, so the numbers differ). An entry may be an int or a [lo, hi] window: every frame in the window is captured
on its side and the best-matching (ref, port) pair counts (tolerates manual timing jitter).

Metric per screen per frame, after quantising both images to RGB555 (DS colour depth):
  structure = SSIM on grayscale, 7x7 uniform window
  colour    = 1 - mean dE76 (CIE Lab, D65) over 8x8 blocks, scaled so dE 20 -> 0
  layout    = IoU of Canny edge maps after a 2 px dilation (tolerates anti-aliasing and 1 px offsets)
  score     = 0.5*structure + 0.3*colour + 0.2*layout   (weights overridable per region in the manifest)
Pass: every screen of every frame >= 0.70 (PASS_SCORE) and, for cases with `timing`, the latency (frames after the
trigger) at which the perceptual hash first changes matches the reference within +-3 frames.

Pure numpy + pillow (scikit-image has no wheel for the venv's Python); results are deterministic.
"""
import argparse
import json
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.abspath(os.path.join(HERE, "..", ".."))
ROOT = os.path.abspath(os.path.join(RUST, ".."))
UI_REF = os.path.join(RUST, "ui-ref")
MANIFEST = os.path.join(UI_REF, "manifest.json")
REPORT_DIR = os.path.join(UI_REF, "report")
PROGRESS = os.path.join(ROOT, "docs", "migration", "PROGRESS.json")

W, H = 256, 192
PASS_SCORE = 0.70
REGRESS_DROP = 0.05
TIMING_TOL = 3
DEFAULT_WEIGHTS = (0.5, 0.3, 0.2)  # structure, colour, layout


# ---- image helpers -------------------------------------------------------------------------------------------
def load_rgb(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.uint8)


def quantise555(img):
    """RGB888 -> the nearest RGB555 value, re-expanded to 0..255 (both sides get the same treatment)."""
    q = (img.astype(np.uint16) >> 3)
    return ((q * 255 + 15) // 31).astype(np.uint8)


def gray(img):
    f = img.astype(np.float64)
    return 0.299 * f[..., 0] + 0.587 * f[..., 1] + 0.114 * f[..., 2]


def box_filter(a, k):
    """Mean over a k x k window (edge-replicated), via cumulative sums."""
    p = k // 2
    ap = np.pad(a, p, mode="edge")
    c = np.cumsum(np.cumsum(ap, axis=0), axis=1)
    c = np.pad(c, ((1, 0), (1, 0)))
    s = c[k:, k:] - c[:-k, k:] - c[k:, :-k] + c[:-k, :-k]
    return s / float(k * k)


def gaussian_blur(a, sigma=1.0):
    r = int(3 * sigma)
    x = np.arange(-r, r + 1, dtype=np.float64)
    g = np.exp(-(x * x) / (2 * sigma * sigma))
    g /= g.sum()
    ap = np.pad(a, r, mode="edge")
    tmp = np.zeros_like(ap)
    for i, w in enumerate(g):
        tmp += w * np.roll(ap, i - r, axis=1)
    out = np.zeros_like(ap)
    for i, w in enumerate(g):
        out += w * np.roll(tmp, i - r, axis=0)
    return out[r:-r, r:-r]


def ssim(a, b, k=7):
    """Mean SSIM of two grayscale float images (0..255), uniform k x k window."""
    c1, c2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2
    mu_a, mu_b = box_filter(a, k), box_filter(b, k)
    saa = box_filter(a * a, k) - mu_a * mu_a
    sbb = box_filter(b * b, k) - mu_b * mu_b
    sab = box_filter(a * b, k) - mu_a * mu_b
    num = (2 * mu_a * mu_b + c1) * (2 * sab + c2)
    den = (mu_a * mu_a + mu_b * mu_b + c1) * (saa + sbb + c2)
    m = num / den
    return float(np.clip(m.mean(), 0.0, 1.0)), m


def rgb_to_lab(img):
    f = img.astype(np.float64) / 255.0
    lin = np.where(f <= 0.04045, f / 12.92, ((f + 0.055) / 1.055) ** 2.4)
    m = np.array([[0.4124564, 0.3575761, 0.1804375], [0.2126729, 0.7151522, 0.0721750], [0.0193339, 0.1191920, 0.9503041]])
    xyz = lin @ m.T
    xyz /= np.array([0.95047, 1.0, 1.08883])
    e, kk = 216.0 / 24389.0, 24389.0 / 27.0
    fx = np.where(xyz > e, np.cbrt(xyz), (kk * xyz + 16.0) / 116.0)
    L = 116.0 * fx[..., 1] - 16.0
    a = 500.0 * (fx[..., 0] - fx[..., 1])
    b = 200.0 * (fx[..., 1] - fx[..., 2])
    return np.stack([L, a, b], axis=-1)


def colour_score(a, b, block=8):
    la, lb = rgb_to_lab(a), rgb_to_lab(b)
    h, w = la.shape[:2]
    hb, wb = h // block, w // block
    ma = la[: hb * block, : wb * block].reshape(hb, block, wb, block, 3).mean(axis=(1, 3))
    mb = lb[: hb * block, : wb * block].reshape(hb, block, wb, block, 3).mean(axis=(1, 3))
    de = np.sqrt(((ma - mb) ** 2).sum(axis=-1)).mean()
    return float(max(0.0, 1.0 - de / 20.0)), float(de)


def canny(g, low=12.0, high=30.0):
    """Edge map of a grayscale float image: Gaussian blur, Sobel, non-maximum suppression, hysteresis."""
    s = gaussian_blur(g, 1.0)
    sp = np.pad(s, 1, mode="edge")
    gx = (sp[1:-1, 2:] - sp[1:-1, :-2]) * 2 + (sp[:-2, 2:] - sp[:-2, :-2]) + (sp[2:, 2:] - sp[2:, :-2])
    gy = (sp[2:, 1:-1] - sp[:-2, 1:-1]) * 2 + (sp[2:, 2:] - sp[:-2, 2:]) + (sp[2:, :-2] - sp[:-2, :-2])
    mag = np.hypot(gx, gy) / 8.0
    ang = np.rad2deg(np.arctan2(gy, gx)) % 180.0
    mp = np.pad(mag, 1)
    n1 = np.zeros_like(mag)
    n2 = np.zeros_like(mag)
    d0 = (ang < 22.5) | (ang >= 157.5)
    d45 = (ang >= 22.5) & (ang < 67.5)
    d90 = (ang >= 67.5) & (ang < 112.5)
    d135 = (ang >= 112.5) & (ang < 157.5)
    n1[d0], n2[d0] = mp[1:-1, 2:][d0], mp[1:-1, :-2][d0]
    n1[d45], n2[d45] = mp[:-2, 2:][d45], mp[2:, :-2][d45]
    n1[d90], n2[d90] = mp[:-2, 1:-1][d90], mp[2:, 1:-1][d90]
    n1[d135], n2[d135] = mp[:-2, :-2][d135], mp[2:, 2:][d135]
    nms = np.where((mag >= n1) & (mag >= n2), mag, 0.0)
    strong = nms >= high
    weak = (nms >= low) & ~strong
    edges = strong.copy()
    # hysteresis: grow strong edges into connected weak pixels
    while True:
        grown = dilate(edges, 1) & weak & ~edges
        if not grown.any():
            break
        edges |= grown
    return edges


def dilate(mask, r):
    out = mask.copy()
    mp = np.pad(mask, r)
    h, w = mask.shape
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            out |= mp[r + dy: r + dy + h, r + dx: r + dx + w]
    return out


def layout_score(ga, gb):
    ea, eb = dilate(canny(ga), 2), dilate(canny(gb), 2)
    union = (ea | eb).sum()
    if union == 0:
        return 1.0
    return float((ea & eb).sum() / union)


def phash(img):
    """64-bit perceptual hash: 32x32 gray, 8x8 low-frequency DCT, bits = coefficient > median."""
    g = np.asarray(Image.fromarray(np.asarray(img, dtype=np.uint8)).convert("L").resize((32, 32), Image.BILINEAR), dtype=np.float64)
    n = 32
    k = np.arange(n)
    basis = np.cos(np.pi * (2 * k[None, :] + 1) * k[:, None] / (2 * n))
    d = basis @ g @ basis.T
    low = d[:8, :8].flatten()
    med = np.median(low[1:])
    bits = 0
    for i, v in enumerate(low):
        if v > med:
            bits |= 1 << i
    return "%016x" % bits


def hamming(h1, h2):
    return bin(int(h1, 16) ^ int(h2, 16)).count("1")


# ---- scoring -------------------------------------------------------------------------------------------------
def score_pair(ref, port, weights=DEFAULT_WEIGHTS):
    """Score one 256x192 screen (RGB uint8 arrays). Returns (score, parts dict, ssim map)."""
    r, p = quantise555(ref), quantise555(port)
    gr, gp = gray(r), gray(p)
    structure, smap = ssim(gr, gp)
    colour, de = colour_score(r, p)
    layout = layout_score(gr, gp)
    ws, wc, wl = weights
    score = ws * structure + wc * colour + wl * layout
    return float(score), {"structure": round(structure, 4), "colour": round(colour, 4), "layout": round(layout, 4), "dE76": round(de, 2), "score": round(score, 4)}, smap


def split_screens(stack):
    assert stack.shape[0] == 2 * H and stack.shape[1] == W, "expected a 256x384 top|bottom stack, got %s" % (stack.shape,)
    return {"top": stack[:H], "bottom": stack[H:]}


def region_of(case, screen, img):
    reg = (case.get("regions") or {}).get(screen, "whole")
    weights = DEFAULT_WEIGHTS
    rect = reg
    if isinstance(reg, dict):
        rect = reg.get("rect", "whole")
        weights = tuple(reg.get("weights", DEFAULT_WEIGHTS))
    if rect == "whole":
        return img, weights
    x, y, w, h = rect
    return img[y: y + h, x: x + w], weights


def expand(f):
    return list(range(f[0], f[1] + 1)) if isinstance(f, list) else [f]


def frame_entries(case):
    """[(label, ref_frames, port_frames)]: the i-th entry of ref.frames is compared with the i-th entry of
    port.frames (the two timelines differ: the reference counts from the savestate load, the port from
    power-on). An entry is an int or a [lo, hi] window; windows are compared as a cross product and the
    best pair counts."""
    pf = case["port"]["frames"]
    rf = case.get("ref", {}).get("frames", pf)
    if len(rf) != len(pf):
        raise SystemExit("case %s: ref.frames and port.frames must have the same length" % case["id"])
    out = []
    for r, p in zip(rf, pf):
        label = "%d-%d" % (p[0], p[1]) if isinstance(p, list) else str(p)
        out.append((label, expand(r), expand(p)))
    return out


def heat_image(ref, port, smap):
    """reference | port | heat-map (dark = alike, bright red = differs) for one 256x192 screen."""
    heat = np.zeros((H, W, 3), dtype=np.uint8)
    bad = np.clip(1.0 - smap, 0, 1)
    heat[..., 0] = (bad * 255).astype(np.uint8)
    heat[..., 1] = (np.abs(gray(ref) - gray(port)) / 2).astype(np.uint8)
    return np.concatenate([ref, port, heat], axis=1)


def compare_case(case, case_dir, write_images=True):
    frames_out = {}
    all_scores = []
    worst = 1.0
    failing_images = []
    phashes = {"ref": {}, "port": {}}
    for label, ref_frames, port_frames in frame_entries(case):
        best = None
        for rf in ref_frames:
            rp = os.path.join(case_dir, "ref_%d.png" % rf)
            if not os.path.exists(rp):
                continue
            ref = split_screens(load_rgb(rp))
            phashes["ref"][rf] = phash(load_rgb(rp))
            for pf in port_frames:
                pp = os.path.join(case_dir, "port_%d.png" % pf)
                if not os.path.exists(pp):
                    continue
                port = split_screens(load_rgb(pp))
                phashes["port"][pf] = phash(load_rgb(pp))
                screens = {}
                maps = {}
                for s in ("top", "bottom"):
                    r_, w_ = region_of(case, s, ref[s])
                    p_, _ = region_of(case, s, port[s])
                    sc, parts, smap = score_pair(r_, p_, w_)
                    screens[s] = parts
                    maps[s] = (r_, p_, smap)
                m = min(screens["top"]["score"], screens["bottom"]["score"])
                if best is None or m > best[0]:
                    best = (m, rf, pf, screens, maps)
        if best is None:
            frames_out[label] = {"error": "no ref/port frames found for %s" % label}
            worst = 0.0
            continue
        m, rf, pf, screens, maps = best
        frames_out[label] = {"ref_frame": rf, "port_frame": pf, "top": screens["top"], "bottom": screens["bottom"], "pass": m >= PASS_SCORE}
        all_scores += [screens["top"]["score"], screens["bottom"]["score"]]
        worst = min(worst, m)
        if m < PASS_SCORE and write_images:
            rows = []
            for s in ("top", "bottom"):
                r_, p_, smap = maps[s]
                if r_.shape[:2] != (H, W):  # region crops: pad back to a full-width strip for the image
                    full_r = np.zeros((H, W, 3), np.uint8)
                    full_p = np.zeros((H, W, 3), np.uint8)
                    full_m = np.ones((H, W))
                    full_r[: r_.shape[0], : r_.shape[1]] = r_
                    full_p[: p_.shape[0], : p_.shape[1]] = p_
                    full_m[: smap.shape[0], : smap.shape[1]] = smap
                    r_, p_, smap = full_r, full_p, full_m
                rows.append(heat_image(r_, p_, smap))
            failing_images.append(("%s_%s.png" % (case["id"], label), np.concatenate(rows, axis=0)))
    timing = None
    if case.get("timing"):
        timing = timing_check(case["timing"], phashes)
    passed = bool(all_scores) and worst >= PASS_SCORE and (timing is None or timing["pass"])
    report = {
        "case": case["id"],
        "screen": case.get("screen"),
        "frames": frames_out,
        "score": round(float(np.mean(all_scores)), 4) if all_scores else 0.0,
        "min_screen_score": round(float(worst), 4),
        "timing": timing,
        "pass": passed,
    }
    return report, failing_images


def timing_check(spec, phashes):
    """Per side, the number of frames after `trigger` until the perceptual hash first differs from the
    trigger frame's by > 8 bits; the two latencies must agree within TIMING_TOL. `trigger` is an int
    (same on both timelines) or {"ref": F, "port": G}. Needs dense frames after the trigger."""
    trig = spec["trigger"]
    trig = trig if isinstance(trig, dict) else {"ref": trig, "port": trig}
    out = {"trigger": trig}
    for side in ("ref", "port"):
        hs = phashes[side]
        base_frames = [f for f in sorted(hs) if f >= trig[side]]
        if not base_frames:
            out[side] = None
            continue
        base = hs[base_frames[0]]
        change = None
        for f in base_frames[1:]:
            if hamming(hs[f], base) > 8:
                change = f - trig[side]
                break
        out[side] = change
    ok = out.get("ref") is not None and out.get("port") is not None and abs(out["ref"] - out["port"]) <= TIMING_TOL
    out["pass"] = bool(ok)
    return out


# ---- driver ----------------------------------------------------------------------------------------------------
def load_manifest():
    with open(MANIFEST) as f:
        return json.load(f)


def has_goldens(case):
    d = os.path.join(UI_REF, case["id"])
    return os.path.isdir(d) and any(fn.startswith("ref_") and fn.endswith(".png") for fn in os.listdir(d))


def has_port_frames(case):
    d = os.path.join(UI_REF, case["id"])
    return os.path.isdir(d) and any(fn.startswith("port_") and fn.endswith(".png") for fn in os.listdir(d))


def previous_scores():
    try:
        return json.load(open(PROGRESS)).get("ui_cases", {})
    except (OSError, ValueError):
        return {}


def run(case_ids, regress=False):
    man = load_manifest()
    cases = [c for c in man.get("cases", []) if not case_ids or c["id"] in case_ids]
    with_goldens = [c for c in cases if has_goldens(c)]
    if not with_goldens:
        print("ui-diff: no goldens, skipped")
        return 0
    os.makedirs(REPORT_DIR, exist_ok=True)
    prev = previous_scores() if regress else {}
    rc = 0
    for c in with_goldens:
        case_dir = os.path.join(UI_REF, c["id"])
        if not has_port_frames(c):
            print("SKIP %-24s no port frames (run capture_port.py %s)" % (c["id"], c["id"]))
            continue
        report, images = compare_case(c, case_dir)
        with open(os.path.join(REPORT_DIR, c["id"] + ".json"), "w") as f:
            json.dump(report, f, indent=1)
            f.write("\n")
        for name, arr in images:
            Image.fromarray(arr).save(os.path.join(REPORT_DIR, name))
        status = "PASS" if report["pass"] else "FAIL"
        line = "%s %-24s score %.3f min %.3f" % (status, c["id"], report["score"], report["min_screen_score"])
        if report["timing"]:
            line += " timing ref=%s port=%s" % (report["timing"].get("ref"), report["timing"].get("port"))
        if regress and c["id"] in prev and prev[c["id"]].get("score") is not None:
            old = prev[c["id"]]
            drop = old["score"] - report["score"]
            if (old.get("pass") and not report["pass"]) or drop > REGRESS_DROP:
                line += "  REGRESSION (was %.3f%s)" % (old["score"], ", passing" if old.get("pass") else "")
                rc = 1
        elif not regress and not report["pass"]:
            rc = 1
        print(line)
    return rc


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--case", action="append", default=[])
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--regress", action="store_true", help="exit 1 only on regressions vs docs/migration/PROGRESS.json")
    ap.add_argument("--hash", help="print the perceptual hash of one PNG and exit")
    a = ap.parse_args(argv)
    if a.hash:
        print(phash(load_rgb(a.hash)))
        return 0
    if not a.case and not a.all:
        ap.error("--case ID or --all")
    if not os.path.exists(MANIFEST):
        print("ui-diff: no manifest, skipped")
        return 0
    return run(a.case, a.regress)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
