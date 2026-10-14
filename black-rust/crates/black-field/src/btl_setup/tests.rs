use super::*;
use crate::hooks::{HeapHooks, Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02013090, "9206e67c61bdbb2f"),
    (0x020130B4, "8bd7ebac1b82243e"),
    (0x020130C4, "e13171bda55e12c8"),
    (0x020130DC, "e5de24646de72203"),
    (0x02013128, "cc13a44726265e77"),
    (0x0201313C, "c0a2a7e69f8e0b15"),
    (0x02013144, "1001620537c5df7e"),
    (0x02013158, "96a924dc56753f38"),
    (0x02013160, "535113fdd114dfce"),
    (0x0201320C, "0c3bd759b3ec680e"),
    (0x02013220, "36138428fecde7d2"),
    (0x02013234, "15e6cafd73cd910f"),
    (0x02013270, "414d14df0f5305e4"),
    (0x02013284, "bf17197096dfc9c3"),
    (0x0201333C, "903dccdf5281f95f"),
    (0x02013360, "8a33a4a7b8197047"),
    (0x0201347C, "09c57a9725849319"),
    (0x020134F4, "a852a15e5d3dfdcf"),
    (0x02013528, "078d73d421f58705"),
    (0x02013610, "d9fe80ed6b4eae15"),
    (0x02013644, "0b70ac30a017e716"),
    (0x02013678, "0173c30e0620d641"),
    (0x020136AC, "000cd53221540572"),
    (0x020136E0, "f2f07b407e9fa444"),
    (0x02013734, "574820d781a261fd"),
    (0x02013790, "051e9ff3d478153f"),
    (0x020137C0, "90324c58c356149e"),
    (0x020137F0, "d1101025d046aea2"),
    (0x02013810, "96d8166ad577bd11"),
    (0x02013864, "45a4642919684160"),
    (0x02013894, "3a01cd043921f471"),
    (0x020138C4, "a3b7e71ee5c19945"),
    (0x02013930, "0d5648b5aaf09c94"),
    (0x0201394C, "3fc2ba29c234b7f3"),
    (0x02013980, "db7b5b196cd23303"),
];
// fndiff-recorded-end

/// Every callee of `btl_setup.c` routed to the spec's stub lines; `mem` is the buffer the record is in.
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl Rom<'_> {
    fn v(&mut self, addr: u32, a: &[u32]) -> u32 {
        let mut args: Vec<Arg> = a.iter().map(|&x| Arg::V(x)).collect();
        self.stubs.call(addr, &mut args)
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl HeapHooks for Rom<'_> {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.v(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn heap_free(&mut self, block: u32) {
        self.v(0x0203_07B0, &[block]);
    }
}

impl BtlSetupHooks for Rom<'_> {
    fn fun_02045808(&mut self, v: u32) {
        self.v(0x0204_5808, &[v]);
    }
    fn fun_0201ac2c(&mut self, a: u32, b: u32) {
        self.v(0x0201_AC2C, &[a, b]);
    }
    fn fun_0200ae54(&mut self, x: u32, n: u32) -> u32 {
        self.v(0x0200_AE54, &[x, n])
    }
    fn fun_0200b040(&mut self, x: u32, out: u32) {
        self.stubs
            .call(0x0200_B040, &mut [Arg::V(x), Arg::Ptr(out)]);
    }
    fn fun_0201a30c(&mut self, x: u32, party: u32, y: u32) {
        self.v(0x0201_A30C, &[x, party, y]);
    }
    fn fun_0201a2a8(&mut self, x: u32, party: u32) {
        self.v(0x0201_A2A8, &[x, party]);
    }
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32 {
        self.v(0x0204_57B0, &[n, heap])
    }
    fn fun_0202428c(&mut self, q: u32) {
        self.v(0x0202_428C, &[q]);
    }
    fn fun_0201a920(&mut self, heap: u32) -> u32 {
        self.v(0x0201_A920, &[heap])
    }
    fn fun_0202a440(&mut self, y: u32, rec: u32, heap: u32) {
        self.v(0x0202_A440, &[y, rec, heap]);
    }
    fn fun_0202a4b4(&mut self, y: u32, party: u32, heap: u32) {
        self.v(0x0202_A4B4, &[y, party, heap]);
    }
    fn fun_02012984(&mut self, x: u32) -> u32 {
        self.v(0x0201_2984, &[x])
    }
    fn fun_0216aae4(&mut self, x: u32, party: u32, v: u32) {
        self.v(0x0216_AAE4, &[x, party, v]);
    }
    fn fun_0202a3fc(&mut self, v: u32) -> u32 {
        self.v(0x0202_A3FC, &[v])
    }
    fn fun_020127a4(&mut self, x: u32) -> u32 {
        self.v(0x0201_27A4, &[x])
    }
    fn fun_02012934(&mut self, x: u32) -> u32 {
        self.v(0x0201_2934, &[x])
    }
    fn fun_02012944(&mut self, x: u32) -> u32 {
        self.v(0x0201_2944, &[x])
    }
    fn fun_0201292c(&mut self, x: u32) -> u32 {
        self.v(0x0201_292C, &[x])
    }
    fn fun_02012924(&mut self, x: u32) -> u32 {
        self.v(0x0201_2924, &[x])
    }
    fn fun_0200ca1c(&mut self, x: u32) -> u32 {
        self.v(0x0200_CA1C, &[x])
    }
    fn fun_02012f20(&mut self, x: u32) -> u32 {
        self.v(0x0201_2F20, &[x])
    }
    fn fun_02012f2c(&mut self, x: u32) -> u32 {
        self.v(0x0201_2F2C, &[x])
    }
    fn fun_02012ebc(&mut self, x: u32) -> u32 {
        self.v(0x0201_2EBC, &[x])
    }
    fn fun_0200c0e8(&mut self) -> u32 {
        self.v(0x0200_C0E8, &[])
    }
    fn fun_0200873c(&mut self, v: u32) -> u32 {
        self.v(0x0200_873C, &[v])
    }
    fn fun_0200c258(&mut self, v: u32) -> u32 {
        self.v(0x0200_C258, &[v])
    }
    fn fun_0201296c(&mut self, x: u32) -> u32 {
        self.v(0x0201_296C, &[x])
    }
    fn fun_020142e8(&mut self, ew: u32, id: u32) -> u32 {
        self.v(0x0201_42E8, &[ew, id])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.v(0x0201_AA18, &[party])
    }
    fn fun_0201aa14(&mut self, party: u32) -> u32 {
        self.v(0x0201_AA14, &[party])
    }
    fn fun_0201293c(&mut self, x: u32) -> u32 {
        self.v(0x0201_293C, &[x])
    }
    fn fun_020076d0(&mut self, v: u32, _a: &mut u32, _b: &mut u32) -> u32 {
        // B18b: both out-pointers are locals of the caller (canon_ptr: PTR_LOCAL)
        let local = black_fndiff::STACK_ZERO.start;
        self.stubs.call(
            0x0200_76D0,
            &mut [Arg::V(v), Arg::Ptr(local), Arg::Ptr(local)],
        )
    }
    fn fun_0200846c(&mut self, heap: u32) -> u32 {
        self.v(0x0200_846C, &[heap])
    }
    fn fun_0203d1a4(&mut self, v: u32) -> u32 {
        self.v(0x0203_D1A4, &[v])
    }
    fn fun_020082ec(&mut self, heap: u32) -> u32 {
        self.v(0x0200_82EC, &[heap])
    }
    fn fun_02008314(&mut self, a: u32, b: u32) {
        self.v(0x0200_8314, &[a, b]);
    }
}

/// Run `f` with argument `k`'s buffer as the memory window (padded: the allocator stub hands out the
/// block at `+0xF0` of argument 0, and a record initialised there extends past the 0x140 buffer).
/// `f` gets the scalar arguments and a copy of the buffers (for `q` records passed by pointer).
fn run<R: Ret>(c: &mut Case, k: usize, f: impl FnOnce(&mut Rom, &[u32], &[Vec<u8>]) -> R) -> u64 {
    let bufs = c.bufs.clone();
    let vals = c.vals.clone();
    let mut bytes = std::mem::take(&mut c.bufs[k]);
    let n = bytes.len();
    bytes.resize(n + 0x1000, 0);
    let mut rom = Rom {
        stubs: &mut c.env,
        mem: Mem::new(vals[k], bytes),
    };
    let r = f(&mut rom, &vals, &bufs);
    let mut b = std::mem::take(&mut rom.mem.bytes);
    b.truncate(n);
    c.bufs[k] = b;
    r.ret()
}

/// What a ported function returns, as the harness result word.
trait Ret {
    fn ret(self) -> u64;
}

impl Ret for () {
    fn ret(self) -> u64 {
        0
    }
}

impl Ret for u32 {
    fn ret(self) -> u64 {
        u64::from(self)
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        // `ret=ofs:argK`: the port reports the offset of the returned block in the buffer
        0x0201_3090 => run(c, 0, |e, v, _| alloc(e, v[0]).wrapping_sub(v[0])),
        0x0201_3234 => run(c, 1, |e, v, _| new_trainer(e, v[0]).wrapping_sub(v[1])),
        0x0201_30B4 => run(c, 0, |e, v, _| free(e, v[0])),
        0x0201_30C4 => run(c, 0, |e, v, _| init(e, v[0])),
        0x0201_30DC => run(c, 0, |e, v, _| release(e, v[0])),
        0x0201_3128 => run(c, 0, |e, v, _| copy_to_party(e, v[0], v[1], v[2])),
        0x0201_313C => run(c, 0, |e, v, _| party(e, v[0], v[1])),
        0x0201_3144 => run(c, 0, |e, v, _| or_flags(e, v[0], v[1])),
        0x0201_3158 => run(c, 0, |e, v, _| test_flags(e, v[0], v[1])),
        0x0201_3160 => run(c, 0, |e, v, _| read_from_save(e, v[0], v[1], v[2])),
        0x0201_320C => run(c, 0, |e, v, _| init_config_alt(e, v[0])),
        0x0201_3220 => run(c, 0, |e, v, _| init_config(e, v[0])),
        0x0201_3270 => run(c, 0, |e, v, _| free_trainer(e, v[0])),
        0x0201_3284 => run(c, 0, |e, v, _| {
            load_trainer(e, v[0], v[1], v[2], v[3], v[4], v[5])
        }),
        0x0201_333C => run(c, 0, |e, v, _| init_party0(e, v[0], v[1], v[2])),
        0x0201_3360 => run(c, 0, |e, v, b| init_from_game(e, v[0], v[1], &b[2])),
        0x0201_347C => run(c, 0, |e, v, b| {
            setup_with_party(e, v[0], v[1], v[2], &b[3], v[4], v[5])
        }),
        0x0201_34F4 => run(c, 0, |e, v, b| setup_kind(e, v[0], v[1], v[2], &b[3], v[4])),
        0x0201_3528 => run(c, 0, |e, v, _| {
            setup_multi(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
        }),
        0x0201_3610 => run(c, 0, |e, v, b| {
            setup_single_0(e, v[0], v[1], &b[2], v[3], v[4])
        }),
        0x0201_3644 => run(c, 0, |e, v, b| {
            setup_single_1(e, v[0], v[1], &b[2], v[3], v[4])
        }),
        0x0201_3678 => run(c, 0, |e, v, b| {
            setup_single_2(e, v[0], v[1], &b[2], v[3], v[4])
        }),
        0x0201_36AC => run(c, 0, |e, v, b| {
            setup_single_3(e, v[0], v[1], &b[2], v[3], v[4])
        }),
        0x0201_36E0 => run(c, 0, |e, v, b| {
            setup_double(e, v[0], v[1], &b[2], v[3], v[4], v[5])
        }),
        0x0201_3734 => run(c, 0, |e, v, b| {
            setup_triple(e, v[0], v[1], &b[2], v[3], v[4], v[5], v[6])
        }),
        0x0201_3790 => run(c, 0, |e, v, _| {
            setup_multi_0(e, v[0], v[1], v[2], v[3], v[4])
        }),
        0x0201_37C0 => run(c, 0, |e, v, _| {
            setup_multi_1(e, v[0], v[1], v[2], v[3], v[4])
        }),
        0x0201_3864 => run(c, 0, |e, v, _| {
            setup_multi_2(e, v[0], v[1], v[2], v[3], v[4])
        }),
        0x0201_3894 => run(c, 0, |e, v, _| {
            setup_multi_3(e, v[0], v[1], v[2], v[3], v[4])
        }),
        0x0201_37F0 => run(c, 0, |e, v, _| {
            setup_multi_b(e, v[0], v[1], v[2], v[3], v[4], v[5])
        }),
        0x0201_3810 => run(c, 0, |e, v, _| {
            setup_multi_c(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
        }),
        0x0201_38C4 => run(c, 0, |e, v, _| build_all(e, v[0], v[1], v[2])),
        0x0201_3930 => run(c, 0, |e, v, _| free_all(e, v[0])),
        0x0201_394C => run(c, 0, |e, v, _| start_battle_ids(e, v[0])),
        0x0201_3980 => run(c, 0, |e, v, _| alloc_block(e, v[0], v[1])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field btl_setup`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "btl_setup",
            include_str!("../../../../tools/rom-diff/specs/black-field/btl_setup.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn mem_window_checks_bounds() {
    let m = Mem::new(0x2000, vec![1, 2, 3, 4]);
    assert_eq!(m.r32(0x2000), 0x0403_0201);
    assert_eq!(m.r16(0x2002), 0x0403);
    let r = std::panic::catch_unwind(|| m.r8(0x2004));
    assert!(r.is_err());
}
