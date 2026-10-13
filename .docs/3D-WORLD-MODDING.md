spencerlepine/black · Pokémon Black, checked 13 Oct 2026

# Modding the 3D world

What the Rust version unlocks for art, story and maps, how a Pokémon Black town is actually built, and the practical ways to make your own 3D pieces with an AI helping.

## Is your assessment right?

Mostly yes. One correction: the assets were always in the original too, packed inside the cartridge in Nintendo's own formats. ROM hackers edit them with special tools. The Rust project's extractor converts them once into normal files (PNG images, OGG audio, glTF 3D models, plain-text data), and the Rust game loads only those. That's the big unlock.

yes**Sprites and art.** Pokémon, trainers, overworld characters and item icons are already extracted to PNG. Some are cut into animation pieces, so a redraw means matching those pieces.

yes**Dialog.** All 56,700 text entries come out as editable files, with codes like "player's name" kept as tokens.

yes**Items, Pokémon stats, moves.** Exported as plain-text tables (RON, a JSON-like format).

mostly**Story line.** Story is driven by map scripts ("when you talk to this NPC, play this text, set this flag, start this battle"). The script engine is ported, and the 449 script commands are being ported now. Rewriting the story means editing scripts and events, so it's doable but more than editing text.

not yet**Walking around the 3D world.** The 3D models and map data are extracted, but the Rust game can't draw the overworld yet. That step (G8 in the plan) hasn't started, and the plan still lists "3D or 2D overworld?" as an open choice, defaulting to 3D. If 3D modding matters to you, that's a decision worth weighing in on.

## How a town is built

The DS world isn't one big 3D scene. It's a grid of square map chunks, like floor tiles. The main overworld grid is 29 by 27 chunks. Each chunk is a small bundle the project calls a *container*, and there are 572 of them. Each bundle has three parts, plus a separate list of who and what lives there.

This matters for your plan: only part 1 and the building models are actually 3D art. Everything else, where you can walk, where doors go, who stands where, is data an AI can edit directly like a spreadsheet. If you change the terrain, the walkability grid has to change to match, or the player will walk through your new trees.

Also worth knowing: these models are tiny by modern standards. The DS can only draw a few thousand triangles at once, with very small textures. A Pokémon Black tree is a handful of flat shapes with a picture on them. That makes them much easier to make or generate than modern game assets.

## What glTF is, and why it helps

glTF is the standard file format for 3D models, a bit like PNG is for images. It's a JSON file that lists points in space, the triangles connecting them, and which picture goes on each surface. Blender opens and saves it, web viewers show it, and Bevy (the engine the Rust game uses) loads it natively.

The project already converts all 572 terrain models to glTF, and three of them were checked by hand in a 3D viewer on 12 Oct. What doesn't exist yet is the other direction for map data that the game reads: a tool that takes your edited map and produces the new chunk files. The Rust readers can already write those files back out byte for byte, so it's a short step to build.

## Three ways to make 3D pieces with AI

Your Blender idea works. It's not the only option, and for some jobs it's not the simplest.

### A. Edit what's there

best for: tweaking towns you already know

Open an extracted map chunk in Blender, move a building, add trees by copying existing ones, change a texture in any image editor. No AI needed for the 3D part. The AI then updates the walkability grid and events to match.

### B. AI writes a build script

best for: new objects in the same simple style

This is your idea, and it's a good one. The AI writes Python that tells Blender "make a cone here, a cylinder there, color them," and Blender runs it with no window open and saves glTF. Blender can also render a preview picture, which the AI looks at to catch mistakes. You never have to learn Blender's interface.

### C. Start from a ready-made model

best for: things that need to look detailed

Grab a free low-poly model or generate one with a text-to-3D tool, then have the AI shrink it down to DS-like detail and give it the right scale. Fastest for complex shapes, but matching the Pokémon Black look takes some care.

### "Just give the AI the code and assets?"

your simpler idea

Partly. An AI can write simple 3D geometry straight into a glTF file or into Rust code, no Blender involved. For boxes, fences and basic trees that works. The catch is that it can't see what it made, and 3D mistakes (inside-out faces, wrong scale, floating objects) are invisible in the numbers. Whatever route you pick, the key piece is a step that renders a picture so the AI, and you, can check it.

## A practical first project

Once the overworld renderer exists, a good starter mod is "add a new house to Nuvema Town" (your hometown in the game):

- **Model.** The AI writes a Blender script for a small house in the same blocky style, renders a preview, you approve it, and it saves a .gltf.
- **Place it.** The AI adds a building record for that chunk: which model and where.
- **Make it solid.** The AI marks those squares as blocked in the walkability grid, and the door square as a door.
- **Make it enterable.** A warp event on the door square, an interior chunk (copy an existing house interior), and an NPC with a line of dialog inside.

Only the first step is 3D art. The other three are small data edits, which is exactly the kind of thing an AI does well.

**Where things stand today.** Art, text and data extraction are done. The game can't show the overworld yet, a few walkability values are still not fully decoded, and the map-to-file write-back tool doesn't exist yet. So 3D world modding is a few milestones away, while sprite, dialog and stats mods line up sooner.

Sources: `MIGRATION-AGENT.md` items A3, A5, A6, A10, A11, A12, G8 and the open decision list; `black-rust/crates/nds-rom/src/map/container.rs` (chunk layout, walkability cells marked UNVERIFIED). Commit `ea8cae4`. The routes and the starter project are my suggestions, not part of the repo's plan.