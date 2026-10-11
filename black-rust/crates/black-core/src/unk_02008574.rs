//! C: src/unk_02008574.c

/// Offsets written by [`set_bytes_18_19`].
pub const OFFSET_A: usize = 0x18;
pub const OFFSET_B: usize = 0x19;

/// C: `FUN_02008574 @ 0x02008574` - stores `a` at +0x18 and `b` at +0x19 of an object.
///
/// Panics if `obj` is shorter than 0x1A bytes.
pub fn set_bytes_18_19(obj: &mut [u8], a: u8, b: u8) {
    obj[OFFSET_A] = a;
    obj[OFFSET_B] = b;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_only_two_bytes() {
        let mut o = [0x11u8; 0x20];
        set_bytes_18_19(&mut o, 0xAB, 0xCD);
        for (i, &v) in o.iter().enumerate() {
            match i {
                0x18 => assert_eq!(v, 0xAB),
                0x19 => assert_eq!(v, 0xCD),
                _ => assert_eq!(v, 0x11),
            }
        }
    }

    #[test]
    fn minimum_size() {
        let mut o = [0u8; 0x1A];
        set_bytes_18_19(&mut o, 1, 2);
        assert_eq!(&o[0x18..], &[1, 2]);
    }

    #[test]
    #[should_panic]
    fn too_small_panics() {
        let mut o = [0u8; 0x19];
        set_bytes_18_19(&mut o, 1, 2);
    }
}
