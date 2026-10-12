//! Per-function differential test kit (Z16), for any logic crate's tests.
//!
//! Mirrors `tools/rom-diff/svflow.py`: both sides read the same spec file (grammar in
//! `tools/rom-diff/svflow_specs/README.md`), draw the same inputs from the same xorshift stream,
//! model the unported callees the same way, and fold the same word stream into the same FNV-1a
//! hash. A crate adds this as a `[dev-dependencies]` path dependency and writes one test:
//!
//! ```ignore
//! #[test]
//! fn fndiff_match_rom() {
//!     black_fndiff::run_suite(
//!         &Suite::new("area", include_str!("../../../../tools/rom-diff/specs/crate/area.spec"),
//!                     super::PORTED, RECORDED),
//!         &|addr, c: &mut Case| match addr { 0x0201_9A7C => /* call the port */ 0, _ => unreachable!() },
//!     );
//! }
//! ```
//!
//! Porting workers wire their function into the `call` match; they do not touch the spec, this
//! crate or the recorded hashes (`tools/check-fn.sh` records them).

mod run;
mod spec;
mod stub;

pub use run::{run_suite, Case, Ext, Suite};
pub use stub::{Arg, Halt, StubEnv};

pub(crate) const FNV_OFFSET: u64 = 14695981039346656037;
pub(crate) const FNV_PRIME: u64 = 1099511628211;

pub(crate) fn fold(h: u64, w: u32) -> u64 {
    (h ^ w as u64).wrapping_mul(FNV_PRIME)
}

/// The stub result mixer; same as `mix` in `tools/rom-diff/bpp.py`.
pub fn mix(a: u32, b: u32) -> u32 {
    let x = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
    x ^ (x >> 15)
}

/// xorshift32, seeded like the Python `Rng` in `tools/rom-diff/emu.py`.
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        let s = seed.wrapping_mul(2654435761).wrapping_add(1);
        Rng(if s == 0 { 1 } else { s })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
}

pub(crate) fn num(s: &str) -> u32 {
    // A leading '-' wraps to u32, as svflow.py reads it (int(x, 0) & 0xFFFFFFFF).
    let (neg, t) = match s.strip_prefix('-') {
        Some(t) => (true, t),
        None => (false, s),
    };
    let v = match t.strip_prefix("0x") {
        Some(h) => u32::from_str_radix(h, 16),
        None => t.parse(),
    }
    .unwrap_or_else(|_| panic!("fndiff spec: bad number {s:?}"));
    if neg {
        v.wrapping_neg()
    } else {
        v
    }
}
