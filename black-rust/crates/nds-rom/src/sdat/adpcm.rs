//! IMA-ADPCM as used by the DS sound hardware (SWAV type 2, STRM type 2).
//!
//! A block starts with a 4-byte header: `s16 predictor`, `u8 step index` (+1 pad). Then 4-bit
//! samples follow, low nibble first. Each decoded sample is the predictor after the update.

const STEP: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];
const INDEX: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// Decoder state (predictor, step index).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub pred: i32,
    pub index: i32,
}

impl State {
    pub fn from_header(h: &[u8]) -> State {
        State {
            pred: i32::from(i16::from_le_bytes([h[0], h[1]])),
            index: i32::from(h[2]).min(88),
        }
    }

    fn step(&mut self, nibble: u8) -> i16 {
        let step = STEP[self.index as usize];
        let n = i32::from(nibble);
        let mut diff = step >> 3;
        if n & 1 != 0 {
            diff += step >> 2;
        }
        if n & 2 != 0 {
            diff += step >> 1;
        }
        if n & 4 != 0 {
            diff += step;
        }
        self.pred = if n & 8 != 0 {
            self.pred - diff
        } else {
            self.pred + diff
        }
        .clamp(-32768, 32767);
        self.index = (self.index + INDEX[(n & 7) as usize]).clamp(0, 88);
        self.pred as i16
    }
}

/// Decode a block (4-byte header + nibbles) into `2 * (len - 4)` samples.
pub fn decode_block(block: &[u8]) -> Vec<i16> {
    if block.len() < 4 {
        return Vec::new();
    }
    let mut st = State::from_header(block);
    let mut out = Vec::with_capacity((block.len() - 4) * 2);
    for &b in &block[4..] {
        out.push(st.step(b & 15));
        out.push(st.step(b >> 4));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_values() {
        let s = decode_block(&[100, 0, 0, 0, 0x70, 0xF0]);
        // nibble 0 (index 0, step 7): diff 0 -> 100, index stays 0.
        // nibble 7: diff 0+1+3+7 = 11 -> 111, index 8 (step 16).
        // nibble 0: diff 2 -> 113, index 7 (step 14).
        // nibble 0xF: diff 1+3+7+14 = 25, negative -> 88.
        assert_eq!(s, vec![100, 111, 113, 88]);
    }

    #[test]
    fn clamps_and_short() {
        assert!(decode_block(&[1, 2]).is_empty());
        assert_eq!(decode_block(&[0xFF, 0x7F, 88, 0, 0x77]), vec![32767, 32767]);
        assert_eq!(
            decode_block(&[0x00, 0x80, 88, 0, 0xFF]),
            vec![-32768, -32768]
        );
    }
}
