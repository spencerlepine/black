use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C9934, "2f1a195a1bc0085c"),
    (0x021C9B24, "7557b07016d91564"),
    (0x021C9B78, "eae5d66bd15bd196"),
    (0x021C9BCC, "b8fcd94c479ad24a"),
    (0x021C9C54, "9d89290f464ad899"),
    (0x021C9CAC, "cb71248a3c7fea42"),
    (0x021C9D30, "afdf88d296c05b7f"),
    (0x021C9D70, "07a58e55d75d0d9a"),
    (0x021C9DB0, "d1a00d1f99c39ba2"),
    (0x021C9E8C, "afa3fef712087b31"),
    (0x021C9F44, "22441faf45160fa4"),
    (0x021C9FD0, "6ea29e1f8e9970bf"),
    (0x021C9FFC, "b70691fcee9673e0"),
    (0x021CA048, "01459e7c5a37390f"),
    (0x021CA05C, "705f9c277d28ae95"),
    (0x021CA150, "ee8f5106df8dac60"),
    (0x021CA164, "120e38abe33602cd"),
    (0x021CA1E4, "b1a075dd7998894e"),
    (0x021CA210, "7eb71e531aa6c93d"),
    (0x021CA234, "5cb7681bcaf7fd8d"),
    (0x021CA238, "4f7af0b644b020d1"),
    (0x021CA23C, "f64d065ba0f72fd8"),
    (0x021CA250, "c8a79800acefeaa1"),
    (0x021CA258, "d9659586872315f3"),
    (0x021CA270, "2195266678bb00b1"),
    (0x021CA2A4, "0953c1d341c8cf9e"),
    (0x021CA2CC, "82674b25aacfb9d6"),
    (0x021CA2DC, "8617b7efce74bf62"),
    (0x021CA338, "7ce6f99a3297872f"),
    (0x021CA3B4, "acbfd29954cb72e6"),
    (0x021CA4C0, "5f2e5258a4b80a8a"),
    (0x021CA544, "5bcb54268e2fc09c"),
    (0x021CA57C, "a4709e63a8608f8b"),
    (0x021CA598, "0f6bd7773ade0d23"),
    (0x021CA5C4, "74cb14c497b46a94"),
    (0x021CB6DC, "0916a3edeb3b460e"),
    (0x021CB700, "ddf00acf1e2c39aa"),
    (0x021CB754, "68f6c3499d94deec"),
    (0x021CB7E8, "cdc1c61d69d6a5ad"),
    (0x021CB86C, "75f27ffc588e8036"),
    (0x021CB8AC, "3dc3983ff9916bc0"),
    (0x021CB914, "c38ecd9b4890b01a"),
    (0x021CB934, "2d6276dc49c567c0"),
    (0x021CB9BC, "d48d9f8c478dd631"),
    (0x021CB9E4, "670e17719eed7730"),
    (0x021CBBA0, "27dcd025ed8823c1"),
    (0x021CBBAC, "efe766021f06f2b4"),
    (0x021CBBD0, "d3c957bfd17fd0d8"),
    (0x021CBC14, "7c526f21362719e7"),
    (0x021CBC94, "a671745f0ca3cd40"),
    (0x021CBCC4, "af6ce67fbb3671bc"),
    (0x021CBD08, "15aecd708f39a7aa"),
    (0x021CBD84, "3248ff97724e582e"),
    (0x021CBD90, "c9d67c7bf3bb49c0"),
    (0x021CBED8, "1cd0e5dc2445e129"),
    (0x021CBEFC, "c20fd52b1d423e61"),
    (0x021CBF38, "7e35bd67fc0ea05b"),
    (0x021CBF98, "9c378e7df3c40deb"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer of the case in one memory window set (at the emulator's addresses).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
    /// The function under test (picks the stub variant where the spec's stub lines differ between functions).
    cur: u32,
    /// Every stub answer so far, in call order.
    rets: Vec<(u32, u32)>,
}

#[allow(dead_code)]
impl Rom<'_> {
    fn rd(&mut self, a: u32, n: usize) -> Vec<u8> {
        (0..n as u32).map(|i| self.mem.r8(a + i)).collect()
    }
    fn wr(&mut self, a: u32, b: &[u8]) {
        for (i, &x) in b.iter().enumerate() {
            self.mem.w8(a + i as u32, x);
        }
    }
    /// The address the original passes for a stack local of its own (parameter `k` of the stub at `addr`, logged as a value):
    /// the slot in the original's frame (entry sp 0x02F00000, Z19).
    fn ptr(&self, addr: u32, k: usize) -> u32 {
        const SP: u32 = 0x02F0_0000;
        // command 346: push {r3-r7, lr}; sub sp, 0xF0
        const SP_346: u32 = SP - 0x108;
        // FUN_021CA05C: push {r4-r7, lr}; sub sp, 0x1C (the species / items arrays at sp + 0x18 / + 0x14)
        match (self.cur, addr, k) {
            (0x021C_A5C4, 0x0200_B5CC, 2) => SP_346 + 0x78,
            (0x021C_A05C, 0x0216_8878, 3) => SP - 0x30 + 0x18,
            (0x021C_A05C, 0x0216_8878, 4) => SP - 0x30 + 0x14,
            (0x021C_A5C4, 0x0216_8878, 3) => SP_346 - 0x30 + 0x18,
            (0x021C_A5C4, 0x0216_8878, 4) => SP_346 - 0x30 + 0x14,
            _ => panic!("no address for {addr:#x} arg {k} under {:#x}", self.cur),
        }
    }
    fn buf(&self, b: &Buf, k: usize) -> u32 {
        match b {
            Buf::Addr(a) => *a,
            Buf::Local(_) => self.ptr(0x0216_8878, k),
        }
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

// gen:manual fun_02168878
impl Rom<'_> {
    #[allow(clippy::too_many_arguments)]
    fn stub_02168878(
        &mut self,
        rec: u32,
        id: u32,
        n: u32,
        a: Buf,
        b: Buf,
        c: u32,
        heap: u32,
    ) -> u32 {
        let (pa, pb) = (self.buf(&a, 3), self.buf(&b, 4));
        let r = self.stubs.call(
            0x0216_8878,
            &mut [
                Arg::V(rec),
                Arg::V(id),
                Arg::V(n),
                Arg::V(pa),
                Arg::V(pb),
                Arg::V(c),
                Arg::V(heap),
            ],
        );
        self.rets.push((0x0216_8878, r));
        r
    }
}

// gen:rom-begin (tools: /tmp/blk/f8f/gen.py, from the trait and the spec's stub lines)
impl CoreHooks for Rom<'_> {
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        let r = self.stubs.call(0x02011330, &mut [Arg::V(vm)]);
        self.rets.push((0x02011330, r));
        r
    }
    fn fun_020113d0(&mut self, vm: u32, f: u32) {
        let r = self.stubs.call(0x020113d0, &mut [Arg::V(vm), Arg::V(f)]);
        self.rets.push((0x020113d0, r));
    }
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a5dc, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a5dc, r));
        r
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a5f4, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a5f4, r));
        r
    }
    fn fun_021587e0(&mut self, _sw: u32, _slot: u32) -> u32 {
        unreachable!("fun_021587e0: no stub in the spec")
    }
    fn fun_0215897c(&mut self, sw: u32, child: u32) {
        let r = self
            .stubs
            .call(0x0215897c, &mut [Arg::V(sw), Arg::V(child)]);
        self.rets.push((0x0215897c, r));
    }
    fn fun_021589a0(&mut self, _sw: u32, _zone: u32, _id: u32) -> u32 {
        unreachable!("fun_021589a0: no stub in the spec")
    }
    fn fun_02158a34(&mut self, _gamesys: u32, _sw: u32, _id: u32, _mode: u32) {
        unreachable!("fun_02158a34: no stub in the spec")
    }
    fn fun_02159ae8(&mut self, _vm: u32, _ctx: u32) -> u32 {
        unreachable!("fun_02159ae8: no stub in the spec")
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        let r = self.stubs.call(0x02159b10, &mut [Arg::V(vm), Arg::V(ctx)]);
        self.rets.push((0x02159b10, r));
        r
    }
    fn fun_02159b38(&mut self, _ctx: u32) -> u32 {
        unreachable!("fun_02159b38: no stub in the spec")
    }
    fn fun_0215a684(&mut self, ctx: u32, n: u32) {
        let r = self.stubs.call(0x0215a684, &mut [Arg::V(ctx), Arg::V(n)]);
        self.rets.push((0x0215a684, r));
    }
    fn fun_0215a690(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a690, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a690, r));
        r
    }
    fn fun_0215aa24(&mut self, _ctx: u32) {
        unreachable!("fun_0215aa24: no stub in the spec")
    }
    fn fun_0215aa34(&mut self, _ctx: u32) {
        unreachable!("fun_0215aa34: no stub in the spec")
    }
    fn fun_020120f4(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x020120f4, &mut [Arg::V(gamesys)]);
        self.rets.push((0x020120f4, r));
        r
    }
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32 {
        let r = self.stubs.call(
            0x020122c0,
            &mut [Arg::V(gamesys), Arg::V(parent), Arg::V(f), Arg::V(size)],
        );
        self.rets.push((0x020122c0, r));
        r
    }
    fn fun_020124e8(&mut self, event: u32) -> u32 {
        let r = self.stubs.call(0x020124e8, &mut [Arg::V(event)]);
        self.rets.push((0x020124e8, r));
        r
    }
    fn fun_0201296c(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x0201296c, &mut [Arg::V(gamedata)]);
        self.rets.push((0x0201296c, r));
        r
    }
    fn fun_02012a30(&mut self, _gamedata: u32) -> u32 {
        unreachable!("fun_02012a30: no stub in the spec")
    }
    fn fun_02012b38(&mut self, _gamedata: u32) -> u32 {
        unreachable!("fun_02012b38: no stub in the spec")
    }
    fn fun_020142e8(&mut self, _ew: u32, _flag: u32) -> u32 {
        unreachable!("fun_020142e8: no stub in the spec")
    }
    fn fun_02014314(&mut self, ew: u32, flag: u32) {
        let r = self.stubs.call(0x02014314, &mut [Arg::V(ew), Arg::V(flag)]);
        self.rets.push((0x02014314, r));
    }
    fn fun_0201433c(&mut self, ew: u32, flag: u32) {
        let r = self.stubs.call(0x0201433c, &mut [Arg::V(ew), Arg::V(flag)]);
        self.rets.push((0x0201433c, r));
    }
    fn fun_02014460(&mut self, _x: u32) -> u32 {
        unreachable!("fun_02014460: no stub in the spec")
    }
    fn fun_02014474(&mut self, _x: u32, _v: u32) {
        unreachable!("fun_02014474: no stub in the spec")
    }
    fn fun_020288c4(&mut self, _x: u32) -> u32 {
        unreachable!("fun_020288c4: no stub in the spec")
    }
    fn fun_0202972c(&mut self, _gamesys: u32, _a: u32) -> u32 {
        unreachable!("fun_0202972c: no stub in the spec")
    }
    fn fun_021b1bb8(&mut self, _vm: u32, _ctx: u32) -> u32 {
        unreachable!("fun_021b1bb8: no stub in the spec")
    }
    fn fun_021b1be8(&mut self, _ctx: u32) {
        unreachable!("fun_021b1be8: no stub in the spec")
    }
}

#[allow(clippy::too_many_arguments)]
impl BsubwayHooks for Rom<'_> {
    fn fun_020056ec(&mut self, n: u32) -> u32 {
        let r = self.stubs.call(0x020056ec, &mut [Arg::V(n)]);
        self.rets.push((0x020056ec, r));
        r
    }
    fn fun_020071cc(&mut self, save: u32, id: u32) -> u32 {
        let r = self.stubs.call(0x020071cc, &mut [Arg::V(save), Arg::V(id)]);
        self.rets.push((0x020071cc, r));
        r
    }
    fn fun_02008550(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02008550, &mut [Arg::V(x)]);
        self.rets.push((0x02008550, r));
        r
    }
    fn fun_0200b424(&mut self) {
        let r = self.stubs.call(0x0200b424, &mut []);
        self.rets.push((0x0200b424, r));
    }
    fn fun_0200b5cc(&mut self, save: u32, k: u32, out: &mut u32, x: u32) -> u32 {
        let _ = &out;
        let r = self.stubs.call(
            0x0200b5cc,
            &mut [
                Arg::V(save),
                Arg::V(k),
                Arg::V(self.ptr(0x0200b5cc, 2)),
                Arg::V(x),
            ],
        );
        self.rets.push((0x0200b5cc, r));
        r
    }
    fn fun_0200b6e0(&mut self, gd: u32, k: u32, kind: u32, v: u32, x: u32, a: u32, b: u32) -> u32 {
        let _ = &b;
        let r = self.stubs.call(
            0x0200b6e0,
            &mut [
                Arg::V(gd),
                Arg::V(k),
                Arg::V(kind),
                Arg::V(v),
                Arg::V(x),
                Arg::V(a),
            ],
        );
        self.rets.push((0x0200b6e0, r));
        r
    }
    fn fun_0200ba38(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200ba38, &mut [Arg::V(x)]);
        self.rets.push((0x0200ba38, r));
        r
    }
    fn fun_0200ba58(&mut self) {
        let r = self.stubs.call(0x0200ba58, &mut []);
        self.rets.push((0x0200ba58, r));
    }
    fn fun_0200ba68(&mut self, x: u32) {
        let r = self.stubs.call(0x0200ba68, &mut [Arg::V(x)]);
        self.rets.push((0x0200ba68, r));
    }
    fn fun_0200bac8(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200bac8, &mut [Arg::V(save)]);
        self.rets.push((0x0200bac8, r));
        r
    }
    fn fun_0200baf4(&mut self, x: u32, heap: u32) -> u32 {
        let r = self.stubs.call(0x0200baf4, &mut [Arg::V(x), Arg::V(heap)]);
        self.rets.push((0x0200baf4, r));
        r
    }
    fn fun_0200bba8(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200bba8, &mut [Arg::V(x)]);
        self.rets.push((0x0200bba8, r));
        r
    }
    fn fun_0200d85c(&mut self, p: u32) {
        let r = self.stubs.call(0x0200d85c, &mut [Arg::V(p)]);
        self.rets.push((0x0200d85c, r));
    }
    fn fun_0200d868(&mut self, p: u32, x: u32) {
        let r = self.stubs.call(0x0200d868, &mut [Arg::V(p), Arg::V(x)]);
        self.rets.push((0x0200d868, r));
    }
    fn fun_0200d87c(&mut self, p: u32) -> u32 {
        let r = self.stubs.call(0x0200d87c, &mut [Arg::V(p)]);
        self.rets.push((0x0200d87c, r));
        p
    }
    fn fun_0200d884(&mut self, p: u32, id: u32, out: u32) -> u32 {
        let _ = &out;
        let r = self
            .stubs
            .call(0x0200d884, &mut [Arg::V(p), Arg::V(id), Arg::Ptr(out)]);
        self.rets.push((0x0200d884, r));
        r
    }
    fn fun_0200d914(&mut self, p: u32, id: u32, data: &[u8]) {
        let b2: Vec<u8> = data[..1].to_vec();
        let r = self
            .stubs
            .call(0x0200d914, &mut [Arg::V(p), Arg::V(id), Arg::In(&b2)]);
        self.rets.push((0x0200d914, r));
    }
    fn fun_0200d9e8(&mut self, p: u32, a: u32, b: u32, c: u32) -> u32 {
        let _ = &b;
        let _ = &c;
        let r = self.stubs.call(0x0200d9e8, &mut [Arg::V(p), Arg::V(a)]);
        self.rets.push((0x0200d9e8, r));
        r
    }
    fn fun_0200da14(&mut self, p: u32) {
        let r = self.stubs.call(0x0200da14, &mut [Arg::V(p)]);
        self.rets.push((0x0200da14, r));
    }
    fn fun_0200da28(&mut self, p: u32) -> u32 {
        let r = self.stubs.call(0x0200da28, &mut [Arg::V(p)]);
        self.rets.push((0x0200da28, r));
        r
    }
    fn fun_0200da54(&mut self, p: u32) -> u32 {
        let r = self.stubs.call(0x0200da54, &mut [Arg::V(p)]);
        self.rets.push((0x0200da54, r));
        r
    }
    fn fun_0200da80(&mut self, s: u32, bp: u32) {
        let r = self.stubs.call(0x0200da80, &mut [Arg::V(s), Arg::V(bp)]);
        self.rets.push((0x0200da80, r));
    }
    fn fun_0200dac4(&mut self, s: u32, mode: u32) -> u32 {
        let r = self.stubs.call(0x0200dac4, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200dac4, r));
        r
    }
    fn fun_0200daec(&mut self, s: u32, mode: u32, v: u32) {
        let r = self
            .stubs
            .call(0x0200daec, &mut [Arg::V(s), Arg::V(mode), Arg::V(v)]);
        self.rets.push((0x0200daec, r));
    }
    fn fun_0200db08(&mut self, s: u32, mode: u32, v: u32) {
        let r = self
            .stubs
            .call(0x0200db08, &mut [Arg::V(s), Arg::V(mode), Arg::V(v)]);
        self.rets.push((0x0200db08, r));
    }
    fn fun_0200db1c(&mut self, s: u32, mode: u32) {
        let r = self.stubs.call(0x0200db1c, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200db1c, r));
    }
    fn fun_0200db30(&mut self, s: u32, mode: u32) {
        let r = self.stubs.call(0x0200db30, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200db30, r));
    }
    fn fun_0200db44(&mut self, s: u32, mode: u32) -> u32 {
        let r = self.stubs.call(0x0200db44, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200db44, r));
        r
    }
    fn fun_0200db60(&mut self, s: u32, mode: u32) {
        let r = self.stubs.call(0x0200db60, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200db60, r));
    }
    fn fun_0200db80(&mut self, s: u32, mode: u32) -> u32 {
        let r = self.stubs.call(0x0200db80, &mut [Arg::V(s), Arg::V(mode)]);
        self.rets.push((0x0200db80, r));
        r
    }
    fn fun_0200dba0(&mut self, s: u32, a: u32, b: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0200dba0, &mut [Arg::V(s), Arg::V(a), Arg::V(b)]);
        self.rets.push((0x0200dba0, r));
        s
    }
    fn fun_0200dbf0(&mut self, s: u32) {
        let r = self.stubs.call(0x0200dbf0, &mut [Arg::V(s)]);
        self.rets.push((0x0200dbf0, r));
    }
    fn fun_0200dbfc(&mut self, s: u32) {
        let r = self.stubs.call(0x0200dbfc, &mut [Arg::V(s)]);
        self.rets.push((0x0200dbfc, r));
    }
    fn fun_0200dc08(&mut self, s: u32) -> u32 {
        let r = self.stubs.call(0x0200dc08, &mut [Arg::V(s)]);
        self.rets.push((0x0200dc08, r));
        r
    }
    fn fun_0200dc0c(&mut self, s: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x0200dc0c, &mut [Arg::V(s), Arg::V(k)]);
        self.rets.push((0x0200dc0c, r));
        r
    }
    fn fun_0200dc50(&mut self, s: u32, k: u32, records: u32) {
        let b2 = self.rd(records, 180);
        let r = self
            .stubs
            .call(0x0200dc50, &mut [Arg::V(s), Arg::V(k), Arg::In(&b2)]);
        self.rets.push((0x0200dc50, r));
    }
    fn fun_0200dc94(&mut self, s: u32, p: u32) {
        let r = self.stubs.call(0x0200dc94, &mut [Arg::V(s), Arg::V(p)]);
        self.rets.push((0x0200dc94, r));
    }
    fn fun_0200de5c(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200de5c, &mut [Arg::V(x)]);
        self.rets.push((0x0200de5c, r));
        x
    }
    fn fun_0200de64(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200de64, &mut [Arg::V(x)]);
        self.rets.push((0x0200de64, r));
        x
    }
    fn fun_0200de94(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200de94, &mut [Arg::V(x)]);
        self.rets.push((0x0200de94, r));
        x
    }
    fn fun_0200dea8(&mut self, x: u32, rec: u32, n: u32, heap: u32) -> u32 {
        let _ = &heap;
        let r = self
            .stubs
            .call(0x0200dea8, &mut [Arg::V(x), Arg::V(rec), Arg::V(n)]);
        self.rets.push((0x0200dea8, r));
        r
    }
    fn fun_0200df40(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200df40, &mut [Arg::V(x)]);
        self.rets.push((0x0200df40, r));
        r
    }
    fn fun_0200df4c(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0200df4c, &mut [Arg::V(x)]);
        self.rets.push((0x0200df4c, r));
        r
    }
    fn fun_0200df58(&mut self, x: u32, heap: u32) -> u32 {
        let r = self.stubs.call(0x0200df58, &mut [Arg::V(x), Arg::V(heap)]);
        self.rets.push((0x0200df58, r));
        r
    }
    fn fun_0200df94(&mut self, list: u32) -> u32 {
        let r = self.stubs.call(0x0200df94, &mut [Arg::V(list)]);
        self.rets.push((0x0200df94, r));
        r
    }
    fn fun_0200dfb4(&mut self, rec: u32) -> u32 {
        let r = self.stubs.call(0x0200dfb4, &mut [Arg::V(rec)]);
        self.rets.push((0x0200dfb4, r));
        r
    }
    fn fun_0200ed48(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200ed48, &mut [Arg::V(save)]);
        self.rets.push((0x0200ed48, r));
        r
    }
    fn fun_0200ed54(&mut self, x: u32, v: u32) {
        let r = self.stubs.call(0x0200ed54, &mut [Arg::V(x), Arg::V(v)]);
        self.rets.push((0x0200ed54, r));
    }
    fn fun_0200fb24(&mut self, r0: u32) -> u32 {
        match self.cur {
            0x021CA5C4 => {
                let r = self.stubs.call(0x0200fb24, &mut [Arg::V(r0)]);
                self.rets.push((0x0200fb24, r));
                r
            }
            _ => {
                let _ = &r0;
                let r = self.stubs.call(0x0200fb24, &mut []);
                self.rets.push((0x0200fb24, r));
                r
            }
        }
    }
    fn fun_0200fb38(&mut self, x: u32) {
        let r = self.stubs.call(0x0200fb38, &mut [Arg::V(x)]);
        self.rets.push((0x0200fb38, r));
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)]);
        self.rets.push((0x0201210c, r));
        r
    }
    fn fun_02012124(&mut self, gamesys: u32, x: u32) {
        let r = self
            .stubs
            .call(0x02012124, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x02012124, r));
    }
    fn fun_02012134(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x02012134, &mut [Arg::V(gamesys)]);
        self.rets.push((0x02012134, r));
        r
    }
    fn fun_020124ac(&mut self, gamesys: u32, id: u32, f: u32, data: &[u32; 4]) -> u32 {
        let b3: Vec<u8> = data.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x020124ac,
            &mut [Arg::V(gamesys), Arg::V(id), Arg::V(f), Arg::In(&b3)],
        );
        self.rets.push((0x020124ac, r));
        r
    }
    fn fun_020127a4(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x020127a4, &mut [Arg::V(gamedata)]);
        self.rets.push((0x020127a4, r));
        r
    }
    fn fun_0201289c(&mut self, gamedata: u32, rec: &[u8; 0x1C]) {
        let b1: Vec<u8> = rec.to_vec();
        let r = self
            .stubs
            .call(0x0201289c, &mut [Arg::V(gamedata), Arg::In(&b1)]);
        self.rets.push((0x0201289c, r));
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012934, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012934, r));
        r
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012944, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012944, r));
        r
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ebc, r));
        r
    }
    fn fun_02012ed8(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ed8, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ed8, r));
        r
    }
    fn fun_02012ee4(&mut self, gamedata: u32, w: u32) {
        let r = self
            .stubs
            .call(0x02012ee4, &mut [Arg::V(gamedata), Arg::V(w)]);
        self.rets.push((0x02012ee4, r));
    }
    fn fun_020130b4(&mut self, x: u32) {
        let r = self.stubs.call(0x020130b4, &mut [Arg::V(x)]);
        self.rets.push((0x020130b4, r));
    }
    fn fun_02014184(&mut self, rec: &mut [u8; 0x1C], a: u32, b: u32, c: u32, d: u32, e: u32) {
        let mut b0: Vec<u8> = rec.to_vec();
        let r = self.stubs.call(
            0x02014184,
            &mut [
                Arg::Out(&mut b0),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
            ],
        );
        self.rets.push((0x02014184, r));
        rec.copy_from_slice(&b0);
    }
    fn fun_02014808(
        &mut self,
        gamesys: u32,
        fm: u32,
        id: u32,
        procs: u32,
        work: u32,
        a: u32,
        b: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x02014808,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(id),
                Arg::V(procs),
                Arg::V(work),
                Arg::V(a),
                Arg::V(b),
            ],
        );
        self.rets.push((0x02014808, r));
        r
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)]);
        self.rets.push((0x02017e1c, r));
        r
    }
    fn fun_02019ed4(&mut self, a: u32, party: u32) -> u32 {
        let r = self.stubs.call(0x02019ed4, &mut [Arg::V(a), Arg::V(party)]);
        self.rets.push((0x02019ed4, r));
        r
    }
    fn fun_0201a354(&mut self, k: u32, heap: u32) -> u32 {
        let r = self.stubs.call(0x0201a354, &mut [Arg::V(k), Arg::V(heap)]);
        self.rets.push((0x0201a354, r));
        r
    }
    fn fun_0201a920(&mut self, heap: u32) -> u32 {
        let r = self.stubs.call(0x0201a920, &mut [Arg::V(heap)]);
        self.rets.push((0x0201a920, r));
        r
    }
    fn fun_0201a954(&mut self, party: u32, n: u32) {
        let r = self.stubs.call(0x0201a954, &mut [Arg::V(party), Arg::V(n)]);
        self.rets.push((0x0201a954, r));
    }
    fn fun_0201a98c(&mut self, party: u32, mon: u32) {
        let r = self
            .stubs
            .call(0x0201a98c, &mut [Arg::V(party), Arg::V(mon)]);
        self.rets.push((0x0201a98c, r));
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        let r = self.stubs.call(0x0201aa18, &mut [Arg::V(party)]);
        self.rets.push((0x0201aa18, r));
        r
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x0201ab28, &mut [Arg::V(party), Arg::V(i)]);
        self.rets.push((0x0201ab28, r));
        r
    }
    fn fun_0202b578(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0202b578, &mut [Arg::V(x)]);
        self.rets.push((0x0202b578, r));
        r
    }
    fn fun_0202c0d0(&mut self, x: u32, i: u32) {
        let r = self.stubs.call(0x0202c0d0, &mut [Arg::V(x), Arg::V(i)]);
        self.rets.push((0x0202c0d0, r));
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        let r = self.stubs.call(
            0x02030734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(flag),
                Arg::V(file),
                Arg::V(line),
            ],
        );
        self.rets.push((0x02030734, r));
        r
    }
    fn fun_020307b0(&mut self, p: u32) {
        let r = self.stubs.call(0x020307b0, &mut [Arg::V(p)]);
        self.rets.push((0x020307b0, r));
    }
    fn fun_02039338(&mut self) -> u32 {
        let r = self.stubs.call(0x02039338, &mut []);
        self.rets.push((0x02039338, r));
        r
    }
    fn fun_0203cdbc(&mut self) -> u32 {
        let r = self.stubs.call(0x0203cdbc, &mut []);
        self.rets.push((0x0203cdbc, r));
        r
    }
    fn fun_0203ceac(&mut self, x: u32) {
        let r = self.stubs.call(0x0203ceac, &mut [Arg::V(x)]);
        self.rets.push((0x0203ceac, r));
    }
    fn fun_0203d1b4(&mut self) -> u32 {
        let r = self.stubs.call(0x0203d1b4, &mut []);
        self.rets.push((0x0203d1b4, r));
        r
    }
    fn fun_0203d378(&mut self) -> u32 {
        let r = self.stubs.call(0x0203d378, &mut []);
        self.rets.push((0x0203d378, r));
        r
    }
    fn fun_0203d7bc(&mut self, x: u32) {
        let r = self.stubs.call(0x0203d7bc, &mut [Arg::V(x)]);
        self.rets.push((0x0203d7bc, r));
    }
    fn fun_02087bcc(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02087bcc, &mut [Arg::V(x)]);
        self.rets.push((0x02087bcc, r));
        r
    }
    fn fun_02158f54(&mut self, sw: u32) -> u32 {
        let r = self.stubs.call(0x02158f54, &mut [Arg::V(sw)]);
        self.rets.push((0x02158f54, r));
        r
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        let r = self.stubs.call(0x02158f58, &mut [Arg::V(sw)]);
        self.rets.push((0x02158f58, r));
        r
    }
    fn fun_02158fe0(&mut self, sw: u32, gd: u32, id: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02158fe0, &mut [Arg::V(sw), Arg::V(gd), Arg::V(id)]);
        self.rets.push((0x02158fe0, r));
        r
    }
    fn fun_02167ab0(&mut self, w: u32) {
        let r = self.stubs.call(0x02167ab0, &mut [Arg::V(w)]);
        self.rets.push((0x02167ab0, r));
    }
    fn fun_02167b00(&mut self, w: u32) {
        let r = self.stubs.call(0x02167b00, &mut [Arg::V(w)]);
        self.rets.push((0x02167b00, r));
    }
    fn fun_02167b18(&mut self, w: u32, x: u32) {
        let r = self.stubs.call(0x02167b18, &mut [Arg::V(w), Arg::V(x)]);
        self.rets.push((0x02167b18, r));
    }
    fn fun_02167b24(&mut self, x: u32) {
        let r = self.stubs.call(0x02167b24, &mut [Arg::V(x)]);
        self.rets.push((0x02167b24, r));
    }
    fn fun_02167b34(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02167b34, &mut [Arg::V(x)]);
        self.rets.push((0x02167b34, r));
        r
    }
    fn fun_02167bfc(&mut self, w: u32, a: u32, b: u32) {
        let r = self
            .stubs
            .call(0x02167bfc, &mut [Arg::V(w), Arg::V(a), Arg::V(b)]);
        self.rets.push((0x02167bfc, r));
    }
    fn fun_02167cb4(&mut self, w: u32) -> u32 {
        let r = self.stubs.call(0x02167cb4, &mut [Arg::V(w)]);
        self.rets.push((0x02167cb4, r));
        r
    }
    fn fun_02167cf4(&mut self, w: u32, x: u32) {
        let r = self.stubs.call(0x02167cf4, &mut [Arg::V(w), Arg::V(x)]);
        self.rets.push((0x02167cf4, r));
    }
    fn fun_02167d00(&mut self, w: u32, var: u32) -> u32 {
        let r = self.stubs.call(0x02167d00, &mut [Arg::V(w), Arg::V(var)]);
        self.rets.push((0x02167d00, r));
        r
    }
    fn fun_021681c8(
        &mut self,
        x: u32,
        party: u32,
        mode: u32,
        rec: u32,
        trainer: u32,
        n: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x021681c8,
            &mut [
                Arg::V(x),
                Arg::V(party),
                Arg::V(mode),
                Arg::V(rec),
                Arg::V(trainer),
                Arg::V(n),
            ],
        );
        self.rets.push((0x021681c8, r));
        r
    }
    fn fun_021684e8(
        &mut self,
        rec: u32,
        species: u32,
        a: u32,
        b: u32,
        iv: u32,
        i: u32,
        x: u32,
        heap: u32,
    ) {
        let r = self.stubs.call(
            0x021684e8,
            &mut [
                Arg::V(rec),
                Arg::V(species),
                Arg::V(a),
                Arg::V(b),
                Arg::V(iv),
                Arg::V(i),
                Arg::V(x),
                Arg::V(heap),
            ],
        );
        self.rets.push((0x021684e8, r));
    }
    fn fun_02168878(
        &mut self,
        rec: u32,
        id: u32,
        n: u32,
        a: Buf,
        b: Buf,
        c: u32,
        heap: u32,
    ) -> u32 {
        self.stub_02168878(rec, id, n, a, b, c, heap)
    }
    fn fun_021688bc(&mut self, rec: u32, id: u32, heap: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021688bc, &mut [Arg::V(rec), Arg::V(id), Arg::V(heap)]);
        self.rets.push((0x021688bc, r));
        r
    }
    fn fun_02168aec(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02168aec, &mut [Arg::V(x)]);
        self.rets.push((0x02168aec, r));
        x
    }
    fn fun_0216c538(&mut self, w: u32, gamesys: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216c538, &mut [Arg::V(w), Arg::V(gamesys), Arg::V(k)]);
        self.rets.push((0x0216c538, r));
        w
    }
    fn fun_0216c640(&mut self, w: u32, gamesys: u32, fm: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216c640, &mut [Arg::V(w), Arg::V(gamesys), Arg::V(fm)]);
        self.rets.push((0x0216c640, r));
        w
    }
    fn fun_0216c6e8(&mut self, w: u32, gamesys: u32, a: u32, b: u32, k: u32) -> u32 {
        let r = self.stubs.call(
            0x0216c6e8,
            &mut [Arg::V(w), Arg::V(gamesys), Arg::V(a), Arg::V(b), Arg::V(k)],
        );
        self.rets.push((0x0216c6e8, r));
        w
    }
    fn fun_0216c860(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x0216c860, &mut [Arg::V(gamesys)]);
        self.rets.push((0x0216c860, r));
        gamesys
    }
    fn fun_0216cb48(&mut self, gamesys: u32, k: u32, id: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216cb48, &mut [Arg::V(gamesys), Arg::V(k), Arg::V(id)]);
        self.rets.push((0x0216cb48, r));
        gamesys
    }
    fn fun_0216ce6c(
        &mut self,
        objsys: u32,
        x: u32,
        z: u32,
        dir: u32,
        id: u32,
        code: u32,
        a: u32,
        zone: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x0216ce6c,
            &mut [
                Arg::V(objsys),
                Arg::V(x),
                Arg::V(z),
                Arg::V(dir),
                Arg::V(id),
                Arg::V(code),
                Arg::V(a),
                Arg::V(zone),
            ],
        );
        self.rets.push((0x0216ce6c, r));
        r
    }
    fn fun_0216d5b4(&mut self, obj: u32, k: u32) {
        let r = self.stubs.call(0x0216d5b4, &mut [Arg::V(obj), Arg::V(k)]);
        self.rets.push((0x0216d5b4, r));
    }
    fn fun_0216d5e0(&mut self, obj: u32, x: u32) -> u32 {
        let r = self.stubs.call(0x0216d5e0, &mut [Arg::V(obj), Arg::V(x)]);
        self.rets.push((0x0216d5e0, r));
        r
    }
    fn fun_0216d5f8(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x0216d5f8, &mut [Arg::V(obj)]);
        self.rets.push((0x0216d5f8, r));
        r
    }
    fn fun_0216d620(&mut self, obj: u32, v: u32, k: u32) {
        let r = self
            .stubs
            .call(0x0216d620, &mut [Arg::V(obj), Arg::V(v), Arg::V(k)]);
        self.rets.push((0x0216d620, r));
    }
    fn fun_0216d63c(&mut self, obj: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x0216d63c, &mut [Arg::V(obj), Arg::V(k)]);
        self.rets.push((0x0216d63c, r));
        r
    }
    fn fun_0216d88c(&mut self, obj: u32, pos: &mut [u32; 3]) {
        let mut b1: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x0216d88c, &mut [Arg::V(obj), Arg::Out(&mut b1)]);
        self.rets.push((0x0216d88c, r));
        for (j, w) in pos.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b1[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_0216dac8(&mut self, obj: u32, x: u32) -> u32 {
        let r = self.stubs.call(0x0216dac8, &mut [Arg::V(obj), Arg::V(x)]);
        self.rets.push((0x0216dac8, r));
        r
    }
    fn fun_0216daf8(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x0216daf8, &mut [Arg::V(obj)]);
        self.rets.push((0x0216daf8, r));
        r
    }
    fn fun_0216db4c(&mut self, obj: u32, x: u32) -> u32 {
        let r = self.stubs.call(0x0216db4c, &mut [Arg::V(obj), Arg::V(x)]);
        self.rets.push((0x0216db4c, r));
        r
    }
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216de94, &mut [Arg::V(objsys), Arg::V(id)]);
        self.rets.push((0x0216de94, r));
        r
    }
    fn fun_0216e014(&mut self, obj: u32, pos: &[u32; 3], dir: u32) {
        let b1: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x0216e014, &mut [Arg::V(obj), Arg::In(&b1), Arg::V(dir)]);
        self.rets.push((0x0216e014, r));
    }
    fn fun_02188c54(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c54, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c54, r));
        r
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c7c, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c7c, r));
        r
    }
    fn fun_02188c9c(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c9c, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c9c, r));
        r
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188cb4, &mut [Arg::V(fm)]);
        self.rets.push((0x02188cb4, r));
        r
    }
    fn fun_021909bc(&mut self, x: u32) {
        let r = self.stubs.call(0x021909bc, &mut [Arg::V(x)]);
        self.rets.push((0x021909bc, r));
    }
    fn fun_02190be4(&mut self, x: u32, v: u32) -> u32 {
        let r = self.stubs.call(0x02190be4, &mut [Arg::V(x), Arg::V(v)]);
        self.rets.push((0x02190be4, r));
        r
    }
    fn fun_02190c78(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02190c78, &mut [Arg::V(x)]);
        self.rets.push((0x02190c78, r));
        r
    }
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u32; 3]) {
        let mut b1: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x021a3038, &mut [Arg::V(player), Arg::Out(&mut b1)]);
        self.rets.push((0x021a3038, r));
        for (j, w) in pos.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b1[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_021a3050(&mut self, player: u32) -> u32 {
        let r = self.stubs.call(0x021a3050, &mut [Arg::V(player)]);
        self.rets.push((0x021a3050, r));
        r
    }
    fn fun_021a30b0(&mut self, player: u32) -> u32 {
        let r = self.stubs.call(0x021a30b0, &mut [Arg::V(player)]);
        self.rets.push((0x021a30b0, r));
        r
    }
    fn fun_021a3190(&mut self, x: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x021a3190, &mut [Arg::V(x), Arg::V(k)]);
        self.rets.push((0x021a3190, r));
        r
    }
    fn fun_021a31a4(&mut self, x: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x021a31a4, &mut [Arg::V(x), Arg::V(k)]);
        self.rets.push((0x021a31a4, r));
        x
    }
    fn fun_021c9924(&mut self, gamesys: u32) {
        let r = self.stubs.call(0x021c9924, &mut [Arg::V(gamesys)]);
        self.rets.push((0x021c9924, r));
    }
    fn fun_021cd48c(
        &mut self,
        status: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x021cd48c,
            &mut [
                Arg::V(status),
                Arg::V(fm),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
                Arg::V(f),
            ],
        );
        self.rets.push((0x021cd48c, r));
        r
    }
    fn fun_021cd5a4(&mut self, menu: u32) {
        let r = self.stubs.call(0x021cd5a4, &mut [Arg::V(menu)]);
        self.rets.push((0x021cd5a4, r));
    }
    fn fun_021cd688(&mut self, menu: u32) -> u32 {
        let r = self.stubs.call(0x021cd688, &mut [Arg::V(menu)]);
        self.rets.push((0x021cd688, r));
        r
    }
    fn fun_021d024c(&mut self, x: u32, a: u32) -> u32 {
        let r = self.stubs.call(0x021d024c, &mut [Arg::V(x), Arg::V(a)]);
        self.rets.push((0x021d024c, r));
        r
    }
    fn fun_021d028c(&mut self, x: u32, a: u32) -> u32 {
        let r = self.stubs.call(0x021d028c, &mut [Arg::V(x), Arg::V(a)]);
        self.rets.push((0x021d028c, r));
        r
    }
    fn fun_021eee78(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x021eee78, &mut [Arg::V(x)]);
        self.rets.push((0x021eee78, r));
        r
    }
    fn fun_021f37c0(&mut self, fm: u32, a: u32, rec: u32) -> u32 {
        let _ = &fm;
        let _ = &a;
        let _ = &rec;
        let r = self.stubs.call(0x021f37c0, &mut []);
        self.rets.push((0x021f37c0, r));
        r
    }
    fn fun_021f380c(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x021f380c, &mut [Arg::V(fm)]);
        self.rets.push((0x021f380c, r));
        r
    }
    fn fun_021f388c(&mut self, fm: u32) {
        let r = self.stubs.call(0x021f388c, &mut [Arg::V(fm)]);
        self.rets.push((0x021f388c, r));
    }
}
// gen:rom-end

/// Run `f` on the case's memory: every non-empty argument buffer at its address; the buffers are put back afterwards.
fn with_rom(addr: u32, c: &mut Case, f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(i);
        }
    }
    // Command 346 reads W + 0x8C + 0x120 * operand (sub-command 0x134) past the ctx arena: zeroed emulator RAM behind the
    // argument buffers (searched after them).
    if addr == 0x021C_A5C4 {
        mem.add_window(0x02E2_0000, vec![0; 0x1_0000]);
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
        cur: addr,
        rets: Vec::new(),
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for (i, (_, bytes)) in keep.into_iter().zip(wins) {
        c.bufs[i] = bytes;
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(addr, c, |h, v| match addr {
        0x021C_9934 => work_create(h, v[0], v[1], v[2]) as u64,
        0x021C_9B24 => {
            work_free(h, v[0], v[1]);
            0
        }
        0x021C_9B78 => {
            multi_mode(h, v[0]);
            0
        }
        0x021C_9BCC => {
            play_save(h, v[0]);
            0
        }
        0x021C_9C54 => {
            load_party(h, v[0]);
            0
        }
        0x021C_9CAC => {
            play_save_rest(h, v[0]);
            0
        }
        0x021C_9D30 => saved_mode(h, v[0]) as u64,
        0x021C_9D70 => {
            record_win(h, v[0], v[1]);
            0
        }
        0x021C_9DB0 => bp_reward(h, v[0]) as u64,
        0x021C_9E8C => {
            pick_trainers(h, v[0]);
            0
        }
        0x021C_9F44 => party_from_select(h, v[0]) as u64,
        0x021C_9FD0 => round_7_reached(h, v[0]) as u64,
        0x021C_9FFC => {
            record_loss(h, v[0]);
            0
        }
        0x021C_A048 => {
            clear_flags(h, v[0]);
            0
        }
        0x021C_A05C => {
            build_trainers(h, v[0]);
            0
        }
        0x021C_A150 => trainer_x(h, v[0], v[1]) as u64,
        0x021C_A164 => level_step(h, v[0], v[1], v[2]) as u64,
        0x021C_A1E4 => {
            alloc_party(h, v[0]);
            0
        }
        0x021C_A210 => party(h, v[0]) as u64,
        0x021C_A234 => streak(h, v[0]) as u64,
        0x021C_A238 => {
            set_streak(h, v[0], v[1] as u16);
            0
        }
        0x021C_A23C => {
            inc_streak(h, v[0]);
            0
        }
        0x021C_A250 => {
            zero_streak(h, v[0]);
            0
        }
        0x021C_A258 => {
            streak_from(h, v[0], v[1], v[2]);
            0
        }
        0x021C_A270 => party_size(v[0]) as u64,
        0x021C_A2A4 => list_has(h, v[0], v[1], v[2]) as u64,
        0x021C_A2CC => streak_rounds(v[0]) as u64,
        0x021C_A2DC => {
            save_records(h, v[0], v[1]);
            0
        }
        0x021C_A338 => {
            frontier_trainers(h, v[0], v[1]);
            0
        }
        0x021C_A3B4 => {
            mon_record(h, v[0], v[1]);
            0
        }
        0x021C_A4C0 => {
            save_party_records(h, v[0], v[1], v[2]);
            0
        }
        0x021C_B6DC => wait_6dc(h, v[0], v[1]) as u64,
        0x021C_B700 => wait_700(h, v[0], v[1]) as u64,
        0x021C_B754 => wait_754(h, v[0], v[1]) as u64,
        0x021C_B7E8 => wait_7e8(h, v[0], v[1]) as u64,
        0x021C_B86C => wait_86c(h, v[0], v[1]) as u64,
        0x021C_B8AC => event_disconnect(h, v[0], v[1], v[2]) as u64,
        0x021C_B914 => disconnect_create(h, v[0], v[1]) as u64,
        0x021C_B934 => event_934(h, v[0], v[1], v[2]) as u64,
        0x021C_B9BC => event_934_create(h, v[0], v[1], v[2], v[3]) as u64,
        0x021C_B9E4 => {
            place_npcs(h, v[0], v[1], v[2], v[3]);
            0
        }
        0x021C_BBA0 => obj_npc_id(h, v[0], v[1]) as u64,
        0x021C_BBAC => npc_has_talk(h, v[0], v[1]) as u64,
        0x021C_BBD0 => {
            let (a, b, t) = npc_talk(h, v[0], v[1]);
            h.w16(v[2], a);
            h.w16(v[3], b);
            h.w16(v[4], t);
            0
        }
        0x021C_BC14 => wait_c14(h, v[0], v[1]) as u64,
        0x021C_BC94 => mode_kind(v[0]) as u64,
        0x021C_BCC4 => zone_kind(h, v[0]) as u64,
        0x021C_BD08 => battle_setup(h, v[0], v[1]) as u64,
        0x021C_BD84 => w_74c(v[0]) as u64,
        0x021C_BD90 => trainer_id(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021C_BED8 => make_trainer(
            h,
            v[0],
            v[1],
            v[2],
            v[3],
            Buf::Addr(v[4]),
            Buf::Addr(v[5]),
            v[6],
            v[7],
        ) as u64,
        0x021C_BEFC => trainer_iv(v[0]) as u64,
        0x021C_BF38 => {
            make_partner(h, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        0x021C_BF98 => rand_u16(h, 0) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_bsubway`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_bsubway",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_bsubway.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// Overlay 21 of the ROM build (`$BLACK_ROM_BUILD/OVY_21.sbin`, loaded at 0x02187F00), or `None` (skip) when it is missing
/// and `REQUIRE_ROM` is not set.
fn overlay21() -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_21.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_21.sbin is missing");
    }
    got
}

/// The ROM tables, the command handlers (the table 0x0217062C is overlay 10) and the event / wait / proc literals are the
/// build's.
#[test]
fn tables_match_rom() {
    for &(_, a) in COMMANDS {
        assert!(PORTED.contains(&a), "{a:#x}");
    }
    let Some(ovy) = overlay21() else {
        return;
    };
    let o = |a: u32| (a - 0x0218_7F00) as usize;
    let rd = |a: u32| u32::from_le_bytes(ovy[o(a)..o(a) + 4].try_into().unwrap());
    let a = o(ROM_TABLES_ADDR);
    assert_eq!(&ovy[a..a + ROM_TABLES.len()], &ROM_TABLES[..]);
    for (lit, val) in [
        (0x021C_B930, EVENT_DISCONNECT),
        (0x021C_B9E0, EVENT_934),
        (0x021C_B23C, EVENT_67),
        (0x021C_B5E4, WAIT_C14),
        (0x021C_B600, WAIT_700),
        (0x021C_B604, WAIT_754),
        (0x021C_B608, WAIT_7E8),
        (0x021C_B60C, WAIT_6DC),
        (0x021C_B614, WAIT_86C),
        (0x021C_B5F0, PROC_15C.0),
        (0x021C_B5F4, PROC_15C.1),
        (0x021C_B61C, PROC_198.0),
        (0x021C_B620, PROC_198.1),
        (0x021C_9B0C, W_SIZE),
        (0x021C_9B10, FILE_BSUBWAY),
        (0x021C_9B14, W_MAGIC),
        (0x021C_A538, LINE_RECORDS),
        (0x021C_9E84, T_BP_LEVEL),
        (0x021C_9E88, T_BP_ROWS),
        (0x021C_A1E0, T_LEVEL_STEPS),
        (0x021C_AE88, T_BY_VALUE),
        (0x021C_AE8C, T_U32_88),
        (0x021C_AE90, T_REC_70),
        (0x021C_B22C, T_REC_50),
        (0x021C_B230, T_U32_AC),
        (0x021C_B5E8, T_BY_MODE),
        (0x021C_BB88, T_PLACES),
        (0x021C_BB8C, T_OBJ_A),
        (0x021C_BB90, T_OBJ_B),
        (0x021C_BB94, T_NPCS),
        (0x021C_BB9C, T_LEVELS),
        (0x021C_BEC8, T_IDS_SINGLE),
        (0x021C_BECC, T_IDS_SINGLE_BOSS),
        (0x021C_BED0, T_IDS_MULTI),
        (0x021C_BED4, T_IDS_MULTI_BOSS),
    ] {
        assert_eq!(rd(lit), val, "{lit:#x}");
    }
    let main = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_10.sbin")).ok());
    if let Some(ov10) = main {
        for &(id, a) in COMMANDS {
            let off = (0x0217_062C + 4 * u32::from(id) - 0x0215_5160) as usize;
            let w = u32::from_le_bytes(ov10[off..off + 4].try_into().unwrap());
            assert_eq!(w, a | 1, "command {id}");
        }
    }
}
