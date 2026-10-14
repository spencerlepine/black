# tools/ui-diff — UI comparison testing (Phase G/U guardrail, item U0)

Screens are rewritten in Bevy (D3), so nothing differential checks them the way `tools/rom-diff`
checks logic. This kit scores the port's frames against reference frames of the original, captured headless by
running the game in the DeSmuME core with a frame-exact input script (no human, no emulator GUI). All
image work is local compute; an agent reads numbers and looks at a picture only when a number is red (at
most 3 images per item, `MIGRATION-AGENT.md` §5 "UI screens").

Reference frames and savestates are Nintendo IP (D5): they live in `black-rust/ui-ref/` which is
**gitignored** except `manifest.json`; the gate fails if anything else under `ui-ref/` is tracked. Only
hashes, scores and scripts are committed, and everything regenerates from them.

## Files

| file                       | who runs it              | what                                                                                                                                                              |
| -------------------------- | ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ui-ref/manifest.json`     | committed                | the cases: id, screen (checklist item), port script + frames, ref script/savestate + frames, regions/weights, optional timing, recorded reference phashes          |
| `capture_ref.py <case>`    | agent (headless)         | runs the original in the DeSmuME core (py-desmume, `/tmp/blk/venv-dsm`) from power-on or `ui-ref/states/<name>.dst`, presses the `ref.script` buttons, writes `ui-ref/<case>/ref_<frame>.png`, records phashes; `--save-state` makes reusable states |
| `capture_port.py <case>`   | agent (headless)         | runs `cargo run -p black-game` with `BLACK_SCRIPT`, `BLACK_FADE_FRAMES`, `BLACK_SCREENSHOT_FRAMES`, crops the window capture to `ui-ref/<case>/port_<frame>.png` |
| `compare.py --case <case>` | agent / validator / gate | scores ref vs port per screen per frame → `ui-ref/report/<case>.json`; on failure also `report/<case>_<frame>.png` (reference \| port \| heat-map, 768 px wide)   |
| `test_compare.py`          | anyone (venv)            | self-test of the metric: identical = 1.0, 1 px shift > 0.9, black vs busy screen < 0.3                                                                            |

Python: `/tmp/blk/venv/bin/python -I` for `compare.py`/`test_compare.py` (pillow + numpy; `doctor.sh`
installs them), `/tmp/blk/venv-dsm/bin/python -I` for `capture_ref.py` (py-desmume needs CPython ≤ 3.13;
`doctor.sh` builds that venv), plain `python3 -I` for `capture_port.py` (stdlib). On Linux without a display
`capture_port.py` wraps the game in `xvfb-run -a` (`doctor.sh` checks for it).

## Metric

Both images are quantised to RGB555 (the DS colour depth) first. Per screen, per frame:

- **structure** = SSIM on grayscale, 7×7 uniform window;
- **colour** = 1 − mean ΔE76 in CIE Lab over 8×8 blocks, scaled so ΔE 20 → 0;
- **layout** = IoU of Canny edge maps after a 2 px dilation (tolerates font anti-aliasing and 1 px offsets);
- `score = 0.5·structure + 0.3·colour + 0.2·layout`; **pass ≥ 0.70** (strive higher). A region in the
  manifest can override the weights (a text box: layout 0.5 / structure 0.3 / colour 0.2) and the rect.

Frames: an entry is an int or a `[lo, hi]` window; every frame in a window is captured on its side and
the best-matching pair counts (manual timing jitter). Cases with `"timing": {"trigger": {"ref": F, "port": G}}`
(or one int for both) also report, per side, how many frames after the trigger the perceptual hash first
changes; the two latencies must agree within ±3 frames (list dense frames after the trigger for that). All metrics are plain numpy (scikit-image has no wheel for
the venv's Python); results are deterministic.

## Who does what

- **Reference side** (any agent, once per case, cached): `/tmp/blk/venv-dsm/bin/python -I
  tools/ui-diff/capture_ref.py <case>` (`--all`, `--force`). The DeSmuME core boots the ROM directly: frame 0
  is power-on, the health screen needs `A` at ~400, the intro movie runs to the title screen at ~4900
  (logo top, Reshiram bottom; the layout flips ~5500). States that take long scripts to reach are saved
  once with `--save-state NAME --script "..." --at N` into `ui-ref/states/` and named by `ref.savestate`.
  Save type is forced to FLASH 4 Mbit (autodetect leaves the game on a white screen); the core is a
  process singleton (`reset()` between cases). ~160 frames/s on an M-series Mac. melonDS is not used:
  version 1.1 has no scripting interface.
- **Human**: sign-off decisions only (`HUMAN.md`), never captures.
- **UI worker** (Phase G/U item): edit → `python3 -I tools/ui-diff/capture_port.py <case>` →
  `/tmp/blk/venv/bin/python -I tools/ui-diff/compare.py --case <case>` → read the JSON. Add cases for the
  screen to the manifest (animations as multi-frame cases). Open a failure PNG at most 3 times per item; then
  `[x] pending sign-off` with the best score. Workers never edit `tools/ui-diff/*` (same hands-off rule as
  `tools/rom-diff/*`).
- **Validator** (V6): `compare.py --all`, files a finding when any case dropped by > 0.05 against the score
  recorded in the last `docs/migration/PROGRESS.json`. Never opens images.
- **Gate**: `compare.py --all --regress` when goldens exist — fails if a previously passing case now fails or
  dropped > 0.05; prints `ui-diff: no goldens, skipped` otherwise.
- **progress.py**: a `[x]` Phase U item counts fully only when every manifest case for that screen passes;
  the per-case scores are copied into `PROGRESS.json` (`ui_cases`).

## Writing a case

```json
{ "id": "U6-party-open", "screen": "U6",
  "port": {"script": "70:Start,120:Down,150:A", "fade_frames": 16, "frames": [160, 175, 190]},
  "ref":  {"savestate": "overworld.dst", "script": "70:Start,120:Down,150:A", "hold": 4, "frames": [160, 175, 190]},
  "regions": {"top": "whole", "bottom": {"rect": [0, 0, 256, 192], "weights": [0.3, 0.2, 0.5]}},
  "timing": {"trigger": {"ref": 150, "port": 150}}, "phash": {} }
```

`port.script` is `BLACK_SCRIPT` (fixed-frame:button pairs). The two timelines differ: the port counts from
power-on (Boot → Title with 16-frame fades, Title visible from ~frame 79; `A` → NewGame → Overworld visible
~94 frames later), the reference from the savestate load; `ref.frames[i]` is compared with `port.frames[i]`,
so list them in the same order. In capture mode the game lock-steps its clock to exactly one 60 Hz tick per
render frame (`lockstep_clock` in `main.rs`), so port frames are reproducible run to run.
