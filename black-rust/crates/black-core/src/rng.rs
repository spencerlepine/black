//! Game RNG.
//!
//! The battle RNG is the SDK 64-bit LCG, verified through `black-battle::calc` (`021d78ac`).
//! `gf_standard.c` RNGs (Z15): the main ARM9 Mersenne Twister (`Mt19937`, IVs/PIDs), the 64-bit
//! LCG context seeding (`seed_lcg64`) and `next_scaled`, the byte sum, and the LCG32 helpers.
//! Verified per function with `tools/check-fn.sh black-core rng`
//! (spec `tools/rom-diff/specs/black-core/rng.spec`).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_56EC,
    0x0200_5728,
    0x0203_F034,
    0x0203_F0A8,
    0x0203_F258,
    0x0203_F2FC,
    0x0203_F328,
    0x0203_F388,
];

/// SDK `MATHRandContext32`: 64-bit LCG `x = x * mul + add`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MathRandContext32 {
    pub x: u64,
    pub mul: u64,
    pub add: u64,
}

impl MathRandContext32 {
    pub const SIZE: usize = 24;

    pub fn from_bytes(b: &[u8; 24]) -> Self {
        let q = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
        MathRandContext32 {
            x: q(0),
            mul: q(8),
            add: q(16),
        }
    }

    pub fn to_bytes(&self) -> [u8; 24] {
        let mut out = [0u8; 24];
        out[0..8].copy_from_slice(&self.x.to_le_bytes());
        out[8..16].copy_from_slice(&self.mul.to_le_bytes());
        out[16..24].copy_from_slice(&self.add.to_le_bytes());
        out
    }

    /// C: `FUN_020056EC` (on the global context at `*(*0x020AA194 + 0x10)`): advances the state; returns the high
    /// word scaled to `[0, max)`, or the raw high word when `max == 0`.
    pub fn next(&mut self, max: u32) -> u32 {
        self.x = self.mul.wrapping_mul(self.x).wrapping_add(self.add);
        let hi = (self.x >> 32) as u32;
        if max == 0 {
            hi
        } else {
            ((hi as u64 * max as u64) >> 32) as u32
        }
    }
}

impl MathRandContext32 {
    /// C: `FUN_02005728`: like [`next`](Self::next) but `max == 0` also scales (result 0).
    pub fn next_scaled(&mut self, max: u32) -> u32 {
        self.x = self.mul.wrapping_mul(self.x).wrapping_add(self.add);
        (((self.x >> 32) * max as u64) >> 32) as u32
    }
}

// `FUN_020056EC` is `MathRandContext32::next` (the global context at `*(*0x020AA194 + 0x10)`).

/// Multiplier of the game's 64-bit LCG context (`FUN_0203F258` stores it at +8).
pub const LCG64_MUL: u64 = 0x5D58_8B65_6C07_8965;
/// Increment of the game's 64-bit LCG context (`FUN_0203F258` stores it at +0x10).
pub const LCG64_ADD: u64 = 0x0026_9EC3;

/// The SDK calls `FUN_0203F258` makes to turn hardware entropy into a seed. Unported callees.
pub trait SeedHost {
    /// `FUN_02088168(buf)`: fills 32 bytes of hardware-derived entropy (RTC, ticks, MAC...).
    fn read_entropy(&mut self, out: &mut [u8; 32]);
    /// `FUN_020813D8(ctx)`: SHA-1 context init.
    fn sha1_init(&mut self);
    /// `FUN_0208142C(ctx, data, len)`: SHA-1 update.
    fn sha1_update(&mut self, data: &[u8]);
    /// `FUN_020814C0(ctx, out)`: SHA-1 final, 20-byte digest.
    fn sha1_final(&mut self, out: &mut [u8; 20]);
}

/// C: `FUN_0203F258` (thunk at `0x020056C8`, called at boot by `FUN_020056D0`): seeds a
/// [`MathRandContext32`]: `x` = first 8 digest bytes of SHA-1(20 bytes of stack, 32 bytes of
/// entropy), `mul` = [`LCG64_MUL`], `add` = [`LCG64_ADD`].
///
/// UNVERIFIED: the original's first `sha1_update` hashes 20 bytes of an uninitialised stack
/// buffer (the digest buffer, written only by the final call); the port passes zeros.
pub fn seed_lcg64<H: SeedHost>(host: &mut H, ctx: &mut MathRandContext32) {
    let mut entropy = [0u8; 32];
    host.read_entropy(&mut entropy);
    host.sha1_init();
    let mut digest = [0u8; 20];
    host.sha1_update(&digest);
    host.sha1_update(&entropy);
    host.sha1_final(&mut digest);
    ctx.x = u64::from_le_bytes(digest[..8].try_into().unwrap());
    ctx.mul = LCG64_MUL;
    ctx.add = LCG64_ADD;
}

/// C: `FUN_0203F2FC(data, len)`: sum of the first `len` bytes (wrapping, as u32).
pub fn byte_sum(data: &[u8], len: u32) -> u32 {
    data[..len as usize]
        .iter()
        .fold(0u32, |s, &b| s.wrapping_add(b as u32))
}

/// C: `FUN_0203F388(state)`: LCG32 step `x = x * 0x41C64E6D + 0x6073`, returns `x >> 16`.
pub fn lcg32_next(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
    *state >> 16
}

/// C: `FUN_0203F328(data, size, seed)`: XORs halfword `i < size / 2` of `data` with the LCG32
/// key stream started at `seed` (the same cipher as `poke_crypt` `FUN_02019A50`).
pub fn lcg32_xor(data: &mut [u8], size: u32, seed: u32) {
    let mut st = seed;
    for i in 0..(size >> 1) as usize {
        let k = lcg32_next(&mut st) as u16;
        let v = u16::from_le_bytes([data[2 * i], data[2 * i + 1]]) ^ k;
        data[2 * i..2 * i + 2].copy_from_slice(&v.to_le_bytes());
    }
}

/// The Mersenne Twister of `gf_standard.c`. In the original this is a heap struct (allocated and
/// set up by `FUN_0203EF5C`) whose pointer lives at `0x021469F8`: `mt[624]` at +0, `index` at
/// +0x9C0, the twist table `mag[2]` at +0x9C4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mt19937 {
    pub mt: [u32; 624],
    pub index: u32,
    /// `{0, 0x9908B0DF}`; read from the struct, not from a constant, by `FUN_0203F0A8`.
    pub mag: [u32; 2],
}

impl Default for Mt19937 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mt19937 {
    /// Size of the original struct up to and including `mag` (+0x9C4 + 8).
    pub const BYTES: usize = 0x9CC;

    /// State as `FUN_0203EF5C` leaves it: `index` 0x271 (unseeded: the first `next` seeds with
    /// 5489), `mag = {0, 0x9908B0DF}`. (The allocation itself is not ported.)
    pub fn new() -> Self {
        Mt19937 {
            mt: [0; 624],
            index: 0x271,
            mag: [0, 0x9908_B0DF],
        }
    }

    /// Reads the original struct layout (little endian); `b.len() >= BYTES`.
    pub fn from_bytes(b: &[u8]) -> Self {
        let w = |o: usize| u32::from_le_bytes(b[o..o + 4].try_into().unwrap());
        let mut s = Mt19937::new();
        for (i, m) in s.mt.iter_mut().enumerate() {
            *m = w(4 * i);
        }
        s.index = w(0x9C0);
        s.mag = [w(0x9C4), w(0x9C8)];
        s
    }

    /// Writes the original struct layout into `b` (other bytes untouched).
    pub fn write_bytes(&self, b: &mut [u8]) {
        for (i, m) in self.mt.iter().enumerate() {
            b[4 * i..4 * i + 4].copy_from_slice(&m.to_le_bytes());
        }
        b[0x9C0..0x9C4].copy_from_slice(&self.index.to_le_bytes());
        b[0x9C4..0x9C8].copy_from_slice(&self.mag[0].to_le_bytes());
        b[0x9C8..0x9CC].copy_from_slice(&self.mag[1].to_le_bytes());
    }

    /// C: `FUN_0203F034`: `init_genrand`; leaves `index` = 624.
    pub fn seed(&mut self, seed: u32) {
        self.mt[0] = seed;
        for i in 1..624usize {
            let p = self.mt[i - 1];
            self.mt[i] = (p ^ (p >> 30))
                .wrapping_mul(0x6C07_8965)
                .wrapping_add(i as u32);
        }
        self.index = 624;
    }

    /// C: `FUN_0203F0A8`: next tempered 32-bit output. `index >= 624` regenerates the block
    /// (seeding with 5489 first if `index == 0x271`, i.e. never seeded). The original compares
    /// `index` as a signed int (a negative index reads out of bounds there and panics here).
    pub fn next_u32(&mut self) -> u32 {
        if self.index as i32 >= 624 {
            if self.index == 0x271 {
                self.seed(5489);
            }
            for i in 0..624usize {
                let y = (self.mt[i] & 0x8000_0000) | (self.mt[(i + 1) % 624] & 0x7FFF_FFFF);
                self.mt[i] = self.mt[(i + 397) % 624] ^ (y >> 1) ^ self.mag[(y & 1) as usize];
            }
            self.index = 0;
        }
        let mut y = self.mt[self.index as usize];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9D2C_5680;
        y ^= (y << 15) & 0xEFC6_0000;
        y ^ (y >> 18)
    }
}

#[cfg(test)]
mod tests;
