//! `black extract-assets <black-rom-root> <out>` (A11): run every converter and write the
//! D4 `assets/` layout plus `manifest.json` (path, size, sha1 of every file).
//!
//! Idempotency: every converter is deterministic, so a re-run simply overwrites each output
//! with identical bytes (the manifest is byte-identical too). Nothing is skipped.

use black_data::extract::{extract_data, RomSource};
use clap::Args as ClapArgs;
use nds_rom::g3d::{gltf, model::Bmd0, tex::Tex0};
use nds_rom::map::{
    MapContainer, MapMatrix, ZoneEvents, ZoneHeader, ARC_MAP_CONTAINER, ARC_MAP_MATRIX,
    ARC_ZONE_EVENTS, ARC_ZONE_HEADERS,
};
use nds_rom::msg::Bank;
use nds_rom::narc::Narc;
use nds_rom::sdat::render::{RenderOptions, Renderer};
use nds_rom::sdat::{Kind, Sdat};
use nds_rom::{gffont, sha1_hex};
use std::collections::BTreeMap;
use std::fmt::{Display, Write as _};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

type Res<T> = Result<T, String>;

fn es<E: Display>(e: E) -> String {
    e.to_string()
}

#[derive(ClapArgs)]
pub struct Args {
    /// The black-rom directory (contains files/ and build/black.us/)
    pub rom_root: PathBuf,
    /// Output directory (normally black-rust/assets, which is gitignored)
    pub out: PathBuf,
    /// Only these categories (comma separated or repeated): data, text, fonts, sprites
    /// (= sprites-pokemon,sprites-trainers,sprites-overworld,sprites-items,sprites-icons,
    /// sprites-battle-ui), maps, models, audio (= bgm,se)
    #[arg(long, value_delimiter = ',')]
    pub only: Vec<String>,
    /// Worker threads (default: available parallelism)
    #[arg(long)]
    pub jobs: Option<usize>,
    /// Directory holding OVY_93.sbin and main.sbin (default <rom-root>/build/black.us,
    /// or $BLACK_ROM_BUILD)
    #[arg(long)]
    pub build_dir: Option<PathBuf>,
    /// Keep audio as WAV even when an OGG encoder is available
    #[arg(long)]
    pub wav: bool,
}

/// `(category, output dir relative to <out>)`.
const CATEGORIES: &[(&str, &str)] = &[
    ("data", "data"),
    ("text", "text"),
    ("fonts", "fonts"),
    ("sprites-pokemon", "sprites/pokemon"),
    ("sprites-trainers", "sprites/trainers"),
    ("sprites-overworld", "sprites/overworld"),
    ("sprites-items", "sprites/items"),
    ("sprites-icons", "sprites/icons"),
    ("sprites-battle-ui", "sprites/battle-ui"),
    ("maps", "maps"),
    ("models", "models"),
    ("bgm", "audio/bgm"),
    ("se", "audio/se"),
];

/// Expands group names; errors on unknown ones.
fn select(only: &[String]) -> Res<Vec<&'static str>> {
    if only.is_empty() {
        return Ok(CATEGORIES.iter().map(|c| c.0).collect());
    }
    let mut v = Vec::new();
    for o in only {
        let o = o.trim();
        let group: Vec<&str> = match o {
            "sprites" => CATEGORIES
                .iter()
                .map(|c| c.0)
                .filter(|c| c.starts_with("sprites-"))
                .collect(),
            "audio" => vec!["bgm", "se"],
            n => vec![n],
        };
        for n in group {
            let c = CATEGORIES
                .iter()
                .find(|c| c.0 == n)
                .ok_or_else(|| format!("unknown category '{o}'"))?;
            if !v.contains(&c.0) {
                v.push(c.0);
            }
        }
    }
    Ok(v)
}

// ---------------------------------------------------------------- shared context

enum Encoder {
    None,
    Oggenc(PathBuf),
    Ffmpeg(PathBuf),
}

fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

fn find_encoder() -> Encoder {
    if let Some(p) = on_path("oggenc") {
        return Encoder::Oggenc(p);
    }
    if let Some(p) = on_path("ffmpeg") {
        let has = Command::new(&p)
            .args(["-hide_banner", "-encoders"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("libvorbis"))
            .unwrap_or(false);
        if has {
            return Encoder::Ffmpeg(p);
        }
    }
    Encoder::None
}

struct Ctx {
    files: PathBuf,
    build: PathBuf,
    out: PathBuf,
    encoder: Encoder,
    sdat: Vec<u8>,
    map_tex: OnceLock<Vec<(usize, Tex0)>>,
}

impl Ctx {
    fn narc(&self, id: u32) -> Res<Narc> {
        let p = self
            .files
            .join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
        let b = std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        Narc::parse(&b).map_err(es)
    }
}

#[derive(Default)]
struct Outcome {
    msg: String,
    /// Non-fatal per-item problems.
    warnings: Vec<String>,
    audio: Vec<AudioEntry>,
}

struct AudioEntry {
    id: usize,
    name: String,
    file: String,
    frames: usize,
    rate: u32,
    looping: Option<(usize, usize)>,
}

type Job = Box<dyn Fn() -> Res<Outcome> + Send + Sync>;

struct Unit {
    cat: &'static str,
    job: Job,
}

// ---------------------------------------------------------------- RON helpers

/// Debug text of plain data (no strings) turned into RON: `Name { a: 1 }` -> `Name ( a: 1 )`.
fn ron_of<T: std::fmt::Debug>(v: &T) -> String {
    format!("{v:?}").replace('{', "(").replace('}', ")")
}

fn write_text(path: &Path, text: &str) -> Res<()> {
    write_bytes(path, text.as_bytes())
}

fn write_bytes(path: &Path, b: &[u8]) -> Res<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(es)?;
    }
    std::fs::write(path, b).map_err(|e| format!("{}: {e}", path.display()))
}

fn ron_str(s: &str) -> String {
    format!("{s:?}")
}

// ---------------------------------------------------------------- converters

fn do_data(c: &Ctx) -> Res<Outcome> {
    let src = RomSource {
        files: c.files.clone(),
        overlay93: c.build.join("OVY_93.sbin"),
        main_sbin: c.build.join("main.sbin"),
    };
    let n = extract_data(&src, &c.out)?.len();
    Ok(Outcome {
        msg: format!("{n} tables"),
        ..Outcome::default()
    })
}

/// `(ARC id, file prefix)`: ARC 002 = system/name banks, ARC 003 = story banks.
const TEXT_ARCS: [(u32, &str); 2] = [(2, "sys"), (3, "story")];

fn do_text(c: &Ctx, arc: u32, prefix: &str) -> Res<Outcome> {
    let narc = c.narc(arc)?;
    let mut o = Outcome::default();
    let mut entries = 0;
    for i in 0..narc.len() {
        match Bank::parse(narc.file(i)) {
            Ok(b) => {
                entries += b.entries().len();
                write_text(
                    &c.out.join(format!("text/en/{prefix}_{i:03}.json")),
                    &b.to_json(),
                )?;
            }
            Err(e) => o.warnings.push(format!("{prefix}_{i:03}: {e}")),
        }
    }
    o.msg = format!("{} banks, {entries} entries", narc.len());
    Ok(o)
}

fn do_fonts(c: &Ctx) -> Res<Outcome> {
    const COLUMNS: usize = 64;
    let narc = c.narc(23)?;
    let fonts = gffont::fonts_from_narc(&narc);
    for (i, f) in &fonts {
        write_bytes(
            &c.out.join(format!("fonts/font{i}.png")),
            &f.atlas(COLUMNS).to_png().map_err(es)?,
        )?;
        let mut by_index: BTreeMap<usize, u16> = BTreeMap::new();
        for code in 0..=u16::MAX {
            if let Some(g) = f.glyph_index(code) {
                by_index.entry(g).or_insert(code);
            }
        }
        let mut s = String::new();
        let _ = write!(
            s,
            "(\n    atlas: \"font{i}.png\",\n    columns: {COLUMNS},\n    cell_w: {},\n    cell_h: {},\n    baseline: {},\n    max_width: {},\n    bpp: {},\n    line_feed: {},\n    // (atlas index, UTF-16 code or 65535 when unmapped, left, glyph width, advance)\n    glyphs: [\n",
            f.cell_w, f.cell_h, f.baseline, f.max_width, f.bpp, f.line_feed
        );
        for n in 0..f.len() {
            if let Some(g) = f.glyph_at(n) {
                let code = by_index.get(&n).copied().unwrap_or(0xFFFF);
                let _ = writeln!(
                    s,
                    "        ({n}, {code}, {}, {}, {}),",
                    g.left, g.glyph_width, g.char_width
                );
            }
        }
        s.push_str("    ],\n)\n");
        write_text(&c.out.join(format!("fonts/font{i}.metrics.ron")), &s)?;
    }
    Ok(Outcome {
        msg: format!("{} fonts", fonts.len()),
        ..Outcome::default()
    })
}

fn sprite_msg<K: Display>(failures: &[(K, String)], o: &mut Outcome) {
    for (n, e) in failures {
        o.warnings.push(format!("sprite set {n}: {e}"));
    }
}

fn do_sprites_pokemon(c: &Ctx) -> Res<Outcome> {
    let r = crate::cmd_extract_sprites::pokemon::extract(&c.files, &c.out).map_err(es)?;
    let mut o = Outcome {
        msg: format!(
            "{} blocks, {} sprite sets, {} icons",
            r.blocks, r.sprites, r.icons
        ),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

fn do_sprites_trainers(c: &Ctx) -> Res<Outcome> {
    let r = crate::cmd_extract_sprites::trainers::extract(&c.files, &c.out).map_err(es)?;
    let mut o = Outcome {
        msg: format!("{} sprite sets", r.sets),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

fn do_sprites_overworld(c: &Ctx) -> Res<Outcome> {
    let r = crate::cmd_extract_sprites::overworld::extract(&c.files, &c.out).map_err(es)?;
    let mut o = Outcome {
        msg: format!("{} character sets, {} frames", r.sets, r.frames),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

fn do_sprites_items(c: &Ctx) -> Res<Outcome> {
    let main = c.build.join("main.sbin");
    let r = crate::cmd_extract_sprites::items::extract_at(&c.files, &c.out, &main).map_err(es)?;
    let mut o = Outcome {
        msg: format!("{} items, {} unique icons", r.items, r.unique_icons),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

fn do_sprites_icons(c: &Ctx) -> Res<Outcome> {
    let r = crate::cmd_extract_sprites::ui::extract(&c.files, &c.out, "icons").map_err(es)?;
    let mut o = Outcome {
        msg: format!("{} icons", r.images),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

fn do_sprites_battle_ui(c: &Ctx) -> Res<Outcome> {
    let r = crate::cmd_extract_sprites::ui::extract(&c.files, &c.out, "battle-ui").map_err(es)?;
    let mut o = Outcome {
        msg: format!("{} battle UI images", r.images),
        ..Outcome::default()
    };
    sprite_msg(&r.failures, &mut o);
    Ok(o)
}

/// Zone RON (header, matrix, events) for every zone, plus one container RON (permission
/// grid, buildings) per `WB` map container; the matrix cells index the containers.
fn do_maps(c: &Ctx) -> Res<Outcome> {
    let zones = ZoneHeader::parse_table(c.narc(ARC_ZONE_HEADERS)?.file(0)).map_err(es)?;
    let matrices = c.narc(ARC_MAP_MATRIX)?;
    let events = c.narc(ARC_ZONE_EVENTS)?;
    let containers = c.narc(ARC_MAP_CONTAINER)?;
    let mut o = Outcome::default();
    for (id, z) in zones.iter().enumerate() {
        let mut s = format!("(\n    zone: {id},\n    header: {},\n", ron_of(z));
        match matrices
            .get(z.matrix_id as usize)
            .map(MapMatrix::parse)
            .transpose()
        {
            Ok(Some(m)) => {
                let _ = writeln!(s, "    matrix: ({}, {}),", z.matrix_id, ron_of(&m));
            }
            Ok(None) => o
                .warnings
                .push(format!("zone {id}: matrix id out of range")),
            Err(e) => o.warnings.push(format!("zone {id}: matrix: {e}")),
        }
        match events.get(id).map(ZoneEvents::parse).transpose() {
            Ok(Some(e)) => {
                let _ = writeln!(s, "    events: {},", ron_of(&e));
            }
            Ok(None) => {}
            Err(e) => o.warnings.push(format!("zone {id}: events: {e}")),
        }
        s.push_str(")\n");
        write_text(&c.out.join(format!("maps/zone_{id:03}.ron")), &s)?;
    }
    let mut wb = 0;
    for i in 0..containers.len() {
        let Ok(mc) = MapContainer::parse(containers.file(i)) else {
            o.warnings.push(format!("container {i}: parse failed"));
            continue;
        };
        if !mc.is_wb() {
            continue;
        }
        let g = mc.permissions().map_err(es)?;
        let mut s = format!(
            "(\n    container: {i},\n    permissions: (\n        width: {}, height: {},\n        trailing: {:?},\n        cells: [",
            g.width, g.height, g.trailing
        );
        for (k, p) in g.cells.iter().enumerate() {
            if k % 8 == 0 {
                s.push_str("\n            ");
            }
            let _ = write!(s, "({},{},{},{}), ", p.unk_00, p.unk_02, p.unk_04, p.unk_06);
        }
        s.push_str("\n        ],\n    ),\n    buildings: [");
        for b in mc.buildings().map_err(es)? {
            let _ = write!(s, "\n        ({:?}, {}),", b.pos, b.unk_0c);
        }
        s.push_str("\n    ],\n)\n");
        write_text(&c.out.join(format!("maps/container_{i:03}.ron")), &s)?;
        wb += 1;
    }
    o.msg = format!("{} zones, {wb} containers", zones.len());
    Ok(o)
}

const MODEL_CHUNK: usize = 24;

/// Every BTX0 of ARC 014, parsed once and shared by the model jobs.
fn map_textures(c: &Ctx) -> Res<&Vec<(usize, Tex0)>> {
    if let Some(v) = c.map_tex.get() {
        return Ok(v);
    }
    let narc = c.narc(14)?;
    let v = (0..narc.len())
        .filter(|&i| narc.file(i).starts_with(b"BTX0"))
        .filter_map(|i| Tex0::parse_btx0(narc.file(i)).ok().map(|t| (i, t)))
        .collect();
    Ok(c.map_tex.get_or_init(|| v))
}

fn do_models(c: &Ctx, lo: usize, hi: usize) -> Res<Outcome> {
    let containers = c.narc(ARC_MAP_CONTAINER)?;
    let texs = map_textures(c)?;
    let mut o = Outcome::default();
    let mut n = 0;
    for i in lo..hi.min(containers.len()) {
        let Ok(mc) = MapContainer::parse(containers.file(i)) else {
            continue;
        };
        let Some(model) = mc.model() else { continue };
        let r: Res<()> = (|| {
            let bmd = Bmd0::parse(model).map_err(es)?;
            // Same choice as `black g3d --tex-narc`: the BTX0 matching most texture names.
            let ext = if bmd.tex.is_none() {
                let want: Vec<&str> = bmd
                    .models
                    .iter()
                    .flat_map(|m| m.materials.iter().filter_map(|x| x.texture.as_deref()))
                    .collect();
                let mut best: Option<(usize, &Tex0)> = None;
                for (_, t) in texs {
                    let k = want.iter().filter(|w| t.texture(w).is_some()).count();
                    if k > 0 && best.is_none_or(|b| k > b.0) {
                        best = Some((k, t));
                    }
                }
                best.map(|b| b.1)
            } else {
                None
            };
            let stem = format!("map_{i:03}");
            let g = gltf::export(&bmd, ext, &stem).map_err(es)?;
            // Texture names repeat across models with different pixels, so the PNGs are
            // content addressed (models/textures/<sha1-12>_<name>) and the glTF rewritten.
            let mut text = g.gltf.clone();
            for (name, png) in &g.images {
                let new = format!("textures/{}_{name}", &sha1_hex(png)[..12]);
                text = text.replace(
                    &format!("\"uri\":\"{name}\""),
                    &format!("\"uri\":\"{new}\""),
                );
                write_bytes(&c.out.join("models").join(&new), png)?;
            }
            write_text(&c.out.join(format!("models/{stem}.gltf")), &text)?;
            write_bytes(&c.out.join(format!("models/{stem}.bin")), &g.bin)?;
            Ok(())
        })();
        match r {
            Ok(()) => n += 1,
            Err(e) => o.warnings.push(format!("container {i}: {e}")),
        }
    }
    o.msg = format!("{n} models");
    Ok(o)
}

// ---------------------------------------------------------------- audio

fn encode_ogg(enc: &Encoder, wav: &[u8], out: &Path, looping: Option<(usize, usize)>) -> Res<()> {
    let mut cmd;
    match enc {
        Encoder::None => return Err("no encoder".into()),
        Encoder::Oggenc(p) => {
            cmd = Command::new(p);
            cmd.args(["-Q", "-q", "5"]);
            if let Some((s, e)) = looping {
                cmd.args(["-c", &format!("LOOPSTART={s}")]);
                cmd.args(["-c", &format!("LOOPLENGTH={}", e - s)]);
            }
            cmd.arg("-o").arg(out).arg("-");
        }
        Encoder::Ffmpeg(p) => {
            cmd = Command::new(p);
            cmd.args([
                "-nostdin",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "wav",
                "-i",
                "pipe:0",
            ]);
            cmd.args(["-map_metadata", "-1", "-c:a", "libvorbis", "-q:a", "5"]);
            cmd.args(["-fflags", "+bitexact", "-flags:a", "+bitexact"]);
            if let Some((s, e)) = looping {
                cmd.args(["-metadata", &format!("LOOPSTART={s}")]);
                cmd.args(["-metadata", &format!("LOOPLENGTH={}", e - s)]);
            }
            cmd.args(["-f", "ogg"]).arg(out);
        }
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(es)?;
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    let o = std::thread::scope(|s| {
        s.spawn(move || {
            let _ = stdin.write_all(wav);
        });
        child.wait_with_output()
    })
    .map_err(es)?;
    if !o.status.success() {
        let _ = std::fs::remove_file(out);
        return Err(String::from_utf8_lossy(&o.stderr).trim().to_string());
    }
    Ok(())
}

fn do_audio(c: &Ctx, dir: &str, ids: &[(usize, String)]) -> Res<Outcome> {
    let sdat = Sdat::parse(&c.sdat).map_err(es)?;
    let mut r = Renderer::new(&sdat);
    let opts = RenderOptions::default();
    let mut o = Outcome::default();
    let has_enc = !matches!(c.encoder, Encoder::None);
    for (id, name) in ids {
        let rendered = match r.render(*id, &opts) {
            Ok(x) => x,
            Err(e) => {
                o.warnings.push(format!("seq {id} ({name}): {e}"));
                continue;
            }
        };
        let wav = rendered.to_wav();
        let base = c.out.join(dir).join(id.to_string());
        if let Some(p) = base.parent() {
            std::fs::create_dir_all(p).map_err(es)?;
        }
        let mut file = format!("{id}.ogg");
        let encoded = has_enc
            && match encode_ogg(
                &c.encoder,
                &wav,
                &base.with_extension("ogg"),
                rendered.pcm.looping,
            ) {
                Ok(()) => true,
                Err(e) => {
                    o.warnings
                        .push(format!("seq {id}: ogg encode failed ({e}), kept WAV"));
                    false
                }
            };
        if !encoded {
            file = format!("{id}.wav");
            write_bytes(&base.with_extension("wav"), &wav)?;
        }
        o.audio.push(AudioEntry {
            id: *id,
            name: name.clone(),
            file,
            frames: rendered.pcm.frames(),
            rate: rendered.pcm.sample_rate,
            looping: rendered.pcm.looping,
        });
    }
    o.msg = format!("{} sequences", o.audio.len());
    Ok(o)
}

fn write_audio_index(out: &Path, dir: &str, mut v: Vec<&AudioEntry>) -> Res<()> {
    v.sort_by_key(|e| e.id);
    let mut s = String::from("[\n");
    for e in v {
        let l = e
            .looping
            .map_or("None".to_string(), |(a, b)| format!("Some(({a}, {b}))"));
        let _ = writeln!(
            s,
            "    (id: {}, name: {}, file: {}, frames: {}, sample_rate: {}, loop_samples: {l}),",
            e.id,
            ron_str(&e.name),
            ron_str(&e.file),
            e.frames,
            e.rate
        );
    }
    s.push_str("]\n");
    write_text(&out.join(dir).join("index.ron"), &s)
}

// ---------------------------------------------------------------- pool, manifest

fn pool(units: &[Unit], jobs: usize) -> Vec<(Res<Outcome>, Duration)> {
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<(usize, Res<Outcome>, Duration)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..jobs.clamp(1, units.len().max(1)) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some(u) = units.get(i) else { break };
                let t = Instant::now();
                let r = (u.job)();
                results.lock().unwrap().push((i, r, t.elapsed()));
            });
        }
    });
    let mut v = results.into_inner().unwrap();
    v.sort_by_key(|x| x.0);
    v.into_iter().map(|(_, r, d)| (r, d)).collect()
}

/// Every file under `dir` (relative path with `/`, size), sorted, skipping `skip`.
fn walk(dir: &Path, skip: &str) -> Vec<(String, u64)> {
    fn rec(base: &Path, d: &Path, skip: &str, v: &mut Vec<(String, u64)>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                rec(base, &p, skip, v);
            } else if let Ok(m) = e.metadata() {
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if rel != skip {
                    v.push((rel, m.len()));
                }
            }
        }
    }
    let mut v = Vec::new();
    rec(dir, dir, skip, &mut v);
    v.sort();
    v
}

fn category_of(path: &str) -> String {
    let mut it = path.split('/');
    match (it.next(), it.next()) {
        (Some(a @ ("sprites" | "audio")), Some(b)) if path.matches('/').count() >= 2 => {
            format!("{a}/{b}")
        }
        (Some(a), _) => a.to_string(),
        _ => String::new(),
    }
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Hashes every file under `out` (except the manifest) and writes `manifest.json`.
fn write_manifest(out: &Path, jobs: usize) -> Res<usize> {
    const NAME: &str = "manifest.json";
    let files = walk(out, NAME);
    let next = AtomicUsize::new(0);
    let hashes: Mutex<Vec<(usize, Res<String>)>> = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..jobs.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::SeqCst);
                let Some((p, _)) = files.get(i) else { break };
                let h = std::fs::read(out.join(p))
                    .map(|b| sha1_hex(&b))
                    .map_err(|e| format!("{p}: {e}"));
                hashes.lock().unwrap().push((i, h));
            });
        }
    });
    let mut hashes = hashes.into_inner().unwrap();
    hashes.sort_by_key(|h| h.0);
    let mut s = format!(
        "{{\n  \"format\": 1,\n  \"converter_version\": {},\n  \"files\": [",
        json_str(env!("CARGO_PKG_VERSION"))
    );
    for ((p, size), (_, h)) in files.iter().zip(hashes) {
        let _ = write!(
            s,
            "{}\n    {{\"path\": {}, \"category\": {}, \"size\": {size}, \"sha1\": {}}}",
            if s.ends_with('[') { "" } else { "," },
            json_str(p),
            json_str(&category_of(p)),
            json_str(&h?)
        );
    }
    s.push_str("\n  ]\n}\n");
    write_text(&out.join(NAME), &s)?;
    Ok(files.len())
}

fn human(n: u64) -> String {
    match n {
        n if n >= 1 << 30 => format!("{:.2} GiB", n as f64 / f64::from(1u32 << 30)),
        n if n >= 1 << 20 => format!("{:.1} MiB", n as f64 / f64::from(1u32 << 20)),
        n => format!("{:.1} KiB", n as f64 / 1024.0),
    }
}

// ---------------------------------------------------------------- command

type AddFn = Box<dyn Fn(&Ctx) -> Res<Outcome> + Send + Sync>;

pub fn run(a: &Args) -> Result<bool, Box<dyn std::error::Error>> {
    let wall = Instant::now();
    let cats = select(&a.only)?;
    let jobs = a
        .jobs
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
        .max(1);
    let build = a
        .build_dir
        .clone()
        .or_else(|| std::env::var_os("BLACK_ROM_BUILD").map(PathBuf::from))
        .unwrap_or_else(|| a.rom_root.join("build/black.us"));
    let want = |c: &str| cats.contains(&c);
    let audio = want("bgm") || want("se");
    let encoder = if a.wav { Encoder::None } else { find_encoder() };
    match (&encoder, audio, a.wav) {
        (Encoder::None, true, false) => println!(
            "note: neither oggenc nor ffmpeg (with libvorbis) found on PATH; audio is kept as WAV"
        ),
        (Encoder::Oggenc(p) | Encoder::Ffmpeg(p), true, _) => {
            println!("audio: encoding OGG Vorbis with {}", p.display());
        }
        _ => {}
    }
    let sdat = if audio {
        std::fs::read(a.rom_root.join("files/wb_sound_data.sdat"))?
    } else {
        Vec::new()
    };
    let ctx = Arc::new(Ctx {
        files: a.rom_root.join("files"),
        build,
        out: a.out.clone(),
        encoder,
        sdat,
        map_tex: OnceLock::new(),
    });
    std::fs::create_dir_all(&a.out)?;

    let mut units: Vec<Unit> = Vec::new();
    let mut add = |cat: &'static str, f: AddFn| {
        let c = ctx.clone();
        units.push(Unit {
            cat,
            job: Box::new(move || f(&c)),
        });
    };
    // Longest first so the pool does not end on a straggler.
    if audio {
        let sdat = Sdat::parse(&ctx.sdat)?;
        let (mut bgm, mut se) = (Vec::new(), Vec::new());
        for (id, e) in sdat.entries(Kind::Seq).iter().enumerate() {
            let Some(e) = e else { continue };
            let name = e.name.clone().unwrap_or_default();
            match id {
                1000..=1178 | 1300..=1331 => bgm.push((id, name)),
                1350..=2075 => se.push((id, name)),
                _ => {}
            }
        }
        for (cat, dir, list, chunk) in [("bgm", "audio/bgm", bgm, 4), ("se", "audio/se", se, 40)] {
            if !want(cat) {
                continue;
            }
            for part in list.chunks(chunk) {
                let part = part.to_vec();
                add(cat, Box::new(move |c| do_audio(c, dir, &part)));
            }
        }
    }
    if want("models") {
        let n = ctx.narc(ARC_MAP_CONTAINER)?.len();
        for lo in (0..n).step_by(MODEL_CHUNK) {
            add(
                "models",
                Box::new(move |c| do_models(c, lo, lo + MODEL_CHUNK)),
            );
        }
    }
    if want("sprites-pokemon") {
        add("sprites-pokemon", Box::new(do_sprites_pokemon));
    }
    if want("sprites-overworld") {
        add("sprites-overworld", Box::new(do_sprites_overworld));
    }
    if want("sprites-trainers") {
        add("sprites-trainers", Box::new(do_sprites_trainers));
    }
    if want("sprites-items") {
        add("sprites-items", Box::new(do_sprites_items));
    }
    if want("sprites-icons") {
        add("sprites-icons", Box::new(do_sprites_icons));
    }
    if want("sprites-battle-ui") {
        add("sprites-battle-ui", Box::new(do_sprites_battle_ui));
    }
    if want("maps") {
        add("maps", Box::new(do_maps));
    }
    if want("text") {
        for (arc, prefix) in TEXT_ARCS {
            add("text", Box::new(move |c| do_text(c, arc, prefix)));
        }
    }
    if want("fonts") {
        add("fonts", Box::new(do_fonts));
    }
    if want("data") {
        add("data", Box::new(do_data));
    }

    let results = pool(&units, jobs);

    let mut ok = true;
    let mut cpu: BTreeMap<&str, Duration> = BTreeMap::new();
    let mut msgs: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut audio_idx: BTreeMap<&str, Vec<AudioEntry>> = BTreeMap::new();
    for (u, (r, d)) in units.iter().zip(results) {
        *cpu.entry(u.cat).or_default() += d;
        match r {
            Ok(o) => {
                msgs.entry(u.cat).or_default().push(o.msg);
                for w in o.warnings {
                    eprintln!("warning [{}]: {w}", u.cat);
                }
                audio_idx.entry(u.cat).or_default().extend(o.audio);
            }
            Err(e) => {
                ok = false;
                eprintln!("error [{}]: {e}", u.cat);
            }
        }
    }
    for (cat, dir) in [("bgm", "audio/bgm"), ("se", "audio/se")] {
        if let Some(v) = audio_idx.get(cat) {
            write_audio_index(&a.out, dir, v.iter().collect())?;
        }
    }
    println!(
        "{:<18} {:>7} {:>12} {:>8}  detail",
        "category", "files", "size", "cpu s"
    );
    for &(cat, dir) in CATEGORIES.iter().filter(|c| want(c.0)) {
        let files = walk(&a.out.join(dir), "");
        let bytes: u64 = files.iter().map(|f| f.1).sum();
        let m = msgs.get(cat).cloned().unwrap_or_default();
        let detail = m.join("; ");
        println!(
            "{cat:<18} {:>7} {:>12} {:>8.1}  {}",
            files.len(),
            human(bytes),
            cpu.get(cat).map_or(0.0, Duration::as_secs_f64),
            if detail.len() > 90 {
                format!("{} units", m.len())
            } else {
                detail
            }
        );
    }
    let n = write_manifest(&a.out, jobs)?;
    println!(
        "manifest.json: {n} files; {} jobs; {:.1} s wall",
        jobs,
        wall.elapsed().as_secs_f64()
    );
    Ok(ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_groups_and_errors() {
        assert_eq!(select(&[]).unwrap().len(), CATEGORIES.len());
        assert_eq!(select(&["audio".into()]).unwrap(), ["bgm", "se"]);
        // sprites = 6 categories (pokemon, trainers, overworld, items, icons, battle-ui) + data
        assert_eq!(select(&["sprites".into(), "data".into()]).unwrap().len(), 7);
        assert!(select(&["nope".into()]).is_err());
    }

    #[test]
    fn ron_of_turns_braces_into_parens() {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct P {
            a: u8,
            b: [u8; 2],
        }
        assert_eq!(ron_of(&P { a: 1, b: [2, 3] }), "P ( a: 1, b: [2, 3] )");
    }

    /// `BLACK_ROM_DIR` or `<manifest>/../../../black-rom`, when it holds a real ROM.
    fn rom_root() -> Option<PathBuf> {
        let root = std::env::var_os("BLACK_ROM_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom")
            });
        root.join("files/a/0/0/0").exists().then_some(root)
    }

    /// Every zone and container RON written by `do_maps` must parse back as a generic value.
    #[test]
    fn maps_ron_parses_back() {
        let Some(rom) = rom_root() else { return };
        let out = std::env::temp_dir().join(format!("blk-a11b-maps-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        let c = Ctx {
            files: rom.join("files"),
            build: rom.join("build/black.us"),
            out: out.clone(),
            encoder: Encoder::None,
            sdat: Vec::new(),
            map_tex: OnceLock::new(),
        };
        do_maps(&c).unwrap();
        let (mut zones, mut containers) = (0, 0);
        for e in std::fs::read_dir(out.join("maps")).unwrap() {
            let p = e.unwrap().path();
            let text = std::fs::read_to_string(&p).unwrap();
            if let Err(err) = ron::from_str::<ron::Value>(&text) {
                panic!("{}: {err}", p.display());
            }
            match p.file_name().unwrap().to_str().unwrap() {
                n if n.starts_with("zone_") => zones += 1,
                n if n.starts_with("container_") => containers += 1,
                n => panic!("unexpected file {n}"),
            }
        }
        let _ = std::fs::remove_dir_all(&out);
        assert!(
            zones > 0 && containers > 0,
            "zones {zones}, containers {containers}"
        );
    }

    #[test]
    fn categories_of_paths() {
        assert_eq!(category_of("sprites/pokemon/1.png"), "sprites/pokemon");
        assert_eq!(category_of("data/x.ron"), "data");
        assert_eq!(category_of("audio/bgm/index.ron"), "audio/bgm");
    }
}
