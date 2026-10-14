#!/usr/bin/env python3
"""Self-test for compare.py (venv only; not part of the gate):

    /tmp/blk/venv/bin/python -I tools/ui-diff/test_compare.py

Checks: identical images score 1.0; a 1 px shift scores > 0.9; an all-black screen against a busy synthetic
"title screen" scores < 0.3. The synthetic screen is generated here (no ROM material, D5).
"""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import compare  # noqa: E402


def synthetic_title(seed=1):
    """A DS-like screen: flat sky, a logo box, text-like dashes, a ground band, a few sprites."""
    rng = np.random.default_rng(seed)
    img = np.zeros((compare.H, compare.W, 3), np.uint8)
    img[:] = (96, 160, 224)  # sky
    img[140:] = (72, 136, 48)  # ground
    img[150:154] = (40, 90, 30)
    img[20:70, 48:208] = (240, 240, 240)  # logo box
    img[24:66, 52:204] = (200, 40, 40)
    for i in range(6):  # text lines
        y = 92 + i * 7
        for x in range(60, 200, 9):
            if rng.random() < 0.8:
                img[y: y + 4, x: x + 6] = (20, 20, 20)
    for _ in range(8):  # sprites
        x, y = int(rng.integers(0, 236)), int(rng.integers(150, 180))
        img[y: y + 10, x: x + 10] = tuple(int(v) for v in rng.integers(0, 255, 3))
    return img


def stack(top, bottom):
    return np.concatenate([top, bottom], axis=0)


def shift1(img):
    out = np.roll(img, 1, axis=1)
    out[:, 0] = img[:, 0]
    return out


def main():
    title = synthetic_title()
    bottom = synthetic_title(seed=7)
    s_same, _, _ = compare.score_pair(title, title)
    s_shift, parts_shift, _ = compare.score_pair(title, shift1(title))
    black = np.zeros_like(title)
    s_black, parts_black, _ = compare.score_pair(title, black)
    print("identical %.4f  shifted-1px %.4f %s  black %.4f %s" % (s_same, s_shift, parts_shift, s_black, parts_black))
    ok = True
    if abs(s_same - 1.0) > 1e-9:
        print("FAIL identical != 1.0")
        ok = False
    if s_shift <= 0.9:
        print("FAIL shifted-1px <= 0.9")
        ok = False
    if s_black >= 0.3:
        print("FAIL black >= 0.3")
        ok = False
    # phash: identical hashes, black far away
    h1, h2, h3 = compare.phash(stack(title, bottom)), compare.phash(stack(shift1(title), bottom)), compare.phash(stack(black, black))
    print("phash same/shift/black hamming:", compare.hamming(h1, h2), compare.hamming(h1, h3))
    if compare.hamming(h1, h2) > 8 or compare.hamming(h1, h3) <= 8:
        print("FAIL phash distances")
        ok = False
    print("test_compare:", "ok" if ok else "FAILED")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
