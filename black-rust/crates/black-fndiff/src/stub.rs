//! The stub model for unported callees (same as `make_stub` in tools/rom-diff/svflow.py).

use crate::spec::{ret_value, StubSpec, BUF_BASE, BUF_STRIDE, WORK_BASE};
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
    /// Calls so far per stub address (`seqptr` results, P4).
    seqptr: Vec<(u32, u32)>,
    /// Calls so far per stub address (`nth:` results, Z21).
    nth: Vec<(u32, u32)>,
    /// Calls so far per stub address (`ticks:` results, Z21) and the last 64-bit `ticks:` answer.
    ticks: Vec<(u32, u32)>,
    last_ticks: u64,
}

impl StubEnv {
    pub(crate) fn new(stubs: Vec<StubSpec>, salt: u32) -> Self {
        StubEnv {
            stubs,
            log: Vec::new(),
            salt,
            halted: None,
            counts: Vec::new(),
            seqptr: Vec::new(),
            nth: Vec::new(),
            ticks: Vec::new(),
            last_ticks: 0,
        }
    }

    /// Address of the `noreturn` stub that ran, if any: the port must have returned right after.
    pub fn halted(&self) -> Option<u32> {
        self.halted
    }

    /// `ret=u64` stub (B14-spec, e.g. OS_GetTick): r0 = the [`StubEnv::call`] result, r1 = `mix(r0, 0x6464)`
    /// (as `make_stub` in svflow.py); returns `r1 << 32 | r0`.
    pub fn call64(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u64 {
        let ret = self
            .stubs
            .iter()
            .find(|s| s.addr == addr)
            .map(|s| s.ret.clone());
        let ticks = ret.as_deref().is_some_and(|r| r.starts_with("ticks:"));
        assert!(
            ticks || ret.as_deref() == Some("u64"),
            "stub {addr:#010x}: call64 needs ret=u64 or ret=ticks:STEP"
        );
        let lo = self.call(addr, args);
        if ticks {
            return self.last_ticks;
        }
        u64::from(lo) | (u64::from(mix(lo, 0x6464)) << 32)
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
                        || k.starts_with("outw:")
                        || k.starts_with("outv:")
                    {
                        k.split(':').nth(1).map(|n| num(n) as usize)
                    } else if k.starts_with("outp:") {
                        // `outp:K:OFS:N` (B12-spec): one address word
                        Some(4)
                    } else if k.starts_with("outk:") || k.starts_with("outki:") {
                        // `outk:N:M..` / `outki:OFS:N:M..` (Z20): N is the second-to-last field; for `outki` the
                        // port passes the record itself (the original reaches it through the u32 at arg + OFS)
                        let t: Vec<&str> = k.split(':').collect();
                        Some(num(t[t.len() - 2]) as usize)
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
        // Z22: the words an `in:N` callee reads feed its answer too (after the scalars, in argument order), so two
        // queries of one stub at different positions answer differently
        for v in &extra {
            acc = mix(acc, *v);
        }
        for (i, (a, k)) in args.iter_mut().zip(st.args.iter()).enumerate() {
            if let Arg::Out(b) = a {
                if let Some(p) = k.strip_prefix("outp:") {
                    // `outp:K:OFS:N` (B12-spec): the out word is BUF_BASE + K * BUF_STRIDE + OFS
                    // + 4 * (mix(mix(acc, key + 0x1000 + 0x100 * (i + 1)), 0xC0DE) % N);
                    // `outp:work:OFS:N` (F4a-spec): the same inside the work image (WORK_BASE + OFS + ..)
                    let mut it = p.split(':');
                    let kk = it.next().unwrap();
                    let base = if kk == "work" {
                        WORK_BASE
                    } else {
                        BUF_BASE.wrapping_add(num(kk).wrapping_mul(BUF_STRIDE))
                    };
                    let (ofs, n) = (num(it.next().unwrap()), num(it.next().unwrap()));
                    let h = mix(mix(acc, key + 0x1000 + 0x100 * (i as u32 + 1)), 0xC0DE);
                    let v = base.wrapping_add(ofs).wrapping_add(4 * (h % n));
                    b.copy_from_slice(&v.to_le_bytes());
                    continue;
                }
                if let Some(p) = k.strip_prefix("outw:") {
                    // `outw:N:W0/W1/..` (F8-spec4): N bytes of 32-bit words, word j =
                    // W[mix(mix(acc, key + 0x1000 + 0x100 * (i + 1) + j), 0xC0DE) % len] (a stack struct whose
                    // field must hit one of a few exact values, e.g. an angle)
                    let w: Vec<u32> = p.split(':').nth(1).unwrap().split('/').map(num).collect();
                    assert!(b.len() % 4 == 0, "stub {addr:#010x}: outw size");
                    let salt = 0x100 * (i as u32 + 1);
                    for (j, c) in b.chunks_mut(4).enumerate() {
                        let h = mix(mix(acc, key + 0x1000 + salt + j as u32), 0xC0DE);
                        c.copy_from_slice(&w[(h % w.len() as u32) as usize].to_le_bytes());
                    }
                    continue;
                }
                if let Some(p) = k.strip_prefix("outv:") {
                    // `outv:N:V1/V2/..` (F3a-spec): the out bytes are the low N little-endian bytes of
                    // V[mix(mix(acc, key + 0x1000 + 0x100 * (i + 1)), 0xC0DE) % count] (N <= 4; the hash of `outp`)
                    let vs: Vec<u32> = p.split(':').nth(1).unwrap().split('/').map(num).collect();
                    let h = mix(mix(acc, key + 0x1000 + 0x100 * (i as u32 + 1)), 0xC0DE);
                    let v = vs[(h % vs.len() as u32) as usize].to_le_bytes();
                    let n = b.len();
                    assert!(n <= 4, "stub {addr:#010x}: outv writes at most 4 bytes");
                    b.copy_from_slice(&v[..n]);
                    continue;
                }
                if k.starts_with("outk:") || k.starts_with("outki:") {
                    // `outk` / `outki` (Z20): the `outm` byte under mask M[j % len], the old bits elsewhere
                    let m: Vec<u8> = k
                        .rsplit(':')
                        .next()
                        .unwrap()
                        .split('/')
                        .map(|x| num(x) as u8)
                        .collect();
                    let salt = 0x100 * (i as u32 + 1);
                    for (j, o) in b.iter_mut().enumerate() {
                        let mj = m[j % m.len()];
                        let h = mix(acc, key + 0x1000 + salt + j as u32);
                        *o = (*o & !mj) | (((mix(h, 0x5EED) >> 8) & 0xFF) as u8 & mj);
                    }
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
        if let Some(step) = st.ret.strip_prefix("ticks:") {
            // `ticks:STEP` (Z21): the n-th call answers the 64-bit n * STEP (r0 here, r1 through `call64`); a clock
            // that advances by STEP per read, independent of acc (`ticks_ret` in svflow.py)
            let n = match self.ticks.iter_mut().find(|c| c.0 == addr) {
                Some(c) => {
                    c.1 += 1;
                    c.1 - 1
                }
                None => {
                    self.ticks.push((addr, 1));
                    0
                }
            };
            self.last_ticks = u64::from(n) * u64::from(num(step));
            return self.last_ticks as u32;
        }
        if let Some(p) = st.ret.strip_prefix("seqptr:arg") {
            // `seqptr:argK:OFS:STEP` (P4): base(K) + OFS + n * STEP, n = this stub's 0-based call index in the
            // run (a bump allocator; independent of acc)
            let mut it = p.split(':');
            let (kk, ofs, step) = (
                num(it.next().unwrap()),
                num(it.next().unwrap()),
                num(it.next().unwrap()),
            );
            let n = match self.seqptr.iter_mut().find(|c| c.0 == addr) {
                Some(c) => {
                    c.1 += 1;
                    c.1 - 1
                }
                None => {
                    self.seqptr.push((addr, 1));
                    0
                }
            };
            return BUF_BASE
                .wrapping_add(kk.wrapping_mul(BUF_STRIDE))
                .wrapping_add(ofs)
                .wrapping_add(n.wrapping_mul(step));
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
        if let Some(p) = st.ret.strip_prefix("nth:") {
            return self.nth_ret(addr, p, mix(acc, key));
        }
        ret_value(&st.ret, mix(acc, key)).unwrap_or(0)
    }

    /// `ret=nth:K0;K1;...` (Z21): the n-th call (0-based, per run, per stub address, with or without `seq`) answers
    /// with kind `K_n` (the last kind for every later call), applied to the usual mix value (`nth_ret` in svflow.py).
    fn nth_ret(&mut self, addr: u32, kinds: &str, val: u32) -> u32 {
        let n = match self.nth.iter_mut().find(|c| c.0 == addr) {
            Some(c) => {
                c.1 += 1;
                c.1 - 1
            }
            None => {
                self.nth.push((addr, 1));
                0
            }
        };
        let ks: Vec<&str> = kinds.split(';').collect();
        ret_value(ks[(n as usize).min(ks.len() - 1)], val).unwrap_or(0)
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

    /// Z22: the words an `in:N` stub reads feed its answer and outs (after the scalars, in argument order); the log is
    /// unchanged. Vectors computed with tools/rom-diff/svflow.py (`tools/rom-diff/test_svflow.py`, salt 0x1234, no `seq`).
    #[test]
    fn in_words_feed_the_answer() {
        let mut e = StubEnv::new(vec![spec("u32", &["v", "in:8"], false)], 0x1234);
        let r1 = e.call(
            0x021B_B798,
            &mut [Arg::V(0xABCD), Arg::In(&[1, 2, 3, 4, 5, 6, 7, 8])],
        );
        let r2 = e.call(
            0x021B_B798,
            &mut [Arg::V(0xABCD), Arg::In(&[1, 2, 3, 4, 5, 6, 7, 9])],
        );
        assert_eq!((r1, r2), (0xF747_FDDA, 0xCE42_B9D0));
        assert_eq!(e.log[..4], [0x021B_B798, 0xABCD, 0x0403_0201, 0x0807_0605]);
        // without an `in:` argument the answer is the old one (the `u64_stub_returns_two_words` low word)
        let mut v = StubEnv::new(vec![spec("u32", &["v"], false)], 0x1234);
        assert_eq!(v.call(0x021B_B798, &mut [Arg::V(0xABCD)]), 0x731C_9F54);
        // the outs depend on the read words as well
        let mut e = StubEnv::new(vec![spec("u32", &["in:3", "v", "out:4"], false)], 0x1234);
        let mut out = [0u8; 4];
        let r = e.call(
            0x021B_B798,
            &mut [Arg::In(&[0xAA; 3]), Arg::V(7), Arg::Out(&mut out)],
        );
        assert_eq!(r, 0x2DC1_3597);
        assert_eq!(out, [36, 203, 171, 8]);
        assert_eq!(e.log, [0x021B_B798, 7, 0x00AA_AAAA]);
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

    /// `seqptr:argK:OFS:STEP` (P4): consecutive blocks of one pointer argument, whatever the arguments
    /// and `seq`; vector computed with tools/rom-diff/svflow.py (`make_stub`, salt 0x1234, `seq`).
    #[test]
    fn seqptr_hands_out_consecutive_blocks() {
        let mut e = StubEnv::new(vec![spec("seqptr:arg2:0x10:0x40", &["v"], true)], 0x1234);
        let got: Vec<u32> = (0..3)
            .map(|_| e.call(0x021B_B798, &mut [Arg::V(0xABCD)]))
            .collect();
        assert_eq!(got, [0x02E3_0010, 0x02E3_0050, 0x02E3_0090]);
        assert_eq!(
            e.log,
            [
                0x021B_B798,
                0xABCD,
                0x021B_B798,
                0xABCD,
                0x021B_B798,
                0xABCD
            ]
        );
    }

    /// `ret=u64` (B14-spec): r0 as a u32 stub, r1 = mix(r0, 0x6464); vector computed with svflow.py `make_stub`
    /// (salt 0x1234, no `seq`, one `v` argument 0xABCD).
    #[test]
    fn u64_stub_returns_two_words() {
        let mut e = StubEnv::new(vec![spec("u64", &["v"], false)], 0x1234);
        let mut u = StubEnv::new(vec![spec("u32", &["v"], false)], 0x1234);
        let lo = u.call(0x021B_B798, &mut [Arg::V(0xABCD)]);
        let v = e.call64(0x021B_B798, &mut [Arg::V(0xABCD)]);
        assert_eq!(v as u32, lo);
        assert_eq!(v, 0x5a15_fb42_731c_9f54);
    }

    /// `nth:K0;K1;..` (Z21): the call index picks the result kind (the last one repeats); vectors computed with
    /// svflow.py `make_stub` (salt 0x1234, one `v` argument 0xABCD; call 1 with `seq` equals the `seq` vector above).
    #[test]
    fn nth_picks_the_result_kind_by_call_index() {
        for (seq, want) in [
            (true, [0x1, 0xcd27_d8e5, 0x5, 0x6]),
            (false, [0x1, 0x731c_9f54, 0x2, 0x2]),
        ] {
            let mut e = StubEnv::new(vec![spec("nth:pick:0/1/2;u32;mod:7", &["v"], seq)], 0x1234);
            let got: Vec<u32> = (0..4)
                .map(|_| e.call(0x021B_B798, &mut [Arg::V(0xABCD)]))
                .collect();
            assert_eq!(got, want);
        }
    }

    /// `ticks:STEP` (Z21): call n answers n * STEP as a 64-bit value, whatever the arguments (svflow.py `ticks_ret`).
    #[test]
    fn ticks_advance_by_step() {
        let mut e = StubEnv::new(vec![spec("ticks:0x80000000", &["v"], true)], 0x1234);
        let got: Vec<u64> = (0..4)
            .map(|i| e.call64(0x021B_B798, &mut [Arg::V(i)]))
            .collect();
        assert_eq!(got, [0, 0x8000_0000, 0x1_0000_0000, 0x1_8000_0000]);
        assert_eq!(e.call(0x021B_B798, &mut [Arg::V(0)]), 0);
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

    /// `outk:N:M..` (Z20): the `outm` bytes under the masks, the old bits elsewhere; vectors computed with
    /// tools/rom-diff/svflow.py (`tools/rom-diff/test_svflow.py`, salt 0x1234, `seq`).
    #[test]
    fn outk_keeps_the_bits_outside_the_mask() {
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outk:5:0xF/0"], true)], 0x1234);
        let mut out = [0x11u8, 0x22, 0x33, 0x44, 0x55];
        let r = e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        // bits under 0xF = the `outm:5:0xF` bytes [1, 7, 15, 3, 15], masked 0xF / 0 / 0xF / 0 / 0xF
        assert_eq!(out, [0x11, 0x22, 0x3F, 0x44, 0x5F]);
        assert_eq!(e.log, [0x021B_B798, 0xABCD]);
        let mut m = StubEnv::new(vec![spec("u32", &["v", "outm:5:0xF"], true)], 0x1234);
        assert_eq!(
            r,
            m.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut [0u8; 5])])
        );
        // a full mask is `outm`
        let mut e = StubEnv::new(vec![spec("u32", &["v", "outk:6:0xFF"], true)], 0x1234);
        let mut out = [0xAAu8; 6];
        e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        assert_eq!(out, [113, 135, 79, 211, 223, 219]);
    }

    /// `outw:N:W0/W1/..` (F8-spec4): N bytes of words picked from the list; vectors from tools/rom-diff/test_svflow.py
    /// (`OutW`), salt 0x1234.
    #[test]
    fn outw_picks_words_from_the_list() {
        let mut e = StubEnv::new(
            vec![spec(
                "u32",
                &["v", "outw:12:0/0x5A000/0xB4000/0x10E000"],
                true,
            )],
            0x1234,
        );
        let mut out = [0u8; 12];
        let r = e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        let w: Vec<u32> = out
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        assert_eq!(w, [0xB4000, 0x5A000, 0xB4000]);
        assert_eq!(r, 0xb0a0_097a);
        assert_eq!(e.log, [0x021B_B798, 0xABCD]);
        let mut e = StubEnv::new(vec![spec("u32", &["outw:8:1/2/3", "v"], false)], 0x1234);
        let mut out = [0u8; 8];
        e.call(0x021B_B798, &mut [Arg::Out(&mut out), Arg::V(7)]);
        assert_eq!(out, [2, 0, 0, 0, 2, 0, 0, 0]);
    }

    /// `outki:OFS:N:M..` (Z20): the port passes the record the original reaches through the u32 at arg + OFS; the bytes
    /// are those of `outk` at the same argument position (vector from tools/rom-diff/test_svflow.py).
    #[test]
    fn outki_is_outk_on_the_record() {
        let mut e = StubEnv::new(
            vec![spec("u32", &["outki:0x4A8:6:0/0/0/0/0/3", "v"], true)],
            0x1234,
        );
        let mut rec = [0x10u8, 0x20, 0x30, 0x40, 0x50, 0x67];
        e.call(0x021B_B798, &mut [Arg::Out(&mut rec), Arg::V(7)]);
        assert_eq!(rec, [0x10, 0x20, 0x30, 0x40, 0x50, 0x65]);
        let mut k = StubEnv::new(
            vec![spec("u32", &["outk:6:0/0/0/0/0/3", "v"], true)],
            0x1234,
        );
        let mut rec2 = [0x10u8, 0x20, 0x30, 0x40, 0x50, 0x67];
        k.call(0x021B_B798, &mut [Arg::Out(&mut rec2), Arg::V(7)]);
        assert_eq!(rec, rec2);
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

    /// `outp:work:OFS:N` (F4a-spec): the out word is an address inside the work image; vectors computed
    /// with tools/rom-diff/svflow.py (`WORK`, `mix`), salt 0x1234, no `seq`.
    #[test]
    fn outp_work_writes_an_address_inside_the_work_image() {
        let mut e = StubEnv::new(vec![spec("u16", &["outp:work:0x100:8"], false)], 0x1234);
        let mut out = [0u8; 4];
        e.call(0x021B_B798, &mut [Arg::Out(&mut out)]);
        assert_eq!(u32::from_le_bytes(out), 0x02E0_0114);
        let mut e = StubEnv::new(vec![spec("u16", &["v", "outp:work:0x20:5"], false)], 0x1234);
        e.call(0x021B_B798, &mut [Arg::V(0), Arg::Out(&mut out)]);
        assert_eq!(u32::from_le_bytes(out), 0x02E0_0030);
    }

    /// `outv:N:V1/V2/..` (F3a-spec): the out bytes are one of the listed values (low N bytes, little-endian),
    /// picked like `outp`; vectors computed with tools/rom-diff/svflow.py, salt 0x1234, no `seq`.
    #[test]
    fn outv_writes_one_of_the_listed_values() {
        let mut e = StubEnv::new(
            vec![spec("u32", &["v", "outv:2:0x3FFF/0/1"], false)],
            0x1234,
        );
        let mut out = [0u8; 2];
        let r = e.call(0x021B_B798, &mut [Arg::V(0xABCD), Arg::Out(&mut out)]);
        assert_eq!(out, [0xFF, 0x3F]);
        assert_eq!(r, 0x731C_9F54);
        let mut e = StubEnv::new(
            vec![spec("u32", &["outv:4:0x11223344/5/6/7/8"], false)],
            0x1234,
        );
        let mut out = [0u8; 4];
        assert_eq!(e.call(0x021B_B798, &mut [Arg::Out(&mut out)]), 0x1868_5E8C);
        assert_eq!(u32::from_le_bytes(out), 7);
        // the argument position joins the hash
        let mut e = StubEnv::new(
            vec![spec("u32", &["v", "outv:4:0x11223344/5/6/7/8"], false)],
            0x1234,
        );
        assert_eq!(
            e.call(0x021B_B798, &mut [Arg::V(9), Arg::Out(&mut out)]),
            0x3512_9D94
        );
        assert_eq!(u32::from_le_bytes(out), 6);
    }
}
