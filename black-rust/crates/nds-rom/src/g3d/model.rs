//! `BMD0` (NSBMD) models: `MDL0` nodes, materials, shapes (GX display lists) and the scene
//! bytecode (SBC) that draws them; plus [`Model::build`] which flattens that into triangles.

use super::tex::Tex0;
use super::{bad, blocks, i16_at, i32_at, u16_at, u32_at, u8_at, Dict};
use crate::error::Result;
use std::collections::HashMap;

/// 4x4 matrix in the DS convention (row vectors: `p' = p * M`, translation in row 3).
pub type Mat = [[f32; 4]; 4];

pub const IDENT: Mat = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `a * b` = apply `a` first, then `b`.
pub fn mul(a: &Mat, b: &Mat) -> Mat {
    let mut r = [[0.0; 4]; 4];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}

fn xform(m: &Mat, p: [f32; 3]) -> [f32; 3] {
    let f = |j: usize| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j];
    [f(0), f(1), f(2)]
}

fn xform_dir(m: &Mat, p: [f32; 3]) -> [f32; 3] {
    let f = |j: usize| p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j];
    let v = [f(0), f(1), f(2)];
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 1e-6 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        v
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub name: String,
    pub local: Mat,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub texture: Option<String>,
    pub palette: Option<String>,
    /// Diffuse colour (RGB555 expanded to 8 bits).
    pub diffuse: [u8; 3],
    /// POLYGON_ATTR alpha 0..=31.
    pub alpha: u8,
    pub polygon_attr: u32,
    pub tex_param: u32,
    pub width: u32,
    pub height: u32,
    pub flags: u16,
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub name: String,
    pub dl: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub name: String,
    pub pos_scale: f32,
    /// Header statistics: vertices, surfaces, triangles, quads.
    pub stats: [u16; 4],
    pub nodes: Vec<Node>,
    pub materials: Vec<Material>,
    pub shapes: Vec<Shape>,
    pub sbc: Vec<u8>,
}

/// Triangles using one material, with welded vertices.
#[derive(Debug, Clone, Default)]
pub struct Primitive {
    pub material: Option<usize>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[f32; 3]>,
    /// A COLOR command was seen (vertex colours are meaningful).
    pub has_color: bool,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Bmd0 {
    pub models: Vec<Model>,
    pub tex: Option<Tex0>,
}

impl Bmd0 {
    pub fn parse(d: &[u8]) -> Result<Bmd0> {
        let mut models = Vec::new();
        let mut tex = None;
        for (tag, off) in blocks(d, b"BMD0", "BMD0")? {
            match &tag {
                b"MDL0" => models = Model::parse_block(d, off)?,
                b"TEX0" => tex = Some(Tex0::parse_block(d, off)?),
                _ => {}
            }
        }
        Ok(Bmd0 { models, tex })
    }
}

fn fx(v: i32) -> f32 {
    v as f32 / 4096.0
}

fn parse_node(d: &[u8], o: usize) -> Result<Mat> {
    let flags = u16_at(d, o)?;
    let a = i16_at(d, o + 2)?;
    let mut p = o + 4;
    let mut t = [0.0; 3];
    if flags & 1 == 0 {
        for v in &mut t {
            *v = fx(i32_at(d, p)?);
            p += 4;
        }
    }
    let mut r = IDENT;
    if flags & 8 != 0 {
        let (a, b) = (a as f32 / 4096.0, i16_at(d, p)? as f32 / 4096.0);
        p += 2;
        let sel = ((flags >> 4) & 0xF) as usize;
        let neg = (flags >> 8) & 0xF;
        let (or, oc) = (sel / 3, sel % 3);
        r = [[0.0; 4]; 4];
        r[3][3] = 1.0;
        r[or][oc] = if neg & 1 != 0 { -1.0 } else { 1.0 };
        let rows: Vec<usize> = (0..3).filter(|&i| i != or).collect();
        let cols: Vec<usize> = (0..3).filter(|&i| i != oc).collect();
        r[rows[0]][cols[0]] = a;
        r[rows[0]][cols[1]] = b;
        r[rows[1]][cols[0]] = if neg & 2 != 0 { -b } else { b };
        r[rows[1]][cols[1]] = if neg & 4 != 0 { -a } else { a };
    } else if flags & 2 == 0 {
        r[0][0] = a as f32 / 4096.0;
        for (i, j) in [
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ] {
            r[i][j] = i16_at(d, p)? as f32 / 4096.0;
            p += 2;
        }
    }
    let mut s = [1.0; 3];
    if flags & 4 == 0 {
        for v in &mut s {
            *v = fx(i32_at(d, p)?);
            p += 4;
        }
    }
    let mut sm = IDENT;
    for i in 0..3 {
        sm[i][i] = s[i];
    }
    let mut tm = IDENT;
    tm[3][0] = t[0];
    tm[3][1] = t[1];
    tm[3][2] = t[2];
    Ok(mul(&mul(&sm, &r), &tm))
}

fn expand5(c: u32) -> u8 {
    let v = (c & 31) as u8;
    (v << 3) | (v >> 2)
}

impl Model {
    fn parse_block(d: &[u8], b: usize) -> Result<Vec<Model>> {
        let dict = Dict::parse(d, b + 8)?;
        let mut out = Vec::new();
        for (i, name) in dict.names.iter().enumerate() {
            let m = b + u32_at(d, dict.entry(i))? as usize;
            out.push(Model::parse(d, m, name)?);
        }
        Ok(out)
    }

    fn parse(d: &[u8], m: usize, name: &str) -> Result<Model> {
        let size = u32_at(d, m)? as usize;
        let sbc_off = u32_at(d, m + 4)? as usize;
        let mat_off = u32_at(d, m + 8)? as usize;
        let shp_off = u32_at(d, m + 12)? as usize;
        let pos_scale = fx(i32_at(d, m + 0x1C)?);
        let stats = [
            u16_at(d, m + 0x24)?,
            u16_at(d, m + 0x26)?,
            u16_at(d, m + 0x28)?,
            u16_at(d, m + 0x2A)?,
        ];
        let sbc_end = [mat_off, shp_off, size]
            .into_iter()
            .filter(|&o| o > sbc_off)
            .min()
            .unwrap_or(size);
        let sbc = d
            .get(m + sbc_off..m + sbc_end)
            .ok_or_else(|| bad("MDL0", "SBC out of range"))?
            .to_vec();

        // Nodes: dict at +0x40, entries are offsets relative to the dict start.
        let nd = Dict::parse(d, m + 0x40)?;
        let mut nodes = Vec::new();
        for (i, n) in nd.names.iter().enumerate() {
            let o = m + 0x40 + u32_at(d, nd.entry(i))? as usize;
            nodes.push(Node {
                name: n.clone(),
                local: parse_node(d, o)?,
            });
        }

        // Materials.
        let mb = m + mat_off;
        let md = Dict::parse(d, mb + 4)?;
        let mut tex_of: HashMap<usize, String> = HashMap::new();
        let mut pal_of: HashMap<usize, String> = HashMap::new();
        for (dict_at, map) in [
            (mb + u16_at(d, mb)? as usize, &mut tex_of),
            (mb + u16_at(d, mb + 2)? as usize, &mut pal_of),
        ] {
            let td = Dict::parse(d, dict_at)?;
            for (i, tname) in td.names.iter().enumerate() {
                let e = td.entry(i);
                let list = mb + u16_at(d, e)? as usize;
                for k in 0..u8_at(d, e + 2)? as usize {
                    map.insert(u8_at(d, list + k)? as usize, tname.clone());
                }
            }
        }
        let mut materials = Vec::new();
        for (i, n) in md.names.iter().enumerate() {
            let o = mb + u32_at(d, md.entry(i))? as usize;
            let dif = u32_at(d, o + 4)?;
            let pa = u32_at(d, o + 12)?;
            materials.push(Material {
                name: n.clone(),
                texture: tex_of.remove(&i),
                palette: pal_of.remove(&i),
                diffuse: [expand5(dif), expand5(dif >> 5), expand5(dif >> 10)],
                alpha: ((pa >> 16) & 31) as u8,
                polygon_attr: pa,
                tex_param: u32_at(d, o + 20)?,
                flags: u16_at(d, o + 0x1E)?,
                width: u16_at(d, o + 0x20)? as u32,
                height: u16_at(d, o + 0x22)? as u32,
            });
        }

        // Shapes.
        let sb = m + shp_off;
        let sd = Dict::parse(d, sb)?;
        let mut shapes = Vec::new();
        for (i, n) in sd.names.iter().enumerate() {
            let o = sb + u32_at(d, sd.entry(i))? as usize;
            let dl_off = o + u32_at(d, o + 8)? as usize;
            let dl_size = u32_at(d, o + 12)? as usize;
            shapes.push(Shape {
                name: n.clone(),
                dl: d
                    .get(dl_off..dl_off + dl_size)
                    .ok_or_else(|| bad("MDL0", "display list out of range"))?
                    .to_vec(),
            });
        }
        Ok(Model {
            name: name.to_string(),
            pos_scale,
            stats,
            nodes,
            materials,
            shapes,
            sbc,
        })
    }

    /// Interpret the SBC and the display lists; returns one [`Primitive`] per used material.
    pub fn build(&self, tex: Option<&Tex0>) -> Result<Vec<Primitive>> {
        let mut b = Builder {
            model: self,
            tex,
            prims: Vec::new(),
            by_mat: HashMap::new(),
            welded: HashMap::new(),
            ps_on: false,
        };
        let mut world: Vec<Option<Mat>> = vec![None; self.nodes.len()];
        let mut stack: Vec<Option<Mat>> = vec![None; 32];
        let (mut cur, mut mat) = (IDENT, 0usize);
        let s = &self.sbc;
        let mut pc = 0;
        while pc < s.len() {
            let op = s[pc];
            let arg = |i: usize| s.get(pc + i).copied().unwrap_or(0) as usize;
            match op & 0x1F {
                0x00 => pc += 1,
                0x01 => break,
                0x02 => pc += 3,
                0x03 => {
                    if let Some(Some(m)) = stack.get(arg(1)) {
                        cur = *m;
                    }
                    pc += 2;
                }
                0x04 => {
                    mat = arg(1);
                    pc += 2;
                }
                0x05 => {
                    let sh = self
                        .shapes
                        .get(arg(1))
                        .ok_or_else(|| bad("MDL0", "SBC shape index"))?;
                    b.run_dl(&sh.dl, cur, mat, &mut stack)?;
                    pc += 2;
                }
                0x06 => {
                    let (node, parent) = (arg(1), arg(2));
                    let mut len = 4;
                    let (mut store, mut restore) = (None, None);
                    if op & 0x20 != 0 {
                        store = Some(arg(len));
                        len += 1;
                    }
                    if op & 0x40 != 0 {
                        restore = Some(arg(len));
                        len += 1;
                    }
                    let pw = match restore {
                        Some(r) => stack.get(r).copied().flatten(),
                        None => world.get(parent).copied().flatten(),
                    };
                    let local = self.nodes.get(node).map_or(IDENT, |n| n.local);
                    let w = mul(&local, &pw.unwrap_or(IDENT));
                    if let Some(slot) = world.get_mut(node) {
                        *slot = Some(w);
                    }
                    if let Some(Some(slot)) = store.map(|i| stack.get_mut(i)) {
                        *slot = Some(w);
                    }
                    cur = w;
                    pc += len;
                }
                0x07 | 0x08 => pc += 2,
                0x0B => {
                    b.ps_on = op & 0x20 == 0;
                    pc += 1;
                }
                0x09 => pc += 3 + 3 * arg(2),
                0x0C | 0x0D => pc += 3,
                0x0A => pc += 9,
                _ => break,
            }
        }
        Ok(b.prims)
    }
}

struct Builder<'a> {
    model: &'a Model,
    tex: Option<&'a Tex0>,
    prims: Vec<Primitive>,
    by_mat: HashMap<usize, usize>,
    welded: HashMap<(usize, [u32; 11]), u32>,
    ps_on: bool,
}

#[derive(Clone, Copy)]
struct Vtx {
    p: [f32; 3],
    n: [f32; 3],
    uv: [f32; 2],
    c: [f32; 3],
}

fn s10(v: u32) -> i32 {
    ((v & 0x3FF) << 22) as i32 >> 22
}

fn params(op: u8) -> usize {
    match op {
        0x10 | 0x12 | 0x13 | 0x14 => 1,
        0x16 | 0x18 => 16,
        0x17 | 0x19 => 12,
        0x1A => 9,
        0x1B | 0x1C | 0x70 => 3,
        0x1D => 3,
        0x23 | 0x71 => 2,
        0x20..=0x2B | 0x30..=0x33 | 0x40 | 0x50 | 0x60 | 0x72 => 1,
        0x34 => 32,
        _ => 0,
    }
}

impl Builder<'_> {
    fn prim(&mut self, mat: usize) -> usize {
        if let Some(&i) = self.by_mat.get(&mat) {
            return i;
        }
        self.prims.push(Primitive {
            material: (mat < self.model.materials.len()).then_some(mat),
            ..Default::default()
        });
        self.by_mat.insert(mat, self.prims.len() - 1);
        self.prims.len() - 1
    }

    fn emit(&mut self, mat: usize, v: &[Vtx], idx: &[usize], has_color: bool) {
        let pi = self.prim(mat);
        for &i in idx {
            let x = v[i];
            let key = [
                x.p[0].to_bits(),
                x.p[1].to_bits(),
                x.p[2].to_bits(),
                x.n[0].to_bits(),
                x.n[1].to_bits(),
                x.n[2].to_bits(),
                x.uv[0].to_bits(),
                x.uv[1].to_bits(),
                x.c[0].to_bits(),
                x.c[1].to_bits(),
                x.c[2].to_bits(),
            ];
            let p = &mut self.prims[pi];
            let n = p.positions.len() as u32;
            let id = *self.welded.entry((pi, key)).or_insert_with(|| {
                p.positions.push(x.p);
                p.normals.push(x.n);
                p.uvs.push(x.uv);
                p.colors.push(x.c);
                n
            });
            p.indices.push(id);
        }
        self.prims[pi].has_color |= has_color;
    }

    fn run_dl(
        &mut self,
        dl: &[u8],
        start: Mat,
        mat: usize,
        stack: &mut [Option<Mat>],
    ) -> Result<()> {
        let (tw, th) = {
            let m = self.model.materials.get(mat);
            let mut w = m.map_or(0, |m| m.width) as f32;
            let mut h = m.map_or(0, |m| m.height) as f32;
            if let (Some(t), Some(m)) = (self.tex, m) {
                if let Some(ti) = m.texture.as_deref().and_then(|n| t.texture(n)) {
                    if w == 0.0 {
                        w = t.textures[ti].width as f32;
                    }
                    if h == 0.0 {
                        h = t.textures[ti].height as f32;
                    }
                }
            }
            (w.max(1.0), h.max(1.0))
        };
        let ps = if self.ps_on {
            self.model.pos_scale
        } else {
            1.0
        };
        let mut cur = start;
        let mut pos = [0.0f32; 3];
        let mut nrm = [0.0f32, 0.0, 1.0];
        let mut uv = [0.0f32; 2];
        let mut col = [1.0f32; 3];
        let mut has_color = false;
        let mut mode = 0u32;
        let mut vs: Vec<Vtx> = Vec::new();
        let mut off = 0;
        while off + 4 <= dl.len() {
            let ops = [dl[off], dl[off + 1], dl[off + 2], dl[off + 3]];
            off += 4;
            for op in ops {
                if op == 0 {
                    continue;
                }
                let n = params(op);
                let mut w = Vec::with_capacity(n);
                for _ in 0..n {
                    w.push(u32_at(dl, off)?);
                    off += 4;
                }
                match op {
                    0x14 => {
                        if let Some(Some(m)) = stack.get(w[0] as usize & 31) {
                            cur = *m;
                        }
                    }
                    0x15 => cur = IDENT,
                    0x1B => {
                        let mut s = IDENT;
                        for i in 0..3 {
                            s[i][i] = fx(w[i] as i32);
                        }
                        cur = mul(&s, &cur);
                    }
                    0x1C => {
                        let mut t = IDENT;
                        for i in 0..3 {
                            t[3][i] = fx(w[i] as i32);
                        }
                        cur = mul(&t, &cur);
                    }
                    0x20 => {
                        col = [
                            expand5(w[0]) as f32 / 255.0,
                            expand5(w[0] >> 5) as f32 / 255.0,
                            expand5(w[0] >> 10) as f32 / 255.0,
                        ];
                        has_color = true;
                    }
                    0x21 => {
                        nrm = [
                            s10(w[0]) as f32 / 512.0,
                            s10(w[0] >> 10) as f32 / 512.0,
                            s10(w[0] >> 20) as f32 / 512.0,
                        ]
                    }
                    0x22 => {
                        uv = [
                            (w[0] as u16 as i16) as f32 / 16.0 / tw,
                            ((w[0] >> 16) as u16 as i16) as f32 / 16.0 / th,
                        ]
                    }
                    0x23 => {
                        pos = [
                            fx(w[0] as u16 as i16 as i32),
                            fx((w[0] >> 16) as u16 as i16 as i32),
                            fx(w[1] as u16 as i16 as i32),
                        ]
                    }
                    0x24 => {
                        pos = [
                            s10(w[0]) as f32 / 64.0,
                            s10(w[0] >> 10) as f32 / 64.0,
                            s10(w[0] >> 20) as f32 / 64.0,
                        ]
                    }
                    0x25..=0x27 => {
                        let a = fx(w[0] as u16 as i16 as i32);
                        let b = fx((w[0] >> 16) as u16 as i16 as i32);
                        match op {
                            0x25 => (pos[0], pos[1]) = (a, b),
                            0x26 => (pos[0], pos[2]) = (a, b),
                            _ => (pos[1], pos[2]) = (a, b),
                        }
                    }
                    0x28 => {
                        pos[0] += s10(w[0]) as f32 / 4096.0;
                        pos[1] += s10(w[0] >> 10) as f32 / 4096.0;
                        pos[2] += s10(w[0] >> 20) as f32 / 4096.0;
                    }
                    0x40 => {
                        vs.clear();
                        mode = w[0] & 3;
                    }
                    0x41 => {
                        self.assemble(mat, mode, &vs, has_color);
                        vs.clear();
                    }
                    _ => {}
                }
                if (0x23..=0x28).contains(&op) {
                    let sp = [pos[0] * ps, pos[1] * ps, pos[2] * ps];
                    vs.push(Vtx {
                        p: xform(&cur, sp),
                        n: xform_dir(&cur, nrm),
                        uv,
                        c: col,
                    });
                }
            }
        }
        Ok(())
    }

    fn assemble(&mut self, mat: usize, mode: u32, v: &[Vtx], hc: bool) {
        let mut idx: Vec<usize> = Vec::new();
        match mode {
            0 => {
                for t in 0..v.len() / 3 {
                    idx.extend([3 * t, 3 * t + 1, 3 * t + 2]);
                }
            }
            1 => {
                for q in 0..v.len() / 4 {
                    let b = 4 * q;
                    idx.extend([b, b + 1, b + 2, b, b + 2, b + 3]);
                }
            }
            2 => {
                for i in 2..v.len() {
                    if i % 2 == 0 {
                        idx.extend([i - 2, i - 1, i]);
                    } else {
                        idx.extend([i - 1, i - 2, i]);
                    }
                }
            }
            _ => {
                let mut i = 0;
                while i + 3 < v.len() {
                    idx.extend([i, i + 1, i + 3, i, i + 3, i + 2]);
                    i += 2;
                }
            }
        }
        self.emit(mat, v, &idx, hc);
    }
}
