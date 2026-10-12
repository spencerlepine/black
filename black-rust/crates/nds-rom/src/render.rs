//! Rendering helpers: tiles + palette (+ screen or cell) -> RGBA images, and a PNG encoder.

use crate::error::{Error, Result};
use crate::ncer::Ncer;
use crate::ncgr::Ncgr;
use crate::nclr::Nclr;
use crate::nscr::Nscr;
use flate2::{write::ZlibEncoder, Compression};
use std::io::Write;

/// An RGBA8 image, row-major, 4 bytes per pixel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        Image {
            width,
            height,
            rgba: vec![0; width * height * 4],
        }
    }

    pub fn pixel(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * self.width + x) * 4;
        self.rgba[i..i + 4].try_into().unwrap()
    }

    fn put(&mut self, x: i32, y: i32, px: [u8; 4]) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            let i = (y as usize * self.width + x as usize) * 4;
            self.rgba[i..i + 4].copy_from_slice(&px);
        }
    }

    /// Fraction of pixels that are not fully transparent.
    pub fn opaque_pixels(&self) -> usize {
        self.rgba.chunks(4).filter(|p| p[3] != 0).count()
    }

    /// Encode as an 8-bit RGBA PNG.
    pub fn to_png(&self) -> Result<Vec<u8>> {
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&(self.width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.height as u32).to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        let mut raw = Vec::with_capacity((self.width * 4 + 1) * self.height);
        for row in self.rgba.chunks(self.width.max(1) * 4).take(self.height) {
            raw.push(0);
            raw.extend_from_slice(row);
        }
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&raw).map_err(|e| Error::Png(e.to_string()))?;
        let idat = enc.finish().map_err(|e| Error::Png(e.to_string()))?;
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        png_chunk(&mut out, b"IHDR", &ihdr);
        png_chunk(&mut out, b"IDAT", &idat);
        png_chunk(&mut out, b"IEND", &[]);
        Ok(out)
    }
}

fn crc32(parts: &[&[u8]]) -> u32 {
    let mut crc = !0u32;
    for part in parts {
        for &b in *part {
            crc ^= b as u32;
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xEDB8_8320 & (!(crc & 1)).wrapping_add(1));
            }
        }
    }
    !crc
}

fn png_chunk(out: &mut Vec<u8>, tag: &[u8; 4], payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(payload);
    out.extend_from_slice(&crc32(&[tag, payload]).to_be_bytes());
}

/// Colour `index` of `row` in `pal`. For 8bpp graphics pass `bpp8` so the row is ignored and the
/// index addresses the whole palette.
fn colour(pal: &Nclr, row: usize, index: u8, bpp8: bool) -> [u8; 4] {
    if index == 0 {
        return [0, 0, 0, 0];
    }
    let i = if bpp8 {
        index as usize
    } else {
        row * 16 + index as usize
    };
    crate::nclr::bgr555_to_rgba(pal.colors.get(i).copied().unwrap_or(0), true)
}

/// Render a whole character file (see [`Ncgr::to_indices`]) with palette row `row`.
pub fn render_tiles(ncgr: &Ncgr, pal: &Nclr, row: usize) -> Image {
    let (w, h, idx) = ncgr.to_indices();
    let mut img = Image::new(w, h);
    for (i, &p) in idx.iter().enumerate() {
        let c = colour(pal, row, p, ncgr.bpp == 8);
        img.rgba[i * 4..i * 4 + 4].copy_from_slice(&c);
    }
    img
}

/// Render a tilemap using tiles from `ncgr` (tiled mode) and palette `pal`.
pub fn render_screen(ncgr: &Ncgr, pal: &Nclr, scr: &Nscr) -> Image {
    let (w, h) = (scr.width as usize, scr.height as usize);
    let mut img = Image::new(w, h);
    let tx = (w / 8).max(1);
    for (n, e) in scr.entries.iter().enumerate() {
        let Some(px) = ncgr.tile_pixels(e.tile as usize) else {
            continue;
        };
        let (bx, by) = ((n % tx) * 8, (n / tx) * 8);
        for y in 0..8usize {
            for x in 0..8usize {
                let sx = if e.hflip { 7 - x } else { x };
                let sy = if e.vflip { 7 - y } else { y };
                let c = colour(pal, e.palette as usize, px[sy * 8 + sx], ncgr.bpp == 8);
                img.put((bx + x) as i32, (by + y) as i32, c);
            }
        }
    }
    img
}

/// Render cell `cell` of a bank (1D OBJ mapping) to an image sized to the cell's bounds.
/// The returned origin is the position of the image's top-left pixel in cell coordinates.
pub fn render_cell(
    ncer: &Ncer,
    cell: usize,
    ncgr: &Ncgr,
    pal: &Nclr,
) -> Option<(Image, (i32, i32))> {
    let cell = ncer.cells.get(cell)?;
    let objs: Vec<_> = cell.oams.iter().filter(|o| !o.disabled).collect();
    if objs.is_empty() {
        return Some((Image::new(1, 1), (0, 0)));
    }
    // Unrotated drawing of a double-size box centres the sprite in it.
    let pos = |o: &crate::ncer::Oam| {
        let d = o.double_size as i32;
        (
            o.x as i32 + d * o.w as i32 / 2,
            o.y as i32 + d * o.h as i32 / 2,
        )
    };
    let min_x = objs.iter().map(|o| pos(o).0).min()?;
    let min_y = objs.iter().map(|o| pos(o).1).min()?;
    let max_x = objs.iter().map(|o| pos(o).0 + o.w as i32).max()?;
    let max_y = objs.iter().map(|o| pos(o).1 + o.h as i32).max()?;
    let mut img = Image::new((max_x - min_x) as usize, (max_y - min_y) as usize);
    let tb = ncgr.tile_bytes();
    // Draw back to front: earlier OAM entries have higher priority.
    for o in objs.iter().rev() {
        let (tw, th) = (o.w as usize / 8, o.h as usize / 8);
        let bpp8 = o.bpp8 || ncgr.bpp == 8;
        for ty in 0..th {
            for tx in 0..tw {
                let t = if ncer.mapping_mode >= 4 {
                    // 2D mapping: tiles are addressed on a grid as wide as the stored image.
                    o.tile as usize + ty * ncgr.row_tiles() + tx
                } else {
                    (o.tile as usize) * (32usize << ncer.mapping_mode) / tb + ty * tw + tx
                };
                let Some(px) = ncgr.tile_pixels(t) else {
                    continue;
                };
                for y in 0..8usize {
                    for x in 0..8usize {
                        let (ox, oy) = (tx * 8 + x, ty * 8 + y);
                        let dx = if o.hflip { o.w as usize - 1 - ox } else { ox };
                        let dy = if o.vflip { o.h as usize - 1 - oy } else { oy };
                        let i = px[y * 8 + x];
                        if i == 0 {
                            continue;
                        }
                        let c = colour(pal, o.palette as usize, i, bpp8);
                        let (px0, py0) = pos(o);
                        img.put(px0 - min_x + dx as i32, py0 - min_y + dy as i32, c);
                    }
                }
            }
        }
    }
    Some((img, (min_x, min_y)))
}
