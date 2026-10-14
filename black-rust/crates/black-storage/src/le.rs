//! Little-endian field access for the block images (`from_bytes` / `to_bytes` of the save blocks).

pub(crate) fn r16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}

pub(crate) fn r32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

pub(crate) fn w16(b: &mut [u8], o: usize, v: u16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

pub(crate) fn w32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// `n` little-endian halfwords from `o`.
pub(crate) fn r16s<const N: usize>(b: &[u8], o: usize) -> [u16; N] {
    std::array::from_fn(|i| r16(b, o + 2 * i))
}

pub(crate) fn w16s(b: &mut [u8], o: usize, v: &[u16]) {
    for (i, &h) in v.iter().enumerate() {
        w16(b, o + 2 * i, h);
    }
}

/// `n` little-endian words from `o`.
pub(crate) fn r32s<const N: usize>(b: &[u8], o: usize) -> [u32; N] {
    std::array::from_fn(|i| r32(b, o + 4 * i))
}

pub(crate) fn w32s(b: &mut [u8], o: usize, v: &[u32]) {
    for (i, &w) in v.iter().enumerate() {
        w32(b, o + 4 * i, w);
    }
}
