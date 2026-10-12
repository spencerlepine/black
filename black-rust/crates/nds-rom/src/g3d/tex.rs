//! `TEX0` block (textures + palettes), shared by `BMD0` and `BTX0` files.

use super::{bad, blocks, u16_at, u32_at, u8_at, Dict};
use crate::error::Result;
use crate::render::Image;

#[derive(Debug, Clone)]
pub struct Texture {
    pub name: String,
    /// 1 A3I5, 2 4-colour, 3 16-colour, 4 256-colour, 5 4x4 compressed, 6 A5I3, 7 direct colour.
    pub format: u8,
    pub width: usize,
    pub height: usize,
    /// Palette index 0 is transparent (formats 2-4).
    pub color0_transparent: bool,
    data: Vec<u8>,
    /// Format 5 only: one u16 per 4x4 block (palette offset + mode).
    index: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    /// Raw RGB555 colours.
    pub colors: Vec<u16>,
}

#[derive(Debug, Clone, Default)]
pub struct Tex0 {
    pub textures: Vec<Texture>,
    pub palettes: Vec<Palette>,
}

fn rgb(c: u16) -> [u8; 3] {
    let f = |v: u16| {
        let v = (v & 31) as u8;
        (v << 3) | (v >> 2)
    };
    [f(c), f(c >> 5), f(c >> 10)]
}

impl Tex0 {
    /// Parse a standalone `BTX0` file (NSBTX).
    pub fn parse_btx0(d: &[u8]) -> Result<Tex0> {
        for (tag, off) in blocks(d, b"BTX0", "BTX0")? {
            if &tag == b"TEX0" {
                return Tex0::parse_block(d, off);
            }
        }
        Err(bad("BTX0", "no TEX0 block"))
    }

    /// Parse the `TEX0` block at absolute offset `b` of `d`.
    pub fn parse_block(d: &[u8], b: usize) -> Result<Tex0> {
        let tex_dict = Dict::parse(d, b + u16_at(d, b + 0x0E)? as usize)?;
        let tex_data = b + u32_at(d, b + 0x14)? as usize;
        let comp_data = b + u32_at(d, b + 0x24)? as usize;
        let comp_info = b + u32_at(d, b + 0x28)? as usize;
        let pal_dict = Dict::parse(d, b + u32_at(d, b + 0x34)? as usize)?;
        let pal_data = b + u32_at(d, b + 0x38)? as usize;
        let block_end = b + u32_at(d, b + 4)? as usize;

        let mut textures = Vec::new();
        for (i, name) in tex_dict.names.iter().enumerate() {
            let e = tex_dict.entry(i);
            let off = u16_at(d, e)? as usize * 8;
            let param = u16_at(d, e + 2)?;
            let width = 8usize << ((param >> 4) & 7);
            let height = 8usize << ((param >> 7) & 7);
            let format = ((param >> 10) & 7) as u8;
            let color0_transparent = param & 0x2000 != 0;
            let bits = [0, 8, 2, 4, 8, 2, 8, 16][format as usize];
            let len = width * height * bits / 8;
            let (data, index) = if format == 5 {
                let s = comp_data + off;
                let ix = comp_info + off / 2;
                (
                    d.get(s..s + len).unwrap_or(&[]).to_vec(),
                    d.get(ix..ix + len / 8).unwrap_or(&[]).to_vec(),
                )
            } else {
                let s = tex_data + off;
                (d.get(s..s + len).unwrap_or(&[]).to_vec(), Vec::new())
            };
            textures.push(Texture {
                name: name.clone(),
                format,
                width,
                height,
                color0_transparent,
                data,
                index,
            });
        }

        let mut offs = Vec::new();
        for i in 0..pal_dict.names.len() {
            offs.push(u16_at(d, pal_dict.entry(i))? as usize * 8);
        }
        let mut palettes = Vec::new();
        for (i, name) in pal_dict.names.iter().enumerate() {
            let start = pal_data + offs[i];
            // A palette ends where the next one (by offset) starts, or at the block end.
            let end = offs
                .iter()
                .filter(|&&o| o > offs[i])
                .min()
                .map_or(block_end, |&o| pal_data + o)
                .min(d.len());
            let bytes = d.get(start..end.max(start)).unwrap_or(&[]);
            let colors = (0..bytes.len() / 2)
                .map(|i| u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]]))
                .collect();
            palettes.push(Palette {
                name: name.clone(),
                colors,
            });
        }
        let _ = u8_at(d, b)?;
        Ok(Tex0 { textures, palettes })
    }

    pub fn texture(&self, name: &str) -> Option<usize> {
        self.textures.iter().position(|t| t.name == name)
    }
    pub fn palette(&self, name: &str) -> Option<usize> {
        self.palettes.iter().position(|p| p.name == name)
    }

    /// Guess the palette of texture `ti` when no material says (same name, `<name>_pl`, or the
    /// only palette).
    pub fn palette_for(&self, ti: usize) -> Option<usize> {
        let n = &self.textures.get(ti)?.name;
        self.palette(n)
            .or_else(|| self.palette(&format!("{n}_pl")))
            .or((self.palettes.len() == 1).then_some(0))
    }

    /// Decode texture `ti` with palette `pi` (needed for every format except 7) to RGBA.
    pub fn decode(&self, ti: usize, pi: Option<usize>) -> Result<Image> {
        let t = self
            .textures
            .get(ti)
            .ok_or_else(|| bad("TEX0", "texture index"))?;
        let pal: &[u16] = match pi {
            Some(p) => {
                &self
                    .palettes
                    .get(p)
                    .ok_or_else(|| bad("TEX0", "palette index"))?
                    .colors
            }
            None => &[],
        };
        if t.format != 7 && pal.is_empty() {
            return Err(bad("TEX0", format!("texture {} needs a palette", t.name)));
        }
        let (w, h) = (t.width, t.height);
        let mut img = Image::new(w, h);
        let col = |i: usize| pal.get(i).copied().unwrap_or(0);
        let mut put = |x: usize, y: usize, px: [u8; 4]| {
            let o = (y * w + x) * 4;
            img.rgba[o..o + 4].copy_from_slice(&px);
        };
        let need = |n: usize| {
            if t.data.len() < n {
                Err(bad("TEX0", format!("texture {} data truncated", t.name)))
            } else {
                Ok(())
            }
        };
        match t.format {
            1 | 6 => {
                need(w * h)?;
                let (ib, ab) = if t.format == 1 { (5, 3) } else { (3, 5) };
                for i in 0..w * h {
                    let v = t.data[i] as usize;
                    let idx = v & ((1 << ib) - 1);
                    let a = (v >> ib) & ((1 << ab) - 1);
                    let a8 = (a * 255 / ((1 << ab) - 1)) as u8;
                    let c = rgb(col(idx));
                    put(i % w, i / w, [c[0], c[1], c[2], a8]);
                }
            }
            2..=4 => {
                let bpp = [2, 4, 8][t.format as usize - 2];
                need(w * h * bpp / 8)?;
                for i in 0..w * h {
                    let bit = i * bpp;
                    let idx = (t.data[bit / 8] as usize >> (bit % 8)) & ((1 << bpp) - 1);
                    let c = rgb(col(idx));
                    let a = if idx == 0 && t.color0_transparent {
                        0
                    } else {
                        255
                    };
                    put(i % w, i / w, [c[0], c[1], c[2], a]);
                }
            }
            5 => {
                need(w * h / 4)?;
                if t.index.len() < w * h / 32 {
                    return Err(bad("TEX0", "compressed index data truncated"));
                }
                let bw = w / 4;
                for blk in 0..bw * (h / 4) {
                    let ix = u16::from_le_bytes([t.index[blk * 2], t.index[blk * 2 + 1]]) as usize;
                    let base = (ix & 0x3FFF) * 2;
                    let mode = ix >> 14;
                    let p = |i: usize| rgb(col(base + i));
                    let mix = |a: [u8; 3], b: [u8; 3], wa: u32, wb: u32| {
                        let m = |i: usize| ((a[i] as u32 * wa + b[i] as u32 * wb) / 8) as u8;
                        [m(0), m(1), m(2)]
                    };
                    let (c0, c1) = (p(0), p(1));
                    let table: [[u8; 4]; 4] = match mode {
                        0 => {
                            let o = |c: [u8; 3]| [c[0], c[1], c[2], 255];
                            [o(p(0)), o(p(1)), o(p(2)), [0, 0, 0, 0]]
                        }
                        1 => {
                            let m = mix(c0, c1, 4, 4);
                            [
                                [c0[0], c0[1], c0[2], 255],
                                [c1[0], c1[1], c1[2], 255],
                                [m[0], m[1], m[2], 255],
                                [0, 0, 0, 0],
                            ]
                        }
                        2 => [p(0), p(1), p(2), p(3)].map(|c| [c[0], c[1], c[2], 255]),
                        _ => {
                            let a = mix(c0, c1, 5, 3);
                            let b = mix(c0, c1, 3, 5);
                            [
                                [c0[0], c0[1], c0[2], 255],
                                [c1[0], c1[1], c1[2], 255],
                                [a[0], a[1], a[2], 255],
                                [b[0], b[1], b[2], 255],
                            ]
                        }
                    };
                    let (bx, by) = (blk % bw, blk / bw);
                    for y in 0..4 {
                        let row = t.data[blk * 4 + y];
                        for x in 0..4 {
                            let c = table[(row >> (x * 2)) as usize & 3];
                            put(bx * 4 + x, by * 4 + y, c);
                        }
                    }
                }
            }
            7 => {
                need(w * h * 2)?;
                for i in 0..w * h {
                    let c = u16::from_le_bytes([t.data[i * 2], t.data[i * 2 + 1]]);
                    let r = rgb(c);
                    put(
                        i % w,
                        i / w,
                        [r[0], r[1], r[2], if c & 0x8000 != 0 { 255 } else { 0 }],
                    );
                }
            }
            f => return Err(bad("TEX0", format!("texture format {f}"))),
        }
        Ok(img)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tex(format: u8, w: usize, h: usize, data: Vec<u8>, index: Vec<u8>) -> Tex0 {
        Tex0 {
            textures: vec![Texture {
                name: "t".into(),
                format,
                width: w,
                height: h,
                color0_transparent: false,
                data,
                index,
            }],
            palettes: vec![Palette {
                name: "p".into(),
                colors: vec![0x001F, 0x03E0, 0x7C00, 0x7FFF],
            }],
        }
    }

    #[test]
    fn direct_colour_and_alpha() {
        let t = tex(7, 8, 8, [0x1Fu8, 0x80].repeat(64), vec![]);
        let img = t.decode(0, None).unwrap();
        assert_eq!(img.pixel(3, 3), [255, 0, 0, 255]);
    }

    #[test]
    fn a3i5_alpha_scaling() {
        // index 2 (green? colours: 0 red, 1 green, 2 blue), alpha 7 of 7
        let t = tex(1, 8, 8, vec![(7 << 5) | 2; 64], vec![]);
        assert_eq!(t.decode(0, Some(0)).unwrap().pixel(0, 0), [0, 0, 255, 255]);
    }

    #[test]
    fn four_colour_packing() {
        // texels 0..3 of the first byte use indices 0,1,2,3 (LSB first)
        let mut d = vec![0u8; 8 * 8 / 4];
        d[0] = 0b11_10_01_00;
        let img = tex(2, 8, 8, d, vec![]).decode(0, Some(0)).unwrap();
        assert_eq!(img.pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(img.pixel(1, 0), [0, 255, 0, 255]);
        assert_eq!(img.pixel(3, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn compressed_4x4_modes() {
        // 8x8 = 4 blocks. Block 0: mode 0 (transparent index 3); block 1: mode 1 (average).
        let mut d = vec![0u8; 16];
        d[0] = 0b11_10_01_00;
        d[4] = 0b10_10_10_10;
        let idx = [0x00, 0x00, 0x00, 0x40, 0, 0, 0, 0];
        let img = tex(5, 8, 8, d, idx.to_vec()).decode(0, Some(0)).unwrap();
        assert_eq!(img.pixel(3, 0)[3], 0);
        assert_eq!(img.pixel(1, 0), [0, 255, 0, 255]);
        let avg = img.pixel(4, 0);
        assert_eq!(avg[3], 255);
        assert_eq!(avg[1], 127);
    }
}
