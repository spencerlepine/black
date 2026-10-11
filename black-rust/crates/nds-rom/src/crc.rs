//! CRC-16 (the "MODBUS" flavour: polynomial 0xA001, init 0xFFFF) used throughout NDS headers.

/// Compute the NDS CRC-16 of `data` with the standard initial value.
pub fn crc16(data: &[u8]) -> u16 {
    crc16_with(0xFFFF, data)
}

/// Compute the NDS CRC-16 of `data` starting from `init`.
pub fn crc16_with(init: u16, data: &[u8]) -> u16 {
    let mut crc = init;
    for &byte in data {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xA001
            } else {
                crc >> 1
            };
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        // CRC-16/MODBUS check value
        assert_eq!(crc16(b"123456789"), 0x4B37);
        assert_eq!(crc16(&[]), 0xFFFF);
    }
}
