# black-game

The game app: Bevy shell, state machine, rendering, input, audio, screens. Native + wasm.
The only crate that depends on Bevy (D6).

G1 skeleton: Bevy 0.20 (`default-features = false, features = ["2d"]`), two stacked 256x192
screens, each rendered by its own camera into an offscreen nearest-neighbour image, composited
at the largest integer scale that fits the window (black letterbox). `wasm-bindgen` is pinned
to `=0.2.129` (matches the installed CLI).

## Run

- Native: `cargo run -p black-game` (from `black-rust/`). `BLACK_SCREENSHOT=out.png` saves a frame and exits.
- Browser (local only, never deploy, D5): `cd crates/black-game && trunk serve`, open http://127.0.0.1:8080.
  Without trunk: `cargo build -p black-game --target wasm32-unknown-unknown`, then
  `wasm-bindgen --target web --out-dir dist/pkg <target>/wasm32-unknown-unknown/debug/black-game.wasm`
  and load `dist/pkg/black-game.js` from a page with `<canvas id="bevy">`.

## Asset loading (G3, `src/loading/`)

Bevy `AssetServer` over the extracted tree (D4, gitignored, D5). `assets::asset_plugin()` points `AssetPlugin` at
`$BLACK_ASSETS` (default `./assets`, made absolute) on native and at the relative URL `assets/` on wasm; meta files are never
probed. `types.rs`: one newtype asset + loader per RON table (`PersonalAsset`, ... via `black_data::loader::load_<table>`, bytes
in so wasm-safe), `TextBankAsset` (`text/en/*.json`, raw UTF-8), `AtlasAsset` (`*.atlas.ron` bytes; parse with
`black_data::loader::load::<T>`), `ManifestAsset` (`manifest.json`, parsed by `manifest.rs`). PNG goes through Bevy's `Image`;
OGG/glTF need Bevy features that are not enabled yet.

`Screen::Loading` is the default state. `OnEnter` starts `manifest.json` plus the 12 `data/*.ron` tables (`GameTables`); once the
manifest is read it adds every `text/en/*.json` it lists (`TextBanks`) and records required files it does not list
(`LoadReport::unlisted`). When all tracked assets are loaded or failed, the app enters `Boot`. Failures never block (logged,
in `LoadReport::failed`), so a partial tree (e.g. `--only fonts`, used by tools/ui-diff) still runs. A progress bar is drawn
on the bottom screen (`ProgressScreenPlugin`). The frame counter, flow ticks and scripted input are gated off during
`Loading`, so `BLACK_SCREENSHOT_FRAMES` / `BLACK_SCRIPT` frame numbers still start at `Boot`. Sprites/audio/models are loaded on
demand by the screens that need them, using paths from the manifest. Manifest sha1/size are not verified yet.
The UI font still uses the direct native `std::fs` loader in `ui/widgets.rs` (not moved to the asset server).

### Serving assets for the browser (local only, never deploy: D5/R4)

The wasm build fetches `assets/<path>` relative to the page URL, so the page's directory must contain `assets/`.
Trunk wipes `dist/` on every build, so link the tree in afterwards:

```sh
cd crates/black-game && trunk build            # writes dist/
ln -sfn "$PWD/../../assets" dist/assets        # extracted tree (black extract-assets --out assets)
python3 -m http.server 8080 --bind 127.0.0.1 --directory dist
```

(`trunk serve` also works if `dist/assets` is re-linked after each rebuild.) Keep it on 127.0.0.1; the assets are extracted
from a private ROM and must never be hosted publicly. A missing `assets/` only shows up as failed loads in the console.

## App state machine (G2)

`src/flow.rs` is the pure part (no window; unit-tested): `Screen` (`Boot -> Title -> NewGame | Continue -> Overworld <-> Battle`,
a Bevy `States`), `Overlay` (`Closed, StartMenu, Party, Bag, Summary, PCBox, Pokedex, Save`, a `SubStates` that only exists
while `Screen::Overworld`; B from a sub-menu returns to the start menu, B from the start menu closes it), the debug
transition table `on_button`, auto-advance `on_auto`, and the fade state machine `Fade`.
`src/screens.rs` wires it to Bevy: one placeholder per state (coloured rect + text label on the top screen, hint on the
bottom), the overlay panels, and the fade rects.

Fades mimic the DS master-brightness fade: 16 levels (`darkness(frame, total)`, 0 = clear, 16 = black), one black sprite per
screen whose alpha is `level/16`, ticked in `FixedUpdate` at 60 Hz. A screen change fades out, switches state while fully
black, then fades in; overlay open/close is instant. Input is ignored during a fade. Durations are in `FlowConfig`
(`fade_out_frames`, `fade_in_frames`, `boot_frames`, `load_frames`; env `BLACK_FADE_FRAMES=n` overrides both fades).

Debug controls (default keyboard map in `input.rs`): Title: A (or a tap on the bottom screen) = new game, Y = continue;
Boot/NewGame/Continue advance on their own. Overworld: Start = start menu, X = battle. Start menu: L party, R bag,
X summary, Y PC box, Select pokedex, A save, B / Start close. Battle: A returns to the overworld.

Headless capture: `BLACK_SCREENSHOT=out.png BLACK_SCREENSHOT_FRAME=200 BLACK_SCRIPT="70:A,180:Start,190:L" cargo run -p black-game`
(`BLACK_SCRIPT` = `frame:Button` presses by update frame; the capture frame defaults to 30 and the app exits 30 frames later).

### Intended real hooks (not wired yet)

- Boot: starts the asset loader (A11 `extract-assets` output; D5: assets are never committed) and shows progress; leaves Boot
  when loading is done instead of after `boot_frames`. Mirrors the system init of `gfl_use.c` (`unk_02004FD4..unk_02006118`).
- Title / NewGame / Continue: U1 (title + intro) replaces the placeholders; Continue calls the save loader (S5/G9) and
  NewGame the new-game script (`black-script`) before entering the overworld.
- Overworld: the Bevy view of `black-field` state; `black-script` runs event scripts and requests `Request::Screen(Battle)`
  (wild/trainer encounter) or overlay changes. The per-button overlay table becomes menu-driven (cursor + touch).
- Battle: entered with a battle setup; the Bevy battle UI consumes `black-battle`'s server->client command stream only
  (D7, `btl_adapter`/`btl_client` seam, B12) and hands the result back to the field on exit.
- Overlays map to the original menu overlays (party, bag, summary, PC, pokedex, save); each reads game state from the
  logic crates and never owns it.
- Fades: the original's brightness-fade requests become `Fade::start`; per-screen lengths come from `FlowConfig`.

## UI kit (G5, `src/ui/`)

Pure (unit-tested, no window): `font` (metrics parser, `GlyphWidth` impl, `place` glyph layout, `encode` for demo strings),
`msgbox` (paging + typewriter), `list` (list cursor/scroll/wrap, yes/no mapping, number input). Bevy side: `widgets`
(`UiPlugin`, font loading, `BitmapText`, `spawn_msgbox` / `spawn_list` / `spawn_number`), `demo` (G2 placeholder wiring:
Title shows a typewriter message box; the start menu is a real scrolling list; Overworld + Y runs message box -> yes/no ->
number input -> result message).

- **Fonts (D4)**: read from `$BLACK_ASSETS` (default `assets/`) `fonts/font0.png` + `font0.metrics.ron` as written by
  `black extract-assets --only fonts` (atlas of `cell_w x cell_h` cells, tuples `(atlas index, UTF-16 code, left, glyph
  width, advance)`). Nothing is read from the ROM. Native only for now: on wasm the font is not loaded (text is skipped,
  layout falls back to 6 px advances) until G3 loads assets over HTTP. The 2 bpp atlas is recoloured at load: level 1 = body
  (opaque, tintable), level 2 = shadow (translucent); UNVERIFIED reading of the glyph levels. Font 0 (12x15) is used
  everywhere; which font the game uses where is UNVERIFIED (A4b).
- **black-text use**: `layout::wrap` (our helper; the original never wraps) + `layout::paginate` (2 lines/page, `BE01` clear)
  with `FontMetrics` as `layout::GlyphWidth`; `token::parse` splits chars/tags (`BE00` = wait for A/B, other tags dropped);
  `speed::{option_speed, pace, Pacer}` set the reveal rate (`BLACK_TEXT_SPEED=0..2`, default 0; mapping to the 60 Hz tick is
  UNVERIFIED). Glyph positions use the same measure as `layout::line_width` (`spacing + advance`).
- **Input**: `UiInput` is built each frame from `DsButtons` (cursor keys include DS key repeat) plus `BLACK_SCRIPT`. Touch is
  not wired into the widgets yet. Widgets have an `active` flag; consumers read `ListWidget::event`, `NumberWidget::event`,
  `MsgBoxWidget::finished` after `UiSet::Widgets`.
- **Window frame**: placeholder (dark outline, blue rim, white fill; `frame_rects`). The real frame graphics (the DS window
  frame tile sheets, small NCGR + NCLR pair) were not located in the ROM in this item; a later converter should search the
  ARC inventory (`tools/narc-inventory.py`) for small NCGR members next to NCLR palettes (no ARC id known) and replace
  `frame_rects` with a 9-slice sprite.
- Flow change: `StartMenu` + A no longer maps to Save in `flow::on_button` (the list widget takes A and picks the overlay).
