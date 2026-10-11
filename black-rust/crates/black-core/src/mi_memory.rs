//! C: include/mi_memory.h (SDK memory routines, implemented with slice operations).

/// C: `MI_CpuCopy8(src, dest, size)`. Copies `size` bytes.
///
/// Panics if either slice is shorter than `size`.
pub fn cpu_copy8(src: &[u8], dest: &mut [u8], size: u32) {
    let n = size as usize;
    dest[..n].copy_from_slice(&src[..n]);
}

/// C: `MI_CpuFill8(dest, data, size)`. Fills `size` bytes with `data`.
///
/// Panics if `dest` is shorter than `size`.
pub fn cpu_fill8(dest: &mut [u8], data: u8, size: u32) {
    dest[..size as usize].fill(data);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_and_fill() {
        let mut d = [0u8; 6];
        cpu_copy8(&[1, 2, 3, 4], &mut d, 3);
        assert_eq!(d, [1, 2, 3, 0, 0, 0]);
        cpu_fill8(&mut d, 0xAA, 4);
        assert_eq!(d, [0xAA, 0xAA, 0xAA, 0xAA, 0, 0]);
        cpu_copy8(&[], &mut d, 0);
        cpu_fill8(&mut d, 1, 0);
        assert_eq!(d[4], 0);
    }

    #[test]
    #[should_panic]
    fn copy_too_long_panics() {
        let mut d = [0u8; 2];
        cpu_copy8(&[1, 2, 3], &mut d, 3);
    }
}
