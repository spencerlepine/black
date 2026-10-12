//! The stub model for unported callees (same as `make_stub` in tools/rom-diff/svflow.py).

use crate::spec::{ret_value, StubSpec, BUF_BASE, BUF_STRIDE};
use crate::{mix, num};

/// Unwind payload of a `noreturn` stub: the original never comes back from a fatal routine, so
/// the call unwinds to the suite, which hashes the state the port had reached. A wrapper around
/// the port must catch the unwind, move its state back, and `resume_unwind` the payload.
pub struct Halt;

/// One argument of a stubbed callee, in the callee's own register/stack order.
pub enum Arg<'a> {
    /// Scalar (or a pointer whose value matters): logged and mixed into the result.
    V(u32),
    /// Pointer the callee only passes through: ignored by the stub.
    P,
    /// Pointer to bytes the callee reads: the stub logs them (as little-endian words).
    In(&'a [u8]),
    /// Pointer to bytes the callee writes: the stub fills them.
    Out(&'a mut [u8]),
}

/// The unported callees of one function under test; behaviour comes from its spec stub lines.
#[derive(Default)]
pub struct StubEnv {
    stubs: Vec<StubSpec>,
    pub log: Vec<u32>,
    salt: u32,
    halted: Option<u32>,
    /// Calls so far per stub address (`seq` stubs).
    counts: Vec<(u32, u32)>,
}

impl StubEnv {
    pub(crate) fn new(stubs: Vec<StubSpec>, salt: u32) -> Self {
        StubEnv {
            stubs,
            log: Vec::new(),
            salt,
            halted: None,
            counts: Vec::new(),
        }
    }

    /// Address of the `noreturn` stub that ran, if any: the port must have returned right after.
    pub fn halted(&self) -> Option<u32> {
        self.halted
    }

    /// Calls the unported function at `addr`; returns its r0 (0 for a callee returning nothing).
    /// A `noreturn` stub does not return: it unwinds with [`Halt`].
    pub fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        let st = self
            .stubs
            .iter()
            .find(|s| s.addr == addr)
            .unwrap_or_else(|| {
                panic!("callee {addr:#010x} has no stub in the spec: report it, do not invent one")
            })
            .clone();
        assert_eq!(
            st.args.len(),
            args.len(),
            "stub {addr:#010x}: wrong arg count"
        );
        let key = addr & 0xFFFF;
        let mut vals = Vec::new();
        let mut extra = Vec::new();
        for (k, a) in st.args.iter().zip(args.iter()) {
            match a {
                Arg::V(v) => {
                    // `outn:N:M..` (B10-spec2): a null pointer is a plain value argument
                    assert!(
                        k == "v" || k == "v16" || (k.starts_with("outn:") && *v == 0),
                        "stub {addr:#010x}: arg kind mismatch"
                    );
                    // `v16` (B10-spec4): only the low halfword counts (the callee ignores the rest,
                    // e.g. a u16 stored into a stale stack word)
                    vals.push(if k == "v16" { *v & 0xFFFF } else { *v });
                }
                Arg::P => assert_eq!(k, "p", "stub {addr:#010x}: arg kind mismatch"),
                Arg::In(b) => {
                    assert!(
                        k.strip_prefix("in:").map(|n| num(n) as usize) == Some(b.len()),
                        "stub {addr:#010x}: arg kind"
                    );
                    extra.extend(b.chunks(4).map(|c| {
                        // N need not be a multiple of 4: the last word is zero-padded
                        let mut w = [0u8; 4];
                        w[..c.len()].copy_from_slice(c);
                        u32::from_le_bytes(w)
                    }));
                }
                Arg::Out(b) => {
                    // sizes are decimal or 0x-hex, as Python's int(x, 0) reads them
                    let size = if k.starts_with("out:")
                        || k.starts_with("outm:")
                        || k.starts_with("outn:")
                    {
                        k.split(':').nth(1).map(|n| num(n) as usize)
                    } else if k.starts_with("outp:") {
                        // `outp:K:OFS:N` (B12-spec): one address word
                        Some(4)
                    } else {
                        None
                    };
                    assert!(size == Some(b.len()), "stub {addr:#010x}: arg kind");
                }
            }
        }
        self.log.push(addr);
        self.log.extend(&vals);
        self.log.extend(&extra);
        if st.noreturn {
            self.halted = Some(addr);
            std::panic::resume_unwind(Box::new(Halt));
        }
        let mut acc = self.salt;
        if st.seq {
            let n = match self.counts.iter_mut().find(|c| c.0 == addr) {
                Some(c) => {
                    c.1 += 1;
                    c.1 - 1
                }
                None => {
                    self.counts.push((addr, 1));
                    0
                }
            };
            acc = mix(acc, n);
        }
        for v in &vals {
            acc = mix(acc, *v);
        }
        for (i, (a, k)) in args.iter_mut().zip(st.args.iter()).enumerate() {
            if let Arg::Out(b) = a {
                if let Some(p) = k.strip_prefix("outp:") {
                    // `outp:K:OFS:N` (B12-spec): the out word is BUF_BASE + K * BUF_STRIDE + OFS
                    // + 4 * (mix(mix(acc, key + 0x1000 + 0x100 * (i + 1)), 0xC0DE) % N)
                    let mut it = p.split(':');
                    let (kk, ofs, n) = (
                        num(it.next().unwrap()),
                        num(it.next().unwrap()),
                        num(it.next().unwrap()),
                    );
                    let h = mix(mix(acc, key + 0x1000 + 0x100 * (i as u32 + 1)), 0xC0DE);
                    let v = BUF_BASE
                        .wrapping_add(kk.wrapping_mul(BUF_STRIDE))
                        .wrapping_add(ofs)
                        .wrapping_add(4 * (h % n));
                    b.copy_from_slice(&v.to_le_bytes());
                    continue;
                }
                // `outm:N:M0/M1/..` ANDs byte j with M[j % len]
                let m: Vec<u8> = if k.starts_with("outm:") || k.starts_with("outn:") {
                    k.split(':')
                        .nth(2)
                        .unwrap()
                        .split('/')
                        .map(|x| num(x) as u8)
                        .collect()
                } else {
                    vec![0xFF]
                };
                // `outm` also mixes in its argument position (two outs of one call differ), is mixed twice (one
                // mix() keeps the outputs of different calls XOR-correlated) and takes bits 8-15; `out` keeps
                // the plain low byte
                let outm = k.starts_with("outm:") || k.starts_with("outn:");
                let salt = if outm { 0x100 * (i as u32 + 1) } else { 0 };
                for (j, o) in b.iter_mut().enumerate() {
                    let h = mix(acc, key + 0x1000 + salt + j as u32);
                    *o = if outm {
                        ((mix(h, 0x5EED) >> 8) & 0xFF) as u8 & m[j % m.len()]
                    } else {
                        (h & 0xFF) as u8
                    };
                }
            }
        }
        if let Some(k) = st.ret.strip_prefix("lt:") {
            // `lt:K` (B10-spec4): mix(acc, key) % argK, 0 when argK is 0 (a `rand_range(n)` answer below n)
            let n = match args[num(k) as usize] {
                Arg::V(n) => n,
                _ => panic!("stub {addr:#010x}: lt:K needs a scalar argument"),
            };
            let r = mix(acc, key);
            return if n == 0 { 0 } else { r % n };
        }
        ret_value(&st.ret, mix(acc, key)).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(ret: &str, args: &[&str], seq: bool) -> StubSpec {
        StubSpec {
            addr: 0x021B_B798,
            ret: ret.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            noreturn: false,
            seq,
        }
    }

    /// Vectors computed with tools/rom-diff/svflow.py (`seq` stubs, B10-specs).
    #[test]
    fn seq_stub_mixes_the_call_index() {
        let mut e = StubEnv::new(vec![spec("u32", &["v"], true)], 0x1234);
        let got: Vec<u32> = (0..3)
            .map(|_| e.call(0x021B_B798, &mut [Arg::V(0xABCD)]))
            .collect();
        assert_eq!(got, [0xb0a0_097a, 0xcd27_d8e5, 0xe959_12c9]);
        // without `seq` the answer never changes
        let mut e = StubEnv::new(vec![spec("u32", &["v"], false)], 0x1234);
        let a = e.call(0x021B_B798, &mut [Arg::V(0xABCD)]);
        assert_eq!(a, e.call(0x021B_B798, &mut [Arg::V(0xABCD)]));
    }

    #[test]
    fn plain_out_keeps_the_low_byte() {
        let mut e = StubEnv::new(vec![spec("u32", &["v", "out:5"], true)], 0x1234);
        let mut out = [0u8; 5];
        e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        assert_eq!(out, [201, 38, 70, 229, 3]);
    }

    #[test]
    fn in_pointer_of_odd_size_is_zero_padded() {
        let mut e = StubEnv::new(vec![spec("none", &["in:3"], false)], 7);
        e.call(0x021B_B798, &mut [Arg::In(&[1, 2, 3])]);
        assert_eq!(e.log, [0x021B_B798, 0x0003_0201]);
    }

    /// `outn:N:M` (B10-spec2): `outm` for a pointer, a plain value argument for null.
    #[test]
    fn outn_is_outm_for_a_pointer_and_a_value_for_null() {
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outn:5:0xF"], true)], 0x1234);
        let mut out = [0u8; 5];
        let r_ptr = e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        // same bytes as the `outm:5:0xF` vector above
        assert_eq!(out, [1, 7, 15, 3, 15]);
        let mut m = StubEnv::new(vec![spec("u32", &["v", "outm:5:0xF"], true)], 0x1234);
        assert_eq!(
            r_ptr,
            m.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut [0u8; 5])])
        );
        // null: logged and mixed in like a `v` argument of value 0
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outn:5:0xF"], true)], 0x1234);
        let r_null = e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::V(0)]);
        let mut v = StubEnv::new(vec![spec("u32", &["v", "v"], true)], 0x1234);
        assert_eq!(
            r_null,
            v.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::V(0)])
        );
        assert_eq!(e.log, v.log);
    }

    /// `v16` (B10-spec4): the high halfword is neither logged nor mixed in.
    #[test]
    fn v16_drops_the_high_halfword() {
        let mut a = StubEnv::new(vec![spec("u32", &["v16"], false)], 0x1234);
        let mut b = StubEnv::new(vec![spec("u32", &["v"], false)], 0x1234);
        let ra = a.call(0x021B_B798, &mut [Arg::V(0x021C_59B1)]);
        let rb = b.call(0x021B_B798, &mut [Arg::V(0x59B1)]);
        assert_eq!(ra, rb);
        assert_eq!(a.log, b.log);
    }

    /// `lt:K` (B10-spec4): below argument K, 0 for a zero bound.
    #[test]
    fn lt_answers_below_the_bound() {
        let mut e = StubEnv::new(vec![spec("lt:0", &["v"], true)], 0x1234);
        for n in 1..8 {
            for _ in 0..20 {
                assert!(e.call(0x021B_B798, &mut [Arg::V(n)]) < n);
            }
        }
        assert_eq!(e.call(0x021B_B798, &mut [Arg::V(0)]), 0);
    }

    #[test]
    fn outm_masks_the_bytes_and_ptr_returns_an_address() {
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outm:5:0xF"], true)], 0x1234);
        let mut out = [0u8; 5];
        e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        // the same salt, call index 0 and scalar argument as the vector above
        assert_eq!(out, [1, 7, 15, 3, 15]);
        // a mask list is cycled by byte index: bytes 1 and 3 are forced to 0 (outm mixes its argument position, here 1,
        // is mixed twice and takes bits 8-15)
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outm:5:0xF/0"], true)], 0x1234);
        e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        assert_eq!(out, [1, 0, 15, 0, 15]);
        assert_eq!(ret_value("ptr:arg2:0xD00:64", 0x12345), Some(0x02E3_0DB4));
        // nullable (vectors from svflow.py): value 22 gives null, value 1 gives the pointer
        assert_eq!(ret_value("nullable:4:ptr:arg2:0xD00:64", 22), Some(0));
        assert_eq!(
            ret_value("nullable:4:ptr:arg2:0xD00:64", 1),
            Some(0x02E3_0D20)
        );
        // mix(0x12345, 0xC0DE) % 4 = 1 -> the second value
        assert_eq!(ret_value("pick:0xfd/0xfe/0xff/7", 0x12345), Some(0xfe));
        // mix(0x12345, 0xC0DE) % 64 = 45 -> WORK + 0x1000 + 180
        assert_eq!(ret_value("ptr:work:0x1000:64", 0x12345), Some(0x02E0_10B4));
    }

    /// `outp:K:OFS:N` (B12-spec): the out word is an address inside pointer argument K; vectors
    /// computed with tools/rom-diff/svflow.py (`Runner.buf_addr`, `mix`), salt 0x1234, no `seq`.
    #[test]
    fn outp_writes_an_address_inside_a_pointer_argument() {
        let mut e = StubEnv::new(vec![spec("u16", &["outp:0:0x800:64"], false)], 0x1234);
        let mut out = [0u8; 4];
        e.call(0x021B_B798, &mut [Arg::Out(&mut out)]);
        assert_eq!(u32::from_le_bytes(out), 0x02E1_0834);
        // the argument position joins the hash (a second out of the same call differs)
        let mut e = StubEnv::new(vec![spec("u16", &["v", "outp:2:0x10:5"], false)], 0x1234);
        e.call(0x021B_B798, &mut [Arg::V(0), Arg::Out(&mut out)]);
        assert_eq!(u32::from_le_bytes(out), 0x02E3_0020);
    }
}
