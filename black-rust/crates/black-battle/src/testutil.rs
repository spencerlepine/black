//! Shared helpers for the differential tests. The generator and hash mirror the emulator
//! harness that runs the original machine code, so both sides see identical inputs.

/// 64-bit FNV-1a over little-endian words.
pub struct Fnv(pub u64);

impl Fnv {
    pub fn new() -> Self {
        Fnv(14695981039346656037)
    }

    pub fn b(&mut self, v: u8) {
        self.0 ^= v as u64;
        self.0 = self.0.wrapping_mul(1099511628211);
    }

    pub fn w(&mut self, v: u32) {
        for x in v.to_le_bytes() {
            self.b(x);
        }
    }

    pub fn hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

/// xorshift32 with the same seeding as black-core's differential tests.
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        let s = seed.wrapping_mul(2654435761).wrapping_add(1);
        Rng(if s == 0 { 1 } else { s })
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}
