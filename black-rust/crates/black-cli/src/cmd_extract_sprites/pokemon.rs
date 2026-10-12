//! Pokemon battle sprites (ARC 004) and box icons (ARC 007) -> PNG atlases + `.atlas.ron`.
//!
//! ARC 004 layout (verified on the real ROM, 714 blocks of 20 members + 5 trailing members):
//! each block holds two sprite groups of 9 members, then two palettes.
//!
//! | member (in group) | content |
//! |---|---|
//! | 0 | tiled NCGR (96x96, 4bpp) - a flattened still, not used by the cells |
//! | 1 | same, alternate (female) form; empty when there is no difference |
//! | 2 | scanned NCGR sheet (256x128, 4bpp): the tiles the cells point at (2D mapping, 32 tiles/row) |
//! | 3 | scanned sheet of the alternate (female) form; empty when absent |
//! | 4 | NCER cell bank |
//! | 5 | NANR animation bank |
//! | 6 | RCMN multi-cell bank (raw, UNVERIFIED) |
//! | 7 | RAMN multi-cell animations (raw, UNVERIFIED) |
//! | 8 | per-cell side table: `u32 count` + records (raw, UNVERIFIED) |
//!
//! Block members 18 / 19 are the normal / shiny NCLR.

use nds_rom::lz;
use nds_rom::nanr::Nanr;
use nds_rom::narc::Narc;
use nds_rom::ncer::Ncer;
use nds_rom::ncgr::Ncgr;
use nds_rom::nclr::Nclr;
use nds_rom::render::{self, Image};
use std::fmt::Write as _;

pub const BLOCK: usize = 20;
const GROUP: usize = 9;
/// Species blocks of 20 members; the members after them hold two special sets and palettes.
pub const SPECIES_BLOCKS: usize = 712;
const ATLAS_WIDTH: usize = 256;

/// Which sprite group of a block: group 0 holds the many-cell, many-animation set (front),
/// group 1 the small set (back). See `Orientation::label`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Orientation {
    Front,
    Back,
}

impl Orientation {
    pub fn label(self) -> &'static str {
        match self {
            Orientation::Front => "front",
            Orientation::Back => "back",
        }
    }
    fn group(self) -> usize {
        match self {
            Orientation::Front => 0,
            Orientation::Back => 1,
        }
    }
}

/// One frame (an assembled cell) inside an atlas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    /// Position of the frame's top-left pixel relative to the sprite anchor.
    pub origin: (i32, i32),
}

/// One animation frame with the optional transform of NANR element types 1 (SRT) and 2 (T).
#[derive(Clone, Debug, PartialEq)]
pub struct AnimStep {
    pub cell: u16,
    pub duration: u16,
    /// Rotation in degrees (0 when the element type has none).
    pub rot: f32,
    pub scale: (f32, f32),
    pub trans: (i16, i16),
}

/// One node of a multi-cell (RCMN): a cell animation (NANR sequence index) placed at (x, y).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub anim: u16,
    pub x: i16,
    pub y: i16,
    /// Last u16 of the record: low byte constant 0x20, high byte = node index (UNVERIFIED role).
    pub flags: u16,
}

/// A multi-cell: the full sprite assembled from nodes, pre-composed into `frame`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Multicell {
    pub nodes: Vec<Node>,
    /// Index in `Sprite::frames` of this multi-cell composed from each node's first animation step.
    pub frame: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Seq {
    pub kind: u16,
    pub mode: u32,
    pub steps: Vec<AnimStep>,
}

/// A converted sprite set: one orientation, one gender slot, both palettes.
pub struct Sprite {
    /// File stem, e.g. `0001_front` or `0521_front_f`.
    pub name: String,
    pub block: usize,
    pub orientation: Orientation,
    pub female: bool,
    pub normal: Image,
    pub shiny: Image,
    pub frames: Vec<FrameRect>,
    /// Number of plain cell frames; `frames[cells..]` are the composed multi-cells.
    pub cells: usize,
    /// NANR cell animations (cell field = cell index).
    pub seqs: Vec<Seq>,
    pub multicells: Vec<Multicell>,
    /// RAMN multi-cell animations (cell field = multi-cell index).
    pub multi_seqs: Vec<Seq>,
    pub ron: String,
    /// Raw members kept verbatim: (suffix, bytes).
    pub raw: Vec<(&'static str, Vec<u8>)>,
}

impl Sprite {
    /// Cut frame `i` out of an atlas image.
    pub fn describe(&self) -> String {
        format!(
            "{} (block {}, {} cells, {} animations, {} multicells)",
            self.name,
            self.block,
            self.cells,
            self.seqs.len(),
            self.multicells.len()
        )
    }
    pub fn frame_image(&self, atlas: &Image, i: usize) -> Image {
        crop(atlas, &self.frames[i])
    }
    /// Index of the first frame played by animation 0 (the "frame 0" used by goldens).
    pub fn first_frame(&self) -> usize {
        let mc = self
            .multi_seqs
            .first()
            .and_then(|s| s.steps.first())
            .map_or(0, |s| s.cell as usize);
        self.multicells
            .get(mc)
            .or(self.multicells.first())
            .map_or(0, |m| m.frame)
    }
}

pub fn crop(atlas: &Image, r: &FrameRect) -> Image {
    let mut out = Image::new(r.w, r.h);
    for y in 0..r.h {
        let s = ((r.y + y) * atlas.width + r.x) * 4;
        out.rgba[y * r.w * 4..(y + 1) * r.w * 4].copy_from_slice(&atlas.rgba[s..s + r.w * 4]);
    }
    out
}

/// SHA-1 over `w:u32le h:u32le rgba`.
#[cfg(test)]
pub fn image_hash(img: &Image) -> String {
    let mut v = (img.width as u32).to_le_bytes().to_vec();
    v.extend_from_slice(&(img.height as u32).to_le_bytes());
    v.extend_from_slice(&img.rgba);
    nds_rom::sha1_hex(&v)
}

pub(super) fn member(narc: &Narc, i: usize) -> Vec<u8> {
    let f = narc.file(i);
    lz::decompress_any(f).unwrap_or_else(|| f.to_vec())
}

pub(super) type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Decode the per-frame element data of a NANR, including the transforms of element types 1/2
/// that `nds_rom::nanr` does not expose. Returns `None` if the layout is unexpected.
fn nanr_steps(data: &[u8]) -> Option<Vec<Seq>> {
    let b = data.get(24..)?;
    let u16at = |o: usize| b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let u32at = |o: usize| {
        b.get(o..o + 4)
            .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    };
    let n = u16at(0)? as usize;
    let seq_off = u32at(4)? as usize;
    let frame_off = u32at(8)? as usize;
    let data_off = u32at(12)? as usize;
    let mut out = Vec::new();
    for i in 0..n {
        let s = seq_off + i * 16;
        let count = u16at(s)? as usize;
        let kind = u16at(s + 4)?;
        let mode = u32at(s + 8)?;
        let first = u32at(s + 12)? as usize;
        let mut steps = Vec::new();
        for j in 0..count {
            let f = frame_off + first + j * 8;
            let d = data_off + u32at(f)? as usize;
            let duration = u16at(f + 4)?;
            let cell = u16at(d)?;
            let fx = |o: usize| u32at(o).map(|v| v as i32 as f32 / 4096.0);
            let (rot, scale, trans) = match kind {
                1 => (
                    u16at(d + 2)? as f32 * 360.0 / 65536.0,
                    (fx(d + 4)?, fx(d + 8)?),
                    (u16at(d + 12)? as i16, u16at(d + 14)? as i16),
                ),
                2 => (
                    0.0,
                    (1.0, 1.0),
                    (u16at(d + 4)? as i16, u16at(d + 6)? as i16),
                ),
                _ => (0.0, (1.0, 1.0), (0, 0)),
            };
            steps.push(AnimStep {
                cell,
                duration,
                rot,
                scale,
                trans,
            });
        }
        out.push(Seq { kind, mode, steps });
    }
    Some(out)
}

/// Decode an RCMN multi-cell bank: `u16 count, 0xBEEF, u32 table_off, u32 node_off`, then
/// per multi-cell `u16 nodes, u16 anims, u32 first_node_byte_off` and 8-byte nodes
/// `u16 anim, s16 x, s16 y, u16 flags`. Layout inferred from the ROM's data; node semantics
/// UNVERIFIED. Returns `None` on any inconsistency.
fn parse_rcmn(data: &[u8]) -> Option<Vec<Vec<Node>>> {
    let b = data.get(24..)?;
    let u16at = |o: usize| b.get(o..o + 2).map(|s| u16::from_le_bytes([s[0], s[1]]));
    let u32at = |o: usize| {
        b.get(o..o + 4)
            .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    };
    let n = u16at(0)? as usize;
    if u16at(2)? != 0xBEEF {
        return None;
    }
    let table = u32at(4)? as usize;
    let nodes = u32at(8)? as usize;
    let mut out = Vec::new();
    for i in 0..n {
        let e = table + i * 8;
        let count = u16at(e)? as usize;
        let first = u32at(e + 4)? as usize;
        let mut v = Vec::new();
        for j in 0..count {
            let o = nodes + first + j * 8;
            v.push(Node {
                anim: u16at(o)?,
                x: u16at(o + 2)? as i16,
                y: u16at(o + 4)? as i16,
                flags: u16at(o + 6)?,
            });
        }
        out.push(v);
    }
    Some(out)
}

/// Compose a multi-cell from per-cell renders, using the first step of each node's animation.
/// Nodes are drawn last-to-first (node 0 on top): UNVERIFIED draw order.
fn compose(nodes: &[Node], seqs: &[Seq], cells: &[(Image, (i32, i32))]) -> (Image, (i32, i32)) {
    let mut parts = Vec::new();
    for n in nodes.iter().rev() {
        let Some(step) = seqs.get(n.anim as usize).and_then(|q| q.steps.first()) else {
            continue;
        };
        let Some((img, o)) = cells.get(step.cell as usize) else {
            continue;
        };
        let px = n.x as i32 + step.trans.0 as i32 + o.0;
        let py = n.y as i32 + step.trans.1 as i32 + o.1;
        parts.push((img, px, py));
    }
    if parts.is_empty() {
        return (Image::new(1, 1), (0, 0));
    }
    let x0 = parts.iter().map(|p| p.1).min().unwrap_or(0);
    let y0 = parts.iter().map(|p| p.2).min().unwrap_or(0);
    let x1 = parts
        .iter()
        .map(|p| p.1 + p.0.width as i32)
        .max()
        .unwrap_or(1);
    let y1 = parts
        .iter()
        .map(|p| p.2 + p.0.height as i32)
        .max()
        .unwrap_or(1);
    let mut out = Image::new((x1 - x0) as usize, (y1 - y0) as usize);
    for (img, px, py) in parts {
        for y in 0..img.height {
            for x in 0..img.width {
                let p = img.pixel(x, y);
                if p[3] != 0 {
                    let i = (((py - y0) as usize + y) * out.width + (px - x0) as usize + x) * 4;
                    out.rgba[i..i + 4].copy_from_slice(&p);
                }
            }
        }
    }
    (out, (x0, y0))
}

/// Shelf-pack frame images into an atlas of fixed width; returns rects in input order.
fn pack(frames: &[(Image, (i32, i32))]) -> (Vec<FrameRect>, usize, usize) {
    let width = frames
        .iter()
        .map(|f| f.0.width)
        .max()
        .unwrap_or(1)
        .max(ATLAS_WIDTH);
    let (mut x, mut y, mut row_h) = (0usize, 0usize, 0usize);
    let mut rects = Vec::new();
    for (img, origin) in frames {
        if x + img.width > width {
            x = 0;
            y += row_h + 1;
            row_h = 0;
        }
        rects.push(FrameRect {
            x,
            y,
            w: img.width,
            h: img.height,
            origin: *origin,
        });
        x += img.width + 1;
        row_h = row_h.max(img.height);
    }
    (rects, width, y + row_h)
}

fn blit(dst: &mut Image, src: &Image, r: &FrameRect) {
    for y in 0..r.h {
        let d = ((r.y + y) * dst.width + r.x) * 4;
        dst.rgba[d..d + r.w * 4].copy_from_slice(&src.rgba[y * r.w * 4..(y + 1) * r.w * 4]);
    }
}

fn build_atlas(frames: &[(Image, (i32, i32))]) -> (Image, Vec<FrameRect>) {
    let (rects, w, h) = pack(frames);
    let mut img = Image::new(w, h.max(1));
    for ((src, _), r) in frames.iter().zip(&rects) {
        blit(&mut img, src, r);
    }
    (img, rects)
}

fn render_cells(ncer: &Ncer, sheet: &Ncgr, pal: &Nclr) -> Res<Vec<(Image, (i32, i32))>> {
    (0..ncer.cells.len())
        .map(|i| {
            render::render_cell(ncer, i, sheet, pal)
                .ok_or_else(|| format!("cell {i} out of range").into())
        })
        .collect()
}

struct Anim<'a> {
    cells: usize,
    seqs: &'a [Seq],
    multicells: &'a [Multicell],
    multi_seqs: &'a [Seq],
}

fn write_ron(name: &str, source: &str, atlas: &Image, rects: &[FrameRect], anim: &Anim) -> String {
    let Anim {
        cells,
        seqs,
        multicells,
        multi_seqs,
    } = *anim;
    let images = [
        ("normal", format!("{name}.png")),
        ("shiny", format!("{name}_shiny.png")),
    ];

    let mut s = String::new();
    let _ = writeln!(s, "(");
    let _ = writeln!(s, "    source: \"{source}\",");
    let _ = writeln!(s, "    name: \"{name}\",");
    let _ = writeln!(s, "    size: ({}, {}),", atlas.width, atlas.height);
    let _ = writeln!(s, "    images: (");
    for (k, v) in images {
        let _ = writeln!(s, "        {k}: \"{v}\",");
    }
    let _ = writeln!(s, "    ),");
    let _ = writeln!(
        s,
        "    // origin = position of the frame's top-left pixel relative to the sprite anchor"
    );
    let _ = writeln!(s, "    frames: [");
    for r in rects {
        let _ = writeln!(
            s,
            "        (rect: ({}, {}, {}, {}), origin: ({}, {})),",
            r.x, r.y, r.w, r.h, r.origin.0, r.origin.1
        );
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(
        s,
        "    // frames[..cells] are single cells; frames[cells..] are composed multi-cells"
    );
    let _ = writeln!(s, "    cells: {cells},");
    let _ = writeln!(s, "    // durations are in 1/60 s video frames; mode 1 forward, 2 forward loop, 3 reverse, 4 reverse loop");
    let _ = writeln!(
        s,
        "    // sequences: NANR cell animations (frame = cell index); node anim indexes into them"
    );
    write_seqs(&mut s, "sequences", seqs);
    let _ = writeln!(s, "    // multicells (RCMN, layout UNVERIFIED): the full sprite; `frame` = composed from each node's first step");
    let _ = writeln!(s, "    multicells: [");
    for m in multicells {
        let nodes = m
            .nodes
            .iter()
            .map(|n| {
                format!(
                    "(anim: {}, x: {}, y: {}, flags: {})",
                    n.anim, n.x, n.y, n.flags
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(s, "        (frame: {}, nodes: [{nodes}]),", m.frame);
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(
        s,
        "    // multi_sequences: RAMN multi-cell animations (frame = multicell index)"
    );
    write_seqs(&mut s, "multi_sequences", multi_seqs);
    let _ = writeln!(s, ")");
    s
}

fn write_seqs(s: &mut String, key: &str, seqs: &[Seq]) {
    let _ = writeln!(s, "    {key}: [");
    for q in seqs {
        let _ = writeln!(s, "        (kind: {}, mode: {}, steps: [", q.kind, q.mode);
        for t in &q.steps {
            let _ = writeln!(
                s,
                "            (frame: {}, duration: {}, rot: {:.4}, scale: ({:.4}, {:.4}), trans: ({}, {})),",
                t.cell, t.duration, t.rot, t.scale.0, t.scale.1, t.trans.0, t.trans.1
            );
        }
        let _ = writeln!(s, "        ]),");
    }
    let _ = writeln!(s, "    ],");
}

/// Raw members of one sprite set.
pub(super) struct Parts<'a> {
    pub(super) name: String,
    pub(super) source: String,
    pub(super) block: usize,
    pub(super) orientation: Orientation,
    pub(super) female: bool,
    pub(super) sheet: &'a [u8],
    pub(super) ncer: &'a [u8],
    pub(super) nanr: &'a [u8],
    pub(super) rcmn: &'a [u8],
    pub(super) ramn: &'a [u8],
    pub(super) cellinfo: &'a [u8],
    pub(super) pal_n: &'a Nclr,
    pub(super) pal_s: &'a Nclr,
}

pub(super) fn build_sprite(p: &Parts) -> Res<Sprite> {
    let ncer = Ncer::parse(p.ncer)?;
    let seqs = match nanr_steps(p.nanr) {
        Some(s) => s,
        None => Nanr::parse(p.nanr)?
            .sequences
            .iter()
            .map(|q| Seq {
                kind: q.kind,
                mode: q.mode,
                steps: q
                    .frames
                    .iter()
                    .map(|f| AnimStep {
                        cell: f.cell,
                        duration: f.duration,
                        rot: 0.0,
                        scale: (1.0, 1.0),
                        trans: (0, 0),
                    })
                    .collect(),
            })
            .collect(),
    };
    let sheet = Ncgr::parse(p.sheet)?;
    let mut normal = render_cells(&ncer, &sheet, p.pal_n)?;
    let mut shiny = render_cells(&ncer, &sheet, p.pal_s)?;
    let cells = normal.len();
    let mut multicells = Vec::new();
    if let Some(mcs) = parse_rcmn(p.rcmn) {
        for nodes in mcs {
            let frame = normal.len();
            let n = compose(&nodes, &seqs, &normal[..cells]);
            normal.push(n);
            let sh = compose(&nodes, &seqs, &shiny[..cells]);
            shiny.push(sh);
            multicells.push(Multicell { nodes, frame });
        }
    }
    let multi_seqs = nanr_steps(p.ramn).unwrap_or_default();
    let (atlas_n, rects) = build_atlas(&normal);
    let mut atlas_s = Image::new(atlas_n.width, atlas_n.height);
    for ((src, _), r) in shiny.iter().zip(&rects) {
        blit(&mut atlas_s, src, r);
    }
    let ron = write_ron(
        &p.name,
        &p.source,
        &atlas_n,
        &rects,
        &Anim {
            cells,
            seqs: &seqs,
            multicells: &multicells,
            multi_seqs: &multi_seqs,
        },
    );
    let mut raw = Vec::new();
    if !p.female {
        raw.push(("rcmn", p.rcmn.to_vec()));
        raw.push(("ramn", p.ramn.to_vec()));
        raw.push(("cellinfo", p.cellinfo.to_vec()));
    }
    Ok(Sprite {
        name: p.name.clone(),
        block: p.block,
        orientation: p.orientation,
        female: p.female,
        normal: atlas_n,
        shiny: atlas_s,
        frames: rects,
        cells,
        seqs,
        multicells,
        multi_seqs,
        ron,
        raw,
    })
}

/// Convert one block of ARC 004. Returns one `Sprite` per (orientation, gender slot) present.
pub fn convert_block(narc: &Narc, block: usize) -> Res<Vec<Sprite>> {
    let base = block * BLOCK;
    if base + BLOCK > narc.len() || block >= SPECIES_BLOCKS {
        return Err("block out of range".into());
    }
    let m = |i: usize| member(narc, base + i);
    let pal_n = Nclr::parse(&m(18))?;
    let pal_s = Nclr::parse(&m(19))?;
    let mut out = Vec::new();
    for o in [Orientation::Front, Orientation::Back] {
        let g = o.group() * GROUP;
        let (ncer, nanr, rcmn, ramn, cellinfo) = (m(g + 4), m(g + 5), m(g + 6), m(g + 7), m(g + 8));
        for female in [false, true] {
            let sheet = m(g + 2 + female as usize);
            if sheet.is_empty() {
                continue;
            }
            out.push(build_sprite(&Parts {
                name: format!("{block:04}_{}{}", o.label(), if female { "_f" } else { "" }),
                source: format!("ARC 004 block {block} group {}", o.group()),
                block,
                orientation: o,
                female,
                sheet: &sheet,
                ncer: &ncer,
                nanr: &nanr,
                rcmn: &rcmn,
                ramn: &ramn,
                cellinfo: &cellinfo,
                pal_n: &pal_n,
                pal_s: &pal_s,
            })?);
        }
    }
    Ok(out)
}

/// The two special sets after the species blocks (members 14240..14252): each is
/// `[tiled NCGR, NCER, NANR, RCMN, RAMN, side table]`, followed by 33 palettes. Their identity
/// (egg-like) and palette assignment are UNVERIFIED; palette 0 is used for both.
pub fn convert_specials(narc: &Narc) -> Res<Vec<Sprite>> {
    let base = SPECIES_BLOCKS * BLOCK;
    let pal = Nclr::parse(&member(narc, base + 12))?;
    let mut out = Vec::new();
    for k in 0..2 {
        let g = base + k * 6;
        let m = |i: usize| member(narc, g + i);
        let (sheet, ncer, nanr, rcmn, ramn, cellinfo) = (m(0), m(1), m(2), m(3), m(4), m(5));
        out.push(build_sprite(&Parts {
            name: format!("special_{k}"),
            source: format!("ARC 004 members {g}..{}", g + 5),
            block: SPECIES_BLOCKS + k,
            orientation: Orientation::Front,
            female: false,
            sheet: &sheet,
            ncer: &ncer,
            nanr: &nanr,
            rcmn: &rcmn,
            ramn: &ramn,
            cellinfo: &cellinfo,
            pal_n: &pal,
            pal_s: &pal,
        })?);
    }
    Ok(out)
}

pub struct Report {
    pub blocks: usize,
    pub sprites: usize,
    pub failures: Vec<(usize, String)>,
    pub icons: usize,
    pub icon_frames: usize,
}

/// Number of species blocks in ARC 004 (the trailing members past the last whole block are ignored).
pub fn block_count(narc: &Narc) -> usize {
    (narc.len() / BLOCK).min(SPECIES_BLOCKS)
}

pub struct IconSet {
    /// One atlas per palette row (the ROM stores 3 rows; the per-icon row is UNVERIFIED).
    pub atlases: Vec<Image>,
    pub ron: String,
    pub icons: usize,
    pub frames: usize,
}

/// Convert ARC 007: member 0 palette (3 rows), 1..6 NANR/NCER pairs, then `(NCGR, alt NCGR)`
/// pairs from member 7.
pub fn convert_icons(narc: &Narc) -> Res<IconSet> {
    let pal = Nclr::parse(&member(narc, 0))?;
    let ncer = Ncer::parse(&member(narc, 2))?;
    let rows = (pal.colors.len() / 16).max(1);
    let pairs = (narc.len() - 7) / 2;
    // (icon, alt, ncgr)
    let mut icons = Vec::new();
    for k in 0..pairs {
        for alt in [false, true] {
            let raw = member(narc, 7 + k * 2 + alt as usize);
            if raw.is_empty() {
                continue;
            }
            icons.push((k, alt, Ncgr::parse(&raw)?));
        }
    }
    let mut atlases = Vec::new();
    let mut rects: Vec<FrameRect> = Vec::new();
    for row in 0..rows {
        let mut frames = Vec::new();
        for (_, _, ncgr) in &icons {
            for c in 0..ncer.cells.len() {
                // draw with palette row `row` regardless of the OAM palette field
                let mut cell_bank = ncer.clone();
                for o in &mut cell_bank.cells[c].oams {
                    o.palette = row as u8;
                }
                frames.push(
                    render::render_cell(&cell_bank, c, ncgr, &pal)
                        .ok_or("icon cell out of range")?,
                );
            }
        }
        let (img, r) = build_atlas(&frames);
        atlases.push(img);
        rects = r;
    }
    let per = ncer.cells.len();
    let mut s = String::new();
    let _ = writeln!(s, "(");
    let _ = writeln!(s, "    source: \"ARC 007\",");
    let _ = writeln!(
        s,
        "    size: ({}, {}),",
        atlases[0].width, atlases[0].height
    );
    let _ = writeln!(s, "    // one image per palette row; which row an icon uses is UNVERIFIED (table lives in code)");
    let _ = writeln!(
        s,
        "    images: [{}],",
        (0..rows)
            .map(|r| format!("\"icons_pal{r}.png\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(s, "    frames_per_icon: {per},");
    let _ = writeln!(s, "    // icon = index of the (NCGR, alt NCGR) pair = species block; alt = second member of the pair");
    let _ = writeln!(s, "    icons: [");
    for (n, (k, alt, _)) in icons.iter().enumerate() {
        let _ = writeln!(
            s,
            "        (icon: {k}, alt: {alt}, first_frame: {}),",
            n * per
        );
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(s, "    frames: [");
    for r in &rects {
        let _ = writeln!(
            s,
            "        (rect: ({}, {}, {}, {}), origin: ({}, {})),",
            r.x, r.y, r.w, r.h, r.origin.0, r.origin.1
        );
    }
    let _ = writeln!(s, "    ],");
    // sequences of the first animation bank (member 1), as shipped
    let _ = writeln!(s, "    sequences: [");
    if let Some(q) = nanr_steps(&member(narc, 1)) {
        for q in q {
            let _ = writeln!(
                s,
                "        (kind: {}, mode: {}, steps: [{}]),",
                q.kind,
                q.mode,
                q.steps
                    .iter()
                    .map(|t| format!("({}, {})", t.cell, t.duration))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    let _ = writeln!(s, "    ],");
    let _ = writeln!(s, ")");
    Ok(IconSet {
        icons: icons.len(),
        frames: rects.len(),
        atlases,
        ron: s,
    })
}

/// Convert everything and write it under `<out>/sprites/pokemon/`.
pub fn extract(files_dir: &std::path::Path, out: &std::path::Path) -> Res<Report> {
    let dir = out.join("sprites/pokemon");
    std::fs::create_dir_all(&dir)?;
    let a4 = Narc::parse(&std::fs::read(files_dir.join("a/0/0/4"))?)?;
    let mut rep = Report {
        blocks: block_count(&a4),
        sprites: 0,
        failures: Vec::new(),
        icons: 0,
        icon_frames: 0,
    };
    for b in 0..rep.blocks {
        match convert_block(&a4, b) {
            Ok(list) => {
                for s in list {
                    std::fs::write(dir.join(format!("{}.png", s.name)), s.normal.to_png()?)?;
                    std::fs::write(dir.join(format!("{}_shiny.png", s.name)), s.shiny.to_png()?)?;
                    std::fs::write(dir.join(format!("{}.atlas.ron", s.name)), &s.ron)?;
                    for (ext, bytes) in &s.raw {
                        std::fs::write(dir.join(format!("{}.{ext}.bin", s.name)), bytes)?;
                    }
                    rep.sprites += 1;
                }
            }
            Err(e) => rep.failures.push((b, e.to_string())),
        }
    }
    match convert_specials(&a4) {
        Ok(list) => {
            for s in list {
                std::fs::write(dir.join(format!("{}.png", s.name)), s.normal.to_png()?)?;
                std::fs::write(dir.join(format!("{}_shiny.png", s.name)), s.shiny.to_png()?)?;
                std::fs::write(dir.join(format!("{}.atlas.ron", s.name)), &s.ron)?;
                rep.sprites += 1;
                println!("  special: {}", s.describe());
            }
        }
        Err(e) => rep.failures.push((SPECIES_BLOCKS, e.to_string())),
    }
    let a7 = Narc::parse(&std::fs::read(files_dir.join("a/0/0/7"))?)?;
    let icons = convert_icons(&a7)?;
    for (i, img) in icons.atlases.iter().enumerate() {
        std::fs::write(dir.join(format!("icons_pal{i}.png")), img.to_png()?)?;
    }
    std::fs::write(dir.join("icons.atlas.ron"), &icons.ron)?;
    rep.icons = icons.icons;
    rep.icon_frames = icons.frames;
    Ok(rep)
}

/// Contact sheet: frame 0 (animation 0, first step) of normal then shiny, per block, in a grid.
pub fn contact_sheet(narc: &Narc, blocks: &[usize], orientation: Orientation) -> Res<Image> {
    let cell = 100usize;
    let cols = 5usize;
    let rows = (blocks.len() * 2).div_ceil(cols);
    let mut sheet = Image::new(cols * cell, rows * cell);
    for (n, &b) in blocks.iter().enumerate() {
        let list = convert_block(narc, b)?;
        let Some(s) = list
            .iter()
            .find(|s| s.orientation == orientation && !s.female)
        else {
            continue;
        };
        for (k, atlas) in [&s.normal, &s.shiny].into_iter().enumerate() {
            let f = s.frame_image(atlas, s.first_frame());
            let (ox, oy) = s.frames[s.first_frame()].origin;
            let slot = n * 2 + k;
            let (cx, cy) = (
                (slot % cols) * cell + cell / 2,
                (slot / cols) * cell + cell / 2 + 20,
            );
            for y in 0..f.height {
                for x in 0..f.width {
                    let (dx, dy) = (cx as i32 + ox + x as i32, cy as i32 + oy + y as i32);
                    if dx < 0 || dy < 0 || dx as usize >= sheet.width || dy as usize >= sheet.height
                    {
                        continue;
                    }
                    let p = f.pixel(x, y);
                    if p[3] != 0 {
                        let i = (dy as usize * sheet.width + dx as usize) * 4;
                        sheet.rgba[i..i + 4].copy_from_slice(&p);
                    }
                }
            }
        }
    }
    Ok(sheet)
}

#[cfg(test)]
const EXPECTED_SETS: usize = 1616;
#[cfg(test)]
const EXPECTED_FEMALE_SETS: usize = 192;
#[cfg(test)]
const GOLDEN_NORMAL: &str = "191d4ab8dd1e35591fec64a6b39ff518a0c0a341";
#[cfg(test)]
const GOLDEN_SHINY: &str = "a62482fdfeced3ef1e6e035a54f3cd0acee4dbbf";

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn files_dir() -> PathBuf {
        std::env::var_os("BLACK_ROM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"))
            .join("files")
    }

    fn load(id: u32) -> Option<Narc> {
        let p = files_dir().join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
        Narc::parse(&std::fs::read(p).ok()?).ok()
    }

    #[test]
    fn converts_every_species_block() {
        let Some(a4) = load(4) else { return };
        let mut sets = 0;
        let mut female = 0;
        for b in 0..block_count(&a4) {
            let list = convert_block(&a4, b).unwrap_or_else(|e| panic!("block {b}: {e}"));
            assert!(list.len() >= 2, "block {b} needs front and back");
            for s in &list {
                assert_eq!(
                    (s.normal.width, s.normal.height),
                    (s.shiny.width, s.shiny.height)
                );
                assert!(s.cells > 0 && s.frames.len() > s.cells, "block {b} frames");
                assert!(s.frame_image(&s.normal, s.first_frame()).opaque_pixels() > 0);
                assert!(!s.ron.is_empty() && !s.seqs.is_empty());
                female += s.female as usize;
            }
            sets += list.len();
        }
        assert_eq!(block_count(&a4), 712);
        println!("COUNTS {sets} {female}");
        assert_eq!(sets, EXPECTED_SETS);
        assert_eq!(female, EXPECTED_FEMALE_SETS);
        assert_eq!(convert_specials(&a4).unwrap().len(), 2);
    }

    #[test]
    fn bulbasaur_golden() {
        let Some(a4) = load(4) else { return };
        let list = convert_block(&a4, 1).unwrap();
        let front = list.iter().find(|s| s.name == "0001_front").unwrap();
        assert_eq!(front.block, 1);
        let f = front.first_frame();
        let n = image_hash(&front.frame_image(&front.normal, f));
        let sh = image_hash(&front.frame_image(&front.shiny, f));
        println!("GOLDEN {f} {n} {sh} {}", front.cells);
        assert_eq!(n, GOLDEN_NORMAL);
        assert_eq!(sh, GOLDEN_SHINY);
        assert_ne!(n, sh);
    }

    #[test]
    fn icon_atlas_counts() {
        let Some(a7) = load(7) else { return };
        let set = convert_icons(&a7).unwrap();
        assert_eq!(set.atlases.len(), 3);
        assert_eq!(set.icons, 715);
        assert_eq!(set.frames, set.icons * 3);
        assert!(set.ron.contains("frames_per_icon: 3"));
    }
}
