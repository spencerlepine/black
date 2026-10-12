//! glTF 2.0 export (hand-written JSON; no serde dependency).

use super::model::{Bmd0, Primitive};
use super::tex::Tex0;
use crate::error::Result;
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone, Default)]
pub struct GltfOut {
    /// `<stem>.gltf` contents; refers to `<stem>.bin` and the PNG names in `images`.
    pub gltf: String,
    pub bin: Vec<u8>,
    /// `(file name, PNG bytes)`.
    pub images: Vec<(String, Vec<u8>)>,
    pub vertices: usize,
    pub triangles: usize,
    /// SHA-1 over the RGBA bytes of all exported images, in export order.
    pub texture_sha1: String,
    /// Materials whose texture or palette could not be found/decoded.
    pub missing_textures: usize,
}

fn clean(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn push_view(bin: &mut Vec<u8>, views: &mut Vec<String>, data: &[u8], target: u32) -> usize {
    views.push(format!(
        r#"{{"buffer":0,"byteOffset":{},"byteLength":{},"target":{target}}}"#,
        bin.len(),
        data.len()
    ));
    bin.extend_from_slice(data);
    views.len() - 1
}

fn f32s<const N: usize>(v: &[[f32; N]]) -> Vec<u8> {
    v.iter().flatten().flat_map(|x| x.to_le_bytes()).collect()
}

fn minmax(v: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in v {
        for i in 0..3 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    }
    (lo, hi)
}

/// Export every model of `bmd` into one glTF scene. Textures come from the model's own `TEX0`
/// or, if absent, from `ext` (e.g. a separate NSBTX).
pub fn export(bmd: &Bmd0, ext: Option<&Tex0>, stem: &str) -> Result<GltfOut> {
    let stem = clean(stem);
    let tex = bmd.tex.as_ref().or(ext);
    let mut out = GltfOut::default();
    let mut views: Vec<String> = Vec::new();
    let mut accessors: Vec<String> = Vec::new();
    let mut materials: Vec<String> = Vec::new();
    let mut textures: Vec<String> = Vec::new();
    let mut images: Vec<String> = Vec::new();
    let mut samplers: Vec<String> = Vec::new();
    let mut meshes: Vec<String> = Vec::new();
    let mut nodes: Vec<String> = Vec::new();
    let mut hash_input: Vec<u8> = Vec::new();
    let mut img_ids: HashMap<(usize, Option<usize>), Option<usize>> = HashMap::new();
    let mut mat_ids: HashMap<(usize, usize), usize> = HashMap::new();

    for (mi, model) in bmd.models.iter().enumerate() {
        let prims = model.build(tex)?;
        let mut jprims = Vec::new();
        for p in prims.iter().filter(|p| !p.indices.is_empty()) {
            let mat_json_id = match p.material {
                Some(m) => {
                    let id = match mat_ids.get(&(mi, m)) {
                        Some(&id) => id,
                        None => {
                            let id = materials.len();
                            let j = material_json(
                                bmd,
                                mi,
                                m,
                                tex,
                                &mut out,
                                &mut img_ids,
                                &mut images,
                                &mut textures,
                                &mut samplers,
                                &mut hash_input,
                            )?;
                            materials.push(j);
                            mat_ids.insert((mi, m), id);
                            id
                        }
                    };
                    Some(id)
                }
                None => None,
            };
            jprims.push(prim_json(
                p,
                mat_json_id,
                &mut out,
                &mut views,
                &mut accessors,
            ));
        }
        if jprims.is_empty() {
            continue;
        }
        meshes.push(format!(
            r#"{{"name":"{}","primitives":[{}]}}"#,
            clean(&model.name),
            jprims.join(",")
        ));
        nodes.push(format!(
            r#"{{"name":"{}","mesh":{}}}"#,
            clean(&model.name),
            meshes.len() - 1
        ));
    }
    out.texture_sha1 = crate::sha1_hex(&hash_input);

    let scene_nodes: Vec<String> = (0..nodes.len()).map(|i| i.to_string()).collect();
    let mut j = String::new();
    j.push_str(r#"{"asset":{"version":"2.0","generator":"nds-rom g3d"}"#);
    let _ = write!(
        j,
        r#","scene":0,"scenes":[{{"nodes":[{}]}}]"#,
        scene_nodes.join(",")
    );
    let _ = write!(j, r#","nodes":[{}]"#, nodes.join(","));
    let _ = write!(j, r#","meshes":[{}]"#, meshes.join(","));
    if !materials.is_empty() {
        let _ = write!(j, r#","materials":[{}]"#, materials.join(","));
    }
    if !textures.is_empty() {
        let _ = write!(j, r#","textures":[{}]"#, textures.join(","));
        let _ = write!(j, r#","images":[{}]"#, images.join(","));
        let _ = write!(j, r#","samplers":[{}]"#, samplers.join(","));
    }
    let _ = write!(j, r#","accessors":[{}]"#, accessors.join(","));
    let _ = write!(j, r#","bufferViews":[{}]"#, views.join(","));
    let _ = write!(
        j,
        r#","buffers":[{{"uri":"{stem}.bin","byteLength":{}}}]}}"#,
        out.bin.len()
    );
    out.gltf = j;
    Ok(out)
}

fn prim_json(
    p: &Primitive,
    mat: Option<usize>,
    out: &mut GltfOut,
    views: &mut Vec<String>,
    acc: &mut Vec<String>,
) -> String {
    let n = p.positions.len();
    let mut add = |bin: &mut Vec<u8>,
                   data: Vec<u8>,
                   target: u32,
                   ct: u32,
                   ty: &str,
                   extra: &str,
                   count: usize| {
        let v = push_view(bin, views, &data, target);
        acc.push(format!(
            r#"{{"bufferView":{v},"componentType":{ct},"count":{count},"type":"{ty}"{extra}}}"#
        ));
        acc.len() - 1
    };
    let (lo, hi) = minmax(&p.positions);
    let extra = format!(
        r#","min":[{},{},{}],"max":[{},{},{}]"#,
        lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]
    );
    let pos = add(
        &mut out.bin,
        f32s(&p.positions),
        34962,
        5126,
        "VEC3",
        &extra,
        n,
    );
    let nor = add(&mut out.bin, f32s(&p.normals), 34962, 5126, "VEC3", "", n);
    let uv = add(&mut out.bin, f32s(&p.uvs), 34962, 5126, "VEC2", "", n);
    let mut attrs = format!(r#""POSITION":{pos},"NORMAL":{nor},"TEXCOORD_0":{uv}"#);
    if p.has_color {
        let c = add(&mut out.bin, f32s(&p.colors), 34962, 5126, "VEC3", "", n);
        let _ = write!(attrs, r#","COLOR_0":{c}"#);
    }
    let idx: Vec<u8> = p.indices.iter().flat_map(|i| i.to_le_bytes()).collect();
    let ix = add(
        &mut out.bin,
        idx,
        34963,
        5125,
        "SCALAR",
        "",
        p.indices.len(),
    );
    out.vertices += n;
    out.triangles += p.indices.len() / 3;
    let m = mat.map_or(String::new(), |m| format!(r#","material":{m}"#));
    format!(r#"{{"attributes":{{{attrs}}},"indices":{ix},"mode":4{m}}}"#)
}

#[allow(clippy::too_many_arguments)]
fn material_json(
    bmd: &Bmd0,
    mi: usize,
    m: usize,
    tex: Option<&Tex0>,
    out: &mut GltfOut,
    img_ids: &mut HashMap<(usize, Option<usize>), Option<usize>>,
    images: &mut Vec<String>,
    textures: &mut Vec<String>,
    samplers: &mut Vec<String>,
    hash_input: &mut Vec<u8>,
) -> Result<String> {
    let mat = &bmd.models[mi].materials[m];
    let mut base = format!(
        "[{:.4},{:.4},{:.4},{:.4}]",
        mat.diffuse[0] as f32 / 255.0,
        mat.diffuse[1] as f32 / 255.0,
        mat.diffuse[2] as f32 / 255.0,
        mat.alpha as f32 / 31.0
    );
    let mut alpha_mode = if mat.alpha < 31 { "BLEND" } else { "OPAQUE" };
    let mut tex_ref = String::new();
    if let (Some(t), Some(tn)) = (tex, mat.texture.as_deref()) {
        let ti = t.texture(tn);
        let pi = mat
            .palette
            .as_deref()
            .and_then(|n| t.palette(n))
            .or_else(|| ti.and_then(|i| t.palette_for(i)));
        let id = match ti {
            None => None,
            Some(ti) => *img_ids.entry((ti, pi)).or_insert_with(|| {
                let img = t.decode(ti, pi).ok()?;
                let png = img.to_png().ok()?;
                hash_input.extend_from_slice(&img.rgba);
                let name = format!("tex_{}_{}.png", clean(tn), images.len());
                images.push(format!(r#"{{"uri":"{name}"}}"#));
                out.images.push((name, png));
                let fmt = t.textures[ti].format;
                let _ = fmt;
                Some(images.len() - 1)
            }),
        };
        match id {
            Some(img) => {
                let fmt = ti.map_or(0, |i| t.textures[i].format);
                let tp = mat.tex_param;
                let wrap = |rep: u32, flip: u32| {
                    if tp >> rep & 1 == 0 {
                        33071
                    } else if tp >> flip & 1 != 0 {
                        33648
                    } else {
                        10497
                    }
                };
                samplers.push(format!(
                    r#"{{"magFilter":9728,"minFilter":9728,"wrapS":{},"wrapT":{}}}"#,
                    wrap(16, 18),
                    wrap(17, 19)
                ));
                textures.push(format!(
                    r#"{{"source":{img},"sampler":{}}}"#,
                    samplers.len() - 1
                ));
                tex_ref = format!(r#","baseColorTexture":{{"index":{}}}"#, textures.len() - 1);
                base = format!("[1,1,1,{:.4}]", mat.alpha as f32 / 31.0);
                if matches!(fmt, 1 | 6) {
                    alpha_mode = "BLEND";
                } else if matches!(fmt, 2..=5) && alpha_mode == "OPAQUE" {
                    alpha_mode = "MASK";
                }
            }
            None => out.missing_textures += 1,
        }
    }
    let both = mat.polygon_attr & 0xC0 == 0xC0;
    Ok(format!(
        r#"{{"name":"{}","pbrMetallicRoughness":{{"baseColorFactor":{base}{tex_ref},"metallicFactor":0,"roughnessFactor":1}},"alphaMode":"{alpha_mode}","doubleSided":{both}}}"#,
        clean(&mat.name)
    ))
}
