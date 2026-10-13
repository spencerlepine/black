# Asset Master Checklist

Oct 13, 2026 · @spencer

Pokémon Black's content breaks down into 32 asset categories in 8 families, which 12 tools can cover. Only 4 of those categories need a trained image model, and they share one base model with 4 small style add-ons (LoRAs), not one model each and not one model for everything.

This list comes from scanning all 234 archives in the game's file system (`black-rom/files/a/`) and the Rust project's extractors and plan. Where an archive's exact role isn't confirmed yet, the row says so.

## Families at a glance

The split that matters is what kind of thing the asset is (flat pixel art, tile-based screen art, 3D, or data), because that decides the tool, not where it appears in the game.

| Family | Categories | Kind of asset | Main tool | Image model? |
| --- | --- | --- | --- | --- |
| A. Creatures | 4 | 16-colour pixel art | Sprite Generator | Yes |
| B. People | 4 | 16-colour pixel art | Sprite Generator, Walker Workbench | Yes |
| C. Items and icons | 2 | 16-colour pixel art | Sprite Generator | Item icons only |
| D. Screens and UI | 5 | Tile-based screen art and fonts | UI Skin Editor | No |
| E. 3D world | 6 | 3D models and textures (glTF) | 3D World Builder | No |
| F. Effects and cutscenes | 2 | Animation scripts plus 2D and 3D pieces | Effects & Cutscene Editor | No |
| G. World data and story | 6 | Text and data files | Map & Event, Story, Game Data editors | No |
| H. Audio | 3 | Music and sound | Audio tool | No |

One gap to plan for: this game has no character portraits in its dialog boxes. If your lore needs them, that's a new category (and a new screen feature), not a replacement.

## Master checklist

Every category in the game, grouped by family. Counts are from the game's files and the Rust extractors; "unverified" marks an archive whose role isn't confirmed yet. Set Status as each category is customised.

| # | Family | Category | Where it shows | Format | Count | Source | Tool | Image model | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | A | Pokémon battle sprite, front | Battle, opponent side | 96×96 still cut into moving parts | 1,616 sets (front + back) | ARC 004 | Sprite Generator, then Rigger | Creatures | Not started |
| 2 | A | Pokémon battle sprite, back | Battle, your side | 96×96 still cut into moving parts | in the same sets | ARC 004 | Sprite Generator, then Rigger | Creatures | Not started |
| 3 | A | Shiny and female variants | Battle, menus | Alternate palette; alternate still | 1,616 shiny palettes; 192 female | ARC 004 | Palette Tool; Sprite Generator for female | None | Not started |
| 4 | A | Pokémon menu icons | Party, PC box | Small, 2-frame wiggle | 715 | ARC 007 | Sprite Generator (shrink mode) | None (shrunk from #1) | Not started |
| 5 | B | Trainer battle sprites | Trainer battles | 64×64 still cut into moving parts | 95 sets | ARC 072 | Sprite Generator, then Rigger | People | Not started |
| 6 | B | Player battle back sprites | Your side at battle start | Multi-part, animated throw | 2 × 6 sets | ARC 064, 073 | Sprite Generator, then Rigger | People | Not started |
| 7 | B | Overworld walkers (NPCs and player) | Overworld | 32×32 or 64×64 frames on flat cards | 758 sets, 5,521 frames | ARC 049 | Walker Workbench | Walkers | Not started |
| 8 | B | Walker frame timing | Overworld | Which frame shows per direction and step | 29 files | ARC 069 | Walker Workbench | None | Not started |
| 9 | C | Item icons | Bag, shops | 32×32 | 627 items, 496 icons | ARC 025 | Sprite Generator | Items | Not started |
| 10 | C | Type, category, status and ball icons | Battle, summary, Pokédex | Small labels | dozens | ARC 083, 055 | UI Skin Editor | None | Not started |
| 11 | D | Fonts | All text | Bitmap glyphs, 3 sizes | 3 fonts × 5,064 glyphs | ARC 023 | Font Editor | None | Not started |
| 12 | D | Window frames and dialog box | Everywhere | Tile art | a few | not located yet | UI Skin Editor | None | Not started |
| 13 | D | Battle UI | Battle, both screens | Tile art, gauges, menus | 153 pieces extracted so far | ARC 083 and others | UI Skin Editor | None | Not started |
| 14 | D | Menu screens: party, summary, bag, PC, Pokédex, trainer card, town map, naming, start and save | Menus | Full-screen tile art, 256×192 | \~120 small archives | many | UI Skin Editor | None | Not started |
| 15 | D | Title, intro, credits and Hall of Fame screens | Start and end of game | Tile art plus 3D | several | overlays 180–182, 203, 204 | UI Skin Editor; Effects & Cutscene Editor | None | Not started |
| 16 | E | Map terrain chunks | Overworld | 3D model per chunk | 572 | ARC 008 | 3D World Builder | None | Not started |
| 17 | E | Map textures | Overworld | Texture sets | 282 | ARC 014 | 3D World Builder | None | Not started |
| 18 | E | Buildings and placed props | Overworld | 3D models placed by position records | archive not identified | ARC 008 records | 3D World Builder | None | Not started |
| 19 | E | Animated map textures (warps, water) | Overworld | Texture animation | 116 (unverified) | ARC 176, 177 | 3D World Builder | None | Not started |
| 20 | E | Battle backgrounds | Battle | 3D ground and sky | 61 models | ARC 011 | 3D World Builder | None | Not started |
| 21 | E | Field 3D objects and effects (item balls, doors, grass) | Overworld | Small 3D models, texture animation | several archives (unverified) | e.g. ARC 045 | 3D World Builder | None | Not started |
| 22 | F | Move and battle animations | Battle | Effect scripts plus particle art | one per move, 559 moves | not mapped yet | Effects & Cutscene Editor | None | Not started |
| 23 | F | Cutscenes: intro, evolution, egg hatch, ending | Story moments | 3D models plus animation | \~400 non-terrain models in \~40 archives | e.g. ARC 103, 115, 160 | Effects & Cutscene Editor | None | Not started |
| 24 | G | Map layout | World | Grid of chunks plus zone headers | 255 grids, 427 zones | ARC 009, 012 | Map & Event Editor | None | Not started |
| 25 | G | Walkability grids | Overworld | 32×32 cells per chunk | 572 | ARC 008 | Map & Event Editor | None | Not started |
| 26 | G | Events: NPCs, warps, signs, triggers | Overworld | Records per zone | 428 | ARC 125 | Map & Event Editor | None | Not started |
| 27 | G | Scripts | Story logic | Script commands | 899 | ARC 057 | Story Editor | None | Not started |
| 28 | G | Text | Dialog, menus, names | Text banks | 760 banks, 56,700 lines | ARC 002, 003 | Story Editor | None | Not started |
| 29 | G | Game data: species, moves, items, trainers, wild encounters | Everywhere | Data tables | 669 species, 560 moves, 627 items, 616 trainers, 112 encounter tables | ARC 016–024, 092, 093, 126 | Game Data Editor | None | Not started |
| 30 | H | Music | Everywhere | Sequences, rendered to OGG | 211 | sound archive | Audio Tool | None | Not started |
| 31 | H | Sound effects | Everywhere | Rendered to OGG | 726 | sound archive | Audio Tool | None | Not started |
| 32 | H | Pokémon cries | Battle, Pokédex | One per species | not extracted yet | sound archive | Audio Tool | None | Not started |

Archive numbers refer to `black-rom/files/a/` (ARC 049 = `a/0/4/9`). ARC 052 is a second copy of the Pokémon sprite layout whose role is unconfirmed.

## The tools

Twelve tools, split by what they edit, not by where the asset appears. One tool can cover many categories when the files are the same kind: the Sprite Generator handles 6 categories, the 3D World Builder 6.

| Tool | Covers (checklist #) | Input → output | Image model inside? |
| --- | --- | --- | --- |
| Sprite Generator | 1, 2, 4, 5, 6, 9 | Description + up to 10 references → finished 16-colour sprite at game size; shrink mode makes menu icons from a battle sprite | Yes: picks the add-on by category |
| Battle Sprite Rigger | 1, 2, 5, 6 | Still sprite → cut into parts plus motion, borrowed from an existing creature or trainer with a similar body | No |
| Palette Tool | 3, and recolours anywhere | Sprite + palette rule → shiny or recoloured version | No |
| Walker Workbench | 7, 8 | Base walker + edits → frame sheet, frame timing, animated preview | Optional: Walkers add-on drafts new heads and outfits |
| UI Skin Editor | 10, 12, 13, 14, 15 | Screen tile art → re-skinned screens with the same layout | No: layouts must stay pixel-exact |
| Font Editor | 11 | Glyph sheet + widths → new font | No |
| 3D World Builder | 16, 17, 18, 19, 20, 21 | Blender scripts or edited glTF → terrain, buildings, props, battle backgrounds, textures | No |
| Map & Event Editor | 24, 25, 26 | Map grid, walkability, NPC and warp placement → map data files | No |
| Story Editor | 27, 28 | Scripts and dialog as text → game script and text banks | No (an LLM is useful here) |
| Game Data Editor | 29 | Plain-text tables → species, moves, items, trainers, encounters | No |
| Effects & Cutscene Editor | 15, 22, 23 | Re-point or edit animation scripts and their art | No |
| Audio Tool | 30, 31, 32 | OGG files and loop points → music, sound effects, cries | No |

The Sprite Generator, Rigger and Palette Tool are one pipeline in practice: generate the still, rig it, derive the shiny.

## Image models: how many

Download one pretrained base model, then train 4 small style add-ons on top of it: Creatures, People, Items and Walkers. That's the middle ground between one model for everything and one model per category.

&#91;embedded content: image models · 1 base, 4 add-ons, 1 optional\]

- **Why not one add-on for everything:** the categories disagree on size (96, 64 and 32 pixels), viewpoint (side-on battle poses vs. tiny top-down walkers) and detail level. One add-on trained on all of them blends those looks, so a creature comes out half-icon.
- **Why not one per category:** most categories are too small to train on alone (95 trainer sets), and the rest are derived, not drawn: back sprites share the Creatures add-on with a "back view" caption, menu icons are shrunk battle sprites, shinies are palette shifts.
- **What an add-on costs:** a file of tens of MB and roughly a day of CPU training each, so adding a fifth later (for example portraits) is cheap.
- **What should never be an image model:** anything with a precise layout or structure. Screens, fonts, 3D, maps and text all need exact results that a dedicated tool or an LLM editing text gives better.

Start with Items (simplest, fast feedback), then Creatures, then People. Walkers is last because the Walker Workbench works without it.

## Unknowns and sources

The list is complete at the category level; a few archives inside it still need identifying before their tool can target them.

- [ ] Which archive holds the building and prop models that map chunks place (#18)
- [ ] Which archives hold field effects and cutscene models (#21, #23); about 400 non-terrain 3D models are spread over \~40 archives
- [ ] Where the window frame and dialog box art lives (#12)
- [ ] How move animations map to their scripts and art (#22)
- [ ] What ARC 052 (a second Pokémon sprite set) is used for
- [ ] Decide whether the new lore needs dialog portraits, which would add a category, a screen feature and a fifth style add-on

Networking, Musicals, Dream World and online features are out of the Rust project's scope (plan items X1, X2), so their art isn't listed.

Sources: archive scan of `black-rom/files/a/` (234 archives, file types per member); `black-rust/crates/black-cli/src/cmd_extract_sprites/`; `black-rust/crates/nds-rom/tests/g3d_rom.rs` (battle backgrounds in ARC 011); `MIGRATION-AGENT.md` items A3–A12, G7–G8, U1–U22, H1, X1–X2 and the archive table in §7. Commit `ea8cae4`.
