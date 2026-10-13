# 2D Sprite Generator Proposal

Oct 13, 2026 · @spencer

## Recommendation

Build a local, command-line sprite generator with three parts: a small pretrained image model (Stable Diffusion 1.5 class), a style add-on (LoRA) trained on Pokémon Black's own sprites, and a deterministic "finisher" that forces every output to the game's exact size and 16-colour rules. Reference images (up to 10) go in through an image-prompt add-on (IP-Adapter), never as a starting canvas.

Training a model from scratch is not recommended: about 3,000 game sprites is far too few to teach a model what a "mad scientist" is. Fine-tuning a model that already knows the world is what makes text and references work.

Everything runs CPU-only on your c5.4xlarge (16 vCPUs, 32 GB RAM, no GPU). It will be slow, minutes per image and roughly a day per training run, but free to run forever and with no LLM in the loop. Ship it in phases, starting with the finisher, which is useful on day one even for hand-drawn art.

## What the game's 2D art actually is

The Rust project already extracts all of these to PNG with `black extract-sprites`. Every 2D sprite uses a 16-colour palette where colour 0 is see-through.

| Category | Size (pixels) | Count in the game | Animated? | Extra variants |
| --- | --- | --- | --- | --- |
| Pokémon battle sprites | 96×96 | 1,616 sets (front and back) | Yes: cut into parts that move like a puppet | Shiny palette, female form for 192 |
| Trainer battle sprites | 64×64 | 95 sets | Yes, same puppet system | One palette |
| Item icons | 32×32 | 627 items (many share icons) | No | None |
| Pokémon menu icons | small, 2 frames | 715 | Simple 2-frame wiggle | None |

One correction to the brief: Pokémon Black's battle sprites are animated, and famously so. Each one is a single still picture cut into pieces (head, arms, tail) that the game moves around. That's good news for this tool: the generator only has to draw the still picture, and animation is a separate, later step. Also, dialog boxes in this game don't show character portraits, so battle and menu screens are where 2D characters appear.

Items and icons are the easy, high-volume case. Pokémon and trainer sprites are where quality matters most.

## Options considered

Fine-tuning a pretrained model wins because it's the only option that understands new ideas from text and references while staying cheap enough to run on a CPU.

| Option | Understands "mad scientist"? | Matches the style? | Runs on the c5 with no GPU? | Verdict |
| --- | --- | --- | --- | --- |
| Train a small image model from scratch on game sprites | No: \~3,000 images only teach it to remix existing Pokémon | Yes | Yes | Rejected |
| 3D model rendered down to 2D (the z-day tools approach) | Yes, via the LLM | Poorly: 3D renders don't look hand-pixelled at 96×96 | Yes | Rejected for this; fine for 3D-native assets |
| LLM writes pixel grids directly | Somewhat | Stiff, drifts between images | Yes | Rejected: costs tokens per image |
| Hosted image API | Yes | Only with heavy prompting, no style training | n/a, pay per image | Rejected: not free to run |
| Pretrained model + game-style LoRA + finisher | Yes | Yes, once trained | Yes, slowly | Recommended |

I couldn't open the z-day tool folders from here (they're on your machine), so that row goes on your description of them. Rendering 3D to 2D works for item-like objects but tends to look "rendered" rather than hand-drawn, which shows most on characters.

## How the tool works

One command, `spritegen run`, takes a list of characters and produces a contact sheet of finished, game-ready candidates. You pick; nothing else needs a person or an LLM.

&#91;embedded content: spritegen pipeline · 8 steps, 1 human pick\]

- **Reference prep** crops each picture to its main subject and squares it, so movie stills, drawings and thumbnails all work.
- **IP-Adapter** turns references into a description of *what* is in them, averaged across all of them, never their pixels. Content-only mode keeps a photo from making the output look photographic.
- **The finisher** is the piece that makes output look authentic: colour-aware shrinking to the exact game size, at most 15 colours plus see-through, the dark outline the game's sprites use, and centring on the canvas.
- **The style checker** compares palette, outline and silhouette against real game sprites and sorts candidates best first.

## Running it on one c5.4xlarge with no GPU

The c5 can do all of it if the model stays small: SD 1.5 class at 512×512, not SDXL or Flux. The times below are my rough estimates, not measurements; Phase 0 benchmarks them before anything else is built.

| Job | Estimated time on the c5 | Memory | Notes |
| --- | --- | --- | --- |
| Generate one candidate (SD 1.5, 512×512, \~25 steps) | about 1 to 3 minutes | \~6 GB | Intel's OpenVINO runtime can speed this up on Xeon CPUs |
| Overnight batch (10 hours) | roughly 200 to 600 candidates | same | 50 to 150 characters at 4 candidates each |
| Train one LoRA (\~1,500 steps) | roughly 12 to 36 hours | \~16 to 24 GB | Fits in 32 GB with gradient checkpointing; run once per category |
| Finisher + style check per image | under a second | small | Plain Python, no model |
| SDXL instead of SD 1.5 | \~10+ min per image; training impractical in 32 GB | tight | Avoid on this machine |

If the training estimate turns out much worse in Phase 0, the fallback is renting a GPU for one training run (a few dollars), then going back to CPU for all generation. You said a week of training is fine, so this is unlikely to be needed.

## Training data and captions

The game already contains everything needed to label its own art, so captions cost zero LLM tokens.

- **Images.** Pokémon front sprites (the best style signal), trainer sprites and item icons, all from `black extract-sprites`. Back sprites are left out at first; they're a different pose.
- **Preparation.** Each sprite is enlarged 5× with hard pixel edges (96 becomes 480), centred on a 512×512 canvas, see-through areas filled with one flat background colour.
- **Captions, written by a script.** Built from the game's own data tables and text: Pokémon name, its types, body colour and the Pokédex description; trainer class names ("Scientist", "Ace Trainer"); item names and descriptions. Example: `pkmnblack sprite, front view, fire type, red, a lizard with a flame on its tail`.
- **A trigger word** (`pkmnblack sprite`) tells the model when to use the style.
- **One LoRA per category.** Creatures, trainers and items look different enough that three small add-ons beat one big one. Each is a file of tens of MB.

Names of real Pokémon are dropped from about half the captions so the model learns the style, not "draw Charmander". This matters because the goal is new characters.

## How you and the LLM use it

You write one row per character; the tool does the rest; the LLM only fills in rows and imports your picks. No LLM looks at or edits images.

```csv
name,category,description,refs,strength,candidates
dr_vexx,trainer,evil mad scientist with wild white hair and cracked goggles,refs/dr_vexx/,0.6,8
voltmoth,pokemon,electric moth with glowing yellow wing patterns,refs/voltmoth/,0.5,8
repair_kit,item,small red toolbox with a white cross,,0,4
```

- **Run:** `spritegen run jobs.csv` on the c5, overnight if the batch is large.
- **Review:** open one contact sheet per character showing every candidate at real size and enlarged, with its style score. Write down the winners (or mark them in a picks file).
- **Fix misfires:** change the description, add references or adjust strength for just those rows and rerun.
- **Import:** `spritegen export picks.txt` writes each winner as a game-ready PNG (plus shiny palette for Pokémon) into the Rust game's assets folder with its list entry. This is the step an LLM can run for you.

`strength` sets how much the references count (0 = ignore, 1 = follow closely). `refs` is a folder with 1 to 10 pictures of any size or style.

## Phased plan, smallest piece first

Each phase ships something usable on its own, and each one ends with a check that decides whether the next is needed.

1. **Benchmark (Phase 0).** Install SD 1.5 and OpenVINO on the c5, time one image and 20 training steps. Done when the timing table above is replaced with real numbers.
2. **Finisher and importer (Phase 1).** Snap any image to 96×96, 64×64 or 32×32, 15 colours plus see-through, dark outline, then write it into the game's assets. Done when a real game sprite run through an enlarge-blur-finish round trip comes back near-identical. Useful immediately for hand-drawn or edited art.
3. **Baseline with no training (Phase 2).** Plain SD 1.5 + a "pixel art" prompt + the finisher on 10 test characters. Done when you've judged how close it gets; items may already be good enough here.
4. **Style LoRAs (Phase 3).** Train creatures first, then trainers, then items. Done when, in a blind side-by-side, you can't reliably pick generated sprites out from real ones at game size.
5. **References (Phase 4).** Add IP-Adapter in content-only mode, averaging up to 10 reference pictures. Done when a character from references keeps its look across 4 candidates.
6. **Style checker (Phase 5).** Auto-score palette, outline and silhouette against real sprites; hide low scorers from contact sheets. Also flags outputs too similar to an existing Pokémon.
7. **Shiny palettes and battle animation (Phase 6).** Shiny = an automatic colour shift, easy. Animation = borrowing the puppet cut-up and motion of an existing Pokémon with a similar body shape; this is its own project and waits until the battle screen exists in the Rust game.

## Risks and open questions

Expect to pick 1 good sprite out of 4 to 8 candidates and to touch up a few pixels on hero characters. That's still far faster than drawing, but it isn't "perfect every time".

| Risk | Why it happens | Mitigation |
| --- | --- | --- |
| 96×96 creatures lose detail | Much bigger than the 40×40 portraits in your other project; fine details blur when shrunk | Train at 5× scale; finisher uses colour-aware shrinking, not plain resizing |
| Model copies existing Pokémon | Small training set memorised | Drop names from captions; style checker flags near-copies |
| References pull in photo style | Photos drag the look toward realism | IP-Adapter content-only mode plus the strength dial |
| CPU training slower than estimated | Estimates are unmeasured | Phase 0 benchmark; one rented-GPU training run as a fallback |
| New battle sprites can't animate yet | Animation uses per-sprite puppet data | Phase 6; stills work everywhere meanwhile |

- [ ] How does the Rust battle screen want new sprites registered? It doesn't exist yet, so `spritegen export` will target the extracted asset layout and adjust later.
- [ ] Personal use only: the training art and anything the model learns from it belong to Nintendo, so keep the model and outputs inside this fan project.

Sources: `black-rust/crates/black-cli/src/cmd_extract_sprites/` (pokemon.rs, trainers.rs, items.rs, overworld.rs) and `MIGRATION-AGENT.md` items A3, A10, A12 at commit `ea8cae4`. Timing figures are estimates pending Phase 0.
