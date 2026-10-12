use super::*;

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn lz10_literals_and_backref() {
    // "abcabcabc": literals a b c, then ref len 6 disp 3
    let data = [0x10, 9, 0, 0, 0b0001_0000, b'a', b'b', b'c', 0x30, 0x02];
    assert_eq!(decompress(&data).unwrap(), b"abcabcabc");
    assert_eq!(kind(&data), Some(LzKind::Lz10));
}

#[test]
fn lz11_three_forms() {
    // 'a', then short form len 4 disp 1, 3-byte form len 0x11 disp 1, 4-byte form len 0x111 disp 1
    let total = 1 + 4 + 0x11 + 0x111;
    let mut data = vec![0x11, 0, 0, 0, 0b0111_0000, b'a'];
    data[1..4].copy_from_slice(&(total as u32).to_le_bytes()[..3]);
    data.extend_from_slice(&[0x30, 0x00]);
    data.extend_from_slice(&[0x00, 0x00, 0x00]);
    data.extend_from_slice(&[0x10, 0x00, 0x00, 0x00]);
    assert_eq!(decompress(&data).unwrap(), vec![b'a'; total]);
    assert_eq!(decompress_any(&data).unwrap().len(), total);
    assert_eq!(kind(&data), Some(LzKind::Lz11));
}

#[test]
fn extended_size_header() {
    let data = [0x10, 0, 0, 0, 3, 0, 0, 0, 0, b'x', b'y', b'z'];
    assert_eq!(decompress(&data).unwrap(), b"xyz");
}

#[test]
fn rejects_bad_streams() {
    assert!(decompress(&[]).is_err());
    assert!(decompress(&[0x10, 4, 0, 0]).is_err()); // no data
    assert!(decompress(&[0x10, 4, 0, 0, 0x80, 0x00, 0x00]).is_err()); // ref before start
    assert!(decompress(&[0x12, 4, 0, 0, 0, 1, 2, 3, 4]).is_err()); // wrong type
    assert!(decompress_any(b"NARC\x00\x01").is_none());
    assert!(decompress_any(&[0x10, 1, 0, 0, 0, b'a', 0, 0, 0, 0, 0]).is_none());
    // long tail
}

#[test]
fn lz10_round_trip() {
    let mut s = 0x1234_5678_9abc_def0u64;
    for n in [0usize, 1, 2, 17, 300, 5000] {
        let data: Vec<u8> = (0..n).map(|_| (xorshift(&mut s) % 7) as u8).collect();
        let packed = compress_lz10(&data).unwrap();
        assert_eq!(decompress(&packed).unwrap(), data);
    }
}

#[test]
fn garbage_never_panics() {
    let mut s = 0xdead_beef_cafe_f00du64;
    for i in 0..20000 {
        let n = (xorshift(&mut s) % 64) as usize;
        let mut buf: Vec<u8> = (0..n).map(|_| xorshift(&mut s) as u8).collect();
        if i % 2 == 0 && !buf.is_empty() {
            buf[0] = 0x10 + (i / 2 % 2) as u8;
        }
        let _ = decompress(&buf);
        let _ = decompress_any(&buf);
    }
}
