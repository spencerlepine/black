//! Banner icon <-> PNG conversion (port of `banner.py`).

use crate::error::{create_dir, read_file, write_file, Error, Result};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression, Crc};
use std::io::{Read, Write};
use std::path::Path;

const ICON: usize = 0x0020;
const PALETTE: usize = 0x0220;
const ANIM: usize = 0x1240;
const FRAME: usize = 0x200;
const FRAMES: usize = 8;
const TILE: usize = 8;
const SIDE: usize = 32;

/// Banner size in bytes for a given banner version.
pub fn banner_size(version: u16) -> usize {
    match version {
        0x0103 => 0x23C0,
        2 => 0x940,
        3 => 0xA40,
        _ => 0x840,
    }
}

type Pixels = [[u8; SIDE]; SIDE];

fn unswizzle(bitmap: &[u8]) -> Pixels {
    let mut px = [[0u8; SIDE]; SIDE];
    for tile in 0..16 {
        let (ty, tx) = (tile / (SIDE / TILE), tile % (SIDE / TILE));
        for index in 0..TILE * TILE / 2 {
            let byte = bitmap[tile * 32 + index];
            for half in 0..2 {
                let (y, x) = ((index * 2 + half) / TILE, (index * 2 + half) % TILE);
                px[ty * TILE + y][tx * TILE + x] = (byte >> (4 * half)) & 0xF;
            }
        }
    }
    px
}

fn swizzle(px: &Pixels) -> [u8; FRAME] {
    let mut out = [0u8; FRAME];
    for tile in 0..16 {
        let (ty, tx) = (tile / (SIDE / TILE), tile % (SIDE / TILE));
        for index in 0..TILE * TILE / 2 {
            let mut value = 0;
            for half in 0..2 {
                let (y, x) = ((index * 2 + half) / TILE, (index * 2 + half) % TILE);
                value |= (px[ty * TILE + y][tx * TILE + x] & 0xF) << (4 * half);
            }
            out[tile * 32 + index] = value;
        }
    }
    out
}

fn rgb(entry: u16) -> [u8; 3] {
    let c = |shift: u32| (((entry >> shift) & 0x1F) as u32 * 255 / 31) as u8;
    [c(0), c(5), c(10)]
}

fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut crc = Crc::new();
    crc.update(tag);
    crc.update(payload);
    let mut out = (payload.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(tag);
    out.extend_from_slice(payload);
    out.extend_from_slice(&crc.sum().to_be_bytes());
    out
}

fn encode_png(px: &Pixels, palette: &[u16; 16]) -> Result<Vec<u8>> {
    let mut header = Vec::new();
    header.extend_from_slice(&(SIDE as u32).to_be_bytes());
    header.extend_from_slice(&(SIDE as u32).to_be_bytes());
    header.extend_from_slice(&[8, 3, 0, 0, 0]);
    let plte: Vec<u8> = palette.iter().flat_map(|&e| rgb(e)).collect();
    let mut trns = vec![0u8];
    trns.resize(palette.len(), 0xFF);
    let mut raw = Vec::new();
    for row in px {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(&raw).map_err(|e| Error::Png(e.to_string()))?;
    let idat = enc.finish().map_err(|e| Error::Png(e.to_string()))?;
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    out.extend(chunk(b"IHDR", &header));
    out.extend(chunk(b"PLTE", &plte));
    out.extend(chunk(b"tRNS", &trns));
    out.extend(chunk(b"IDAT", &idat));
    out.extend(chunk(b"IEND", &[]));
    Ok(out)
}

fn decode_png(data: &[u8]) -> Result<Pixels> {
    if data.len() < 8 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(Error::Png("not a PNG".into()));
    }
    let mut off = 8;
    let mut idat = Vec::new();
    while off + 8 <= data.len() {
        let len =
            u32::from_be_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        let tag = &data[off + 4..off + 8];
        let payload = data
            .get(off + 8..off + 8 + len)
            .ok_or_else(|| Error::Png("truncated chunk".into()))?;
        if tag == b"IHDR" && (len != 13 || payload[8..10] != [8, 3]) {
            return Err(Error::Png("expected an 8-bit palette image".into()));
        }
        if tag == b"IDAT" {
            idat.extend_from_slice(payload);
        }
        off += 12 + len;
    }
    let mut flat = Vec::new();
    ZlibDecoder::new(&idat[..])
        .read_to_end(&mut flat)
        .map_err(|e| Error::Png(e.to_string()))?;
    if flat.len() != SIDE * (SIDE + 1) {
        return Err(Error::Png(format!(
            "unexpected pixel data size {}",
            flat.len()
        )));
    }
    let mut px = [[0u8; SIDE]; SIDE];
    for (y, row) in px.iter_mut().enumerate() {
        if flat[y * (SIDE + 1)] != 0 {
            return Err(Error::Png("PNG row filters are not supported".into()));
        }
        row.copy_from_slice(&flat[y * (SIDE + 1) + 1..(y + 1) * (SIDE + 1)]);
    }
    Ok(px)
}

/// `(name, bitmap offset, palette offset)` of the icon and every animation frame.
fn frames() -> Vec<(String, usize, usize)> {
    let mut v = vec![("icon".to_string(), ICON, PALETTE)];
    for i in 0..FRAMES {
        v.push((
            format!("frame{i}"),
            ANIM + i * FRAME,
            ANIM + FRAMES * FRAME + i * 32,
        ));
    }
    v
}

fn palette_at(banner: &[u8], off: usize) -> Result<[u16; 16]> {
    let mut p = [0u16; 16];
    for (i, c) in p.iter_mut().enumerate() {
        let b = banner
            .get(off + i * 2..off + i * 2 + 2)
            .ok_or(Error::Truncated {
                what: "banner",
                offset: off,
                need: 32,
                have: banner.len().saturating_sub(off),
            })?;
        *c = u16::from_le_bytes([b[0], b[1]]);
    }
    Ok(p)
}

/// Write the icon and animation frames as PNGs plus `banner.meta` (everything else).
/// Returns the number of PNGs written.
pub fn extract(banner: &[u8], dir: &Path) -> Result<usize> {
    if banner.len() < ANIM + FRAMES * FRAME {
        return Err(Error::invalid(
            "banner",
            "no animated icon area (need a DSi banner)",
        ));
    }
    create_dir(dir)?;
    let mut count = 0;
    for (name, bitmap, palette) in frames() {
        let png = encode_png(
            &unswizzle(&banner[bitmap..bitmap + FRAME]),
            &palette_at(banner, palette)?,
        )?;
        write_file(dir.join(format!("{name}.png")), &png)?;
        count += 1;
    }
    let mut meta = banner[..ICON].to_vec();
    meta.extend_from_slice(&banner[PALETTE..ANIM]);
    meta.extend_from_slice(&banner[ANIM + FRAMES * FRAME..]);
    write_file(dir.join("banner.meta"), &meta)?;
    Ok(count)
}

/// Rebuild a banner from `banner.meta` and the PNGs in `dir`.
pub fn build(dir: &Path) -> Result<Vec<u8>> {
    let meta = read_file(dir.join("banner.meta"))?;
    let tail = ANIM + FRAMES * FRAME;
    let min_meta = ICON + (ANIM - PALETTE);
    if meta.len() < min_meta {
        return Err(Error::invalid("banner.meta", "too short"));
    }
    let mut banner = vec![0u8; tail + meta.len() - min_meta];
    banner[..ICON].copy_from_slice(&meta[..ICON]);
    banner[PALETTE..ANIM].copy_from_slice(&meta[ICON..min_meta]);
    banner[tail..].copy_from_slice(&meta[min_meta..]);
    for (name, bitmap, _) in frames() {
        let px = decode_png(&read_file(dir.join(format!("{name}.png")))?)?;
        banner[bitmap..bitmap + FRAME].copy_from_slice(&swizzle(&px));
    }
    Ok(banner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swizzle_roundtrip() {
        let mut px = [[0u8; SIDE]; SIDE];
        for (y, row) in px.iter_mut().enumerate() {
            for (x, p) in row.iter_mut().enumerate() {
                *p = ((x * 3 + y * 5) & 0xF) as u8;
            }
        }
        assert_eq!(unswizzle(&swizzle(&px)), px);
        let png = encode_png(&px, &[0x7FFF; 16]).unwrap();
        assert_eq!(decode_png(&png).unwrap(), px);
    }

    #[test]
    fn sizes() {
        assert_eq!(banner_size(0x0103), 0x23C0);
        assert_eq!(banner_size(1), 0x840);
    }
}
