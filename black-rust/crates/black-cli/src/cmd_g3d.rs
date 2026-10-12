//! `black g3d <in> <out_dir>`: convert Nitro 3D models to glTF 2.0 (+ PNG textures).
//!
//! `<in>` is a BMD0 file, a BTX0 file (textures only: PNGs), a map container (`WB`, model in
//! section 0) or a NARC (every BMD0/BTX0 member, or just `--member N`).

use clap::Args;
use nds_rom::g3d::{gltf, model::Bmd0, tex::Tex0};
use nds_rom::map::MapContainer;
use nds_rom::narc::Narc;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Args)]
pub struct G3dArgs {
    pub input: PathBuf,
    pub out_dir: PathBuf,
    /// Only this member of a NARC
    #[arg(long)]
    pub member: Option<usize>,
    /// External BTX0 supplying textures to a BMD0 without its own TEX0
    #[arg(long)]
    pub tex: Option<PathBuf>,
    /// NARC of BTX0 members (e.g. a/0/1/4): use the member matching most texture names
    #[arg(long)]
    pub tex_narc: Option<PathBuf>,
}

/// The BTX0 member of `narc` that provides the most texture names used by `bmd`'s materials.
fn best_tex(narc: &Narc, bmd: &Bmd0) -> Option<(usize, Tex0)> {
    let want: Vec<&str> = bmd
        .models
        .iter()
        .flat_map(|m| m.materials.iter().filter_map(|x| x.texture.as_deref()))
        .collect();
    let mut best: Option<(usize, usize, Tex0)> = None;
    for (i, f) in narc.files().enumerate() {
        if !f.starts_with(b"BTX0") {
            continue;
        }
        let Ok(t) = Tex0::parse_btx0(f) else { continue };
        let n = want.iter().filter(|w| t.texture(w).is_some()).count();
        if n > 0 && best.as_ref().is_none_or(|b| n > b.0) {
            best = Some((n, i, t));
        }
    }
    best.map(|(_, i, t)| (i, t))
}

fn convert(
    bytes: &[u8],
    stem: &str,
    out: &Path,
    ext: Option<&Tex0>,
    tex_narc: Option<&Narc>,
) -> Result<bool> {
    let model_bytes;
    let bytes = if bytes.starts_with(b"WB") {
        match MapContainer::parse(bytes)?.model() {
            Some(m) => {
                model_bytes = m.to_vec();
                &model_bytes[..]
            }
            None => return Ok(false),
        }
    } else {
        bytes
    };
    if bytes.starts_with(b"BTX0") {
        let t = Tex0::parse_btx0(bytes)?;
        let mut n = 0;
        for (i, tx) in t.textures.iter().enumerate() {
            if let Ok(img) = t.decode(i, t.palette(&tx.name)) {
                std::fs::write(out.join(format!("{stem}_{}.png", tx.name)), img.to_png()?)?;
                n += 1;
            }
        }
        println!("{stem}: {} textures, {n} decoded", t.textures.len());
        return Ok(true);
    }
    if !bytes.starts_with(b"BMD0") {
        return Ok(false);
    }
    let bmd = Bmd0::parse(bytes)?;
    let found = match (&bmd.tex, ext, tex_narc) {
        (None, None, Some(n)) => best_tex(n, &bmd),
        _ => None,
    };
    if let Some((i, _)) = &found {
        println!("{stem}: textures from NARC member {i}");
    }
    let g = gltf::export(&bmd, ext.or(found.as_ref().map(|f| &f.1)), stem)?;
    std::fs::write(out.join(format!("{stem}.gltf")), &g.gltf)?;
    std::fs::write(out.join(format!("{stem}.bin")), &g.bin)?;
    for (name, png) in &g.images {
        std::fs::write(out.join(name), png)?;
    }
    println!(
        "{stem}: {} models, {} vertices, {} triangles, {} images (sha1 {}), {} missing",
        bmd.models.len(),
        g.vertices,
        g.triangles,
        g.images.len(),
        g.texture_sha1,
        g.missing_textures
    );
    Ok(true)
}

pub fn run(a: G3dArgs) -> Result<()> {
    std::fs::create_dir_all(&a.out_dir)?;
    let ext = match &a.tex {
        Some(p) => Some(Tex0::parse_btx0(&std::fs::read(p)?)?),
        None => None,
    };
    let tex_narc = match &a.tex_narc {
        Some(p) => Some(Narc::parse(&std::fs::read(p)?)?),
        None => None,
    };
    let bytes = std::fs::read(&a.input)?;
    let stem = a
        .input
        .file_stem()
        .map_or("model".to_string(), |s| s.to_string_lossy().into_owned());
    if bytes.starts_with(b"NARC") {
        let narc = Narc::parse(&bytes)?;
        for i in (0..narc.len()).filter(|i| a.member.is_none_or(|m| m == *i)) {
            if let Err(e) = convert(
                narc.file(i),
                &format!("{stem}_{i}"),
                &a.out_dir,
                ext.as_ref(),
                tex_narc.as_ref(),
            ) {
                eprintln!("{stem}_{i}: {e}");
            }
        }
    } else if !convert(&bytes, &stem, &a.out_dir, ext.as_ref(), tex_narc.as_ref())? {
        return Err("not a BMD0, BTX0, WB map container or NARC".into());
    }
    Ok(())
}
