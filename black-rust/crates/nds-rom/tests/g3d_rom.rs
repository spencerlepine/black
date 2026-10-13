//! G3D readers against the real ROM. Skipped when the ROM files are absent.

use nds_rom::g3d::{anim::AnimFile, gltf, model::Bmd0, tex::Tex0};
use nds_rom::map::{MapContainer, MapMatrix, ARC_MAP_CONTAINER, ARC_MAP_MATRIX};
use nds_rom::narc::Narc;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn files_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("BLACK_ROM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../black-rom"));
    let f = dir.join("files");
    f.is_dir().then_some(f)
}

fn narc(id: u32) -> Option<Narc> {
    let p = files_dir()?.join(format!("a/{}/{}/{}", id / 100, id / 10 % 10, id % 10));
    Narc::parse(&std::fs::read(p).ok()?).ok()
}

fn all_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut es: Vec<_> = std::fs::read_dir(dir).unwrap().flatten().collect();
    es.sort_by_key(|e| e.path());
    for e in es {
        let p = e.path();
        if p.is_dir() {
            all_files(&p, out);
        } else {
            out.push(p);
        }
    }
}

/// Parse every BMD0/BTX0/animation file in the ROM (bare, NARC member, or map container).
#[test]
fn parse_every_g3d_file() {
    let Some(files) = files_dir() else { return };
    let mut paths = Vec::new();
    all_files(&files, &mut paths);
    let mut ok: BTreeMap<&str, usize> = BTreeMap::new();
    let mut fail: Vec<String> = Vec::new();
    let mut models = 0;
    let mut textures = 0;
    let mut decoded = 0;
    let mut unpaired = 0;
    let mut tri_mismatch = 0;
    let mut tri_total = 0;
    let mut check = |name: String, d: &[u8]| {
        let kind = match d.get(..4) {
            Some(b"BMD0") => "BMD0",
            Some(b"BTX0") => "BTX0",
            Some(b"BCA0") => "BCA0",
            Some(b"BTA0") => "BTA0",
            Some(b"BTP0") => "BTP0",
            Some(b"BMA0") => "BMA0",
            Some(b"BVA0") => "BVA0",
            _ => return,
        };
        let r: Result<(), String> = (|| {
            match kind {
                "BMD0" => {
                    let b = Bmd0::parse(d).map_err(|e| e.to_string())?;
                    models += b.models.len();
                    let g = gltf::export(&b, None, "x").map_err(|e| e.to_string())?;
                    tri_total += g.triangles;
                    let want: usize = b
                        .models
                        .iter()
                        .map(|m| (m.stats[2] + 2 * m.stats[3]) as usize)
                        .sum();
                    tri_mismatch += usize::from(want != g.triangles);
                    if let Some(t) = &b.tex {
                        textures += t.textures.len();
                        for i in 0..t.textures.len().min(4) {
                            let pal = t.palette_for(i);
                            if pal.is_none() && t.textures[i].format != 7 {
                                unpaired += 1;
                                continue;
                            }
                            t.decode(i, pal).map_err(|e| e.to_string())?;
                            decoded += 1;
                        }
                    }
                }
                "BTX0" => {
                    let t = Tex0::parse_btx0(d).map_err(|e| e.to_string())?;
                    textures += t.textures.len();
                    for i in 0..t.textures.len().min(4) {
                        let pal = t.palette_for(i);
                        if pal.is_none() && t.textures[i].format != 7 {
                            unpaired += 1;
                            continue;
                        }
                        t.decode(i, pal).map_err(|e| e.to_string())?;
                        decoded += 1;
                    }
                }
                _ => {
                    AnimFile::parse(d).map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        })();
        match r {
            Ok(()) => *ok.entry(kind).or_default() += 1,
            Err(e) => fail.push(format!("{name}: {kind}: {e}")),
        }
    };
    for p in paths {
        let Ok(d) = std::fs::read(&p) else { continue };
        let n = p.strip_prefix(&files).unwrap().display().to_string();
        if let Ok(nr) = Narc::parse(&d) {
            for (i, f) in nr.files().enumerate() {
                if f.starts_with(b"WB") {
                    if let Some(m) = MapContainer::parse(f)
                        .ok()
                        .and_then(|c| c.model().map(<[u8]>::to_vec))
                    {
                        check(format!("{n}:{i}:map"), &m);
                    }
                } else {
                    check(format!("{n}:{i}"), f);
                }
            }
        } else {
            check(n, &d);
        }
    }
    eprintln!("g3d: parsed ok {ok:?}; {models} models, {textures} textures ({decoded} decoded, {unpaired} unpaired), {tri_total} triangles, {tri_mismatch} models with triangle count != header");
    for f in fail.iter().take(20) {
        eprintln!("FAIL {f}");
    }
    assert!(fail.is_empty(), "{} g3d files failed", fail.len());
}

/// Texture decoding is capped at 4 per file in the scan above; `decoded` counts those.
fn map_model(id: usize) -> Option<Vec<u8>> {
    let c = narc(ARC_MAP_CONTAINER)?;
    MapContainer::parse(c.get(id)?)
        .ok()?
        .model()
        .map(<[u8]>::to_vec)
}

/// Export `d` with textures from member `tex_member` of ARC 014 (or its own TEX0).
fn golden(d: &[u8], tex: Option<Tex0>) -> (gltf::GltfOut, Bmd0) {
    let b = Bmd0::parse(d).unwrap();
    assert_eq!(b.models.len(), 1);
    let g = gltf::export(&b, tex.as_ref(), "g").unwrap();
    let m = &b.models[0];
    // Every triangle/quad of the header statistics must come out of the display lists.
    assert_eq!(g.triangles, (m.stats[2] + 2 * m.stats[3]) as usize);
    assert_eq!(g.missing_textures, 0);
    (g, b)
}

/// Container 389, the model `m_dun1703_00_00` (a dungeon map; A6 took it for Nuvema Town because the
/// interleaved matrix reading put the word 389, really the zone id of Nuvema in matrix 0's zone plane at block
/// (24, 23), into the container cells; see `map_rom::matrix_planes_match_the_game`).
#[test]
fn nuvema_golden() {
    let (Some(d), Some(c14), Some(m)) = (map_model(389), narc(14), narc(ARC_MAP_MATRIX)) else {
        return;
    };
    let mx = MapMatrix::parse(m.file(0)).unwrap();
    assert_eq!(mx.zone_at(24, 23), Some(389));
    let tex = Tex0::parse_btx0(c14.file(251)).unwrap();
    let (g, b) = golden(&d, Some(tex));
    assert_eq!(b.models[0].name, "m_dun1703_00_00");
    assert_eq!((g.vertices, g.triangles, g.images.len()), (2926, 1987, 9));
    assert_eq!(g.texture_sha1, "bc862dd6f945e7d668f67f13f4fde2ba0c80fe61");
}

/// An outdoor overworld map (`map23_23`, container 1) with its terrain textures (ARC 014 #2).
#[test]
fn overworld_map_golden() {
    let (Some(d), Some(c14)) = (map_model(1), narc(14)) else {
        return;
    };
    let tex = Tex0::parse_btx0(c14.file(2)).unwrap();
    let (g, b) = golden(&d, Some(tex));
    assert_eq!(b.models[0].name, "map23_23");
    assert_eq!((g.vertices, g.triangles, g.images.len()), (1587, 939, 24));
    assert_eq!(g.texture_sha1, "6cbc7490c22430a5c61bc4e71f53e53bde33de92");
}

/// Battle background `batt_stage01` (ARC 011 #0, BMD0 with its own TEX0).
#[test]
fn battle_background_golden() {
    let Some(c11) = narc(11) else { return };
    let (g, b) = golden(c11.file(0), None);
    assert_eq!(b.models[0].name, "batt_stage01");
    assert_eq!((g.vertices, g.triangles, g.images.len()), (65, 48, 1));
    assert_eq!(g.texture_sha1, "fce13543c23d9fe7ade295a48eb2c64665783157");
    assert!(g.gltf.contains(r#""version":"2.0""#) && g.gltf.contains("baseColorTexture"));
}
