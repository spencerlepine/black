use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021867A8, "5e3185fff5704316"),
    (0x021867B4, "82198230fcb9242c"),
    (0x021869AC, "92508d940ac0392a"),
    (0x021869B0, "1c89e7f5d54ca891"),
    (0x021869B8, "5e0bdbea7039f000"),
    (0x021869BC, "9445a7a36f32e14d"),
    (0x021869D0, "17b6dcfa35821fa3"),
    (0x021869D8, "acef319dbd8cc0d4"),
    (0x02186BE4, "3e1619e5477dc2a3"),
    (0x02186C44, "834b2c51574c5c1e"),
    (0x02186CC4, "304d1979f9be196a"),
    (0x02186D48, "304bbae1546cfd2c"),
    (0x02186E94, "8cc2cbc6dff3ffcc"),
    (0x02186EB8, "6a97e6496b90b654"),
    (0x02187000, "b3a2f1bd53d6a4d1"),
    (0x02187098, "9f9cd1100d540fb4"),
    (0x021870C4, "ea8d5abc3a9e1179"),
    (0x021C457C, "963fd327eaf2d3ab"),
    (0x021C459C, "dbc67575a7d64d4e"),
    (0x021C4608, "245a16b15339d5ec"),
    (0x021C4628, "87e2b6588bc2b40e"),
    (0x021C4678, "daea025e590f83af"),
    (0x021C46A4, "5dae7ed151cd04f1"),
    (0x021C812C, "24b8e11222a297f9"),
    (0x021C8170, "94373c329b8007a2"),
    (0x021C81B4, "1e3103862f3537a4"),
    (0x021C81F8, "eeb98c4c1e1640b0"),
    (0x021C8608, "b29f30303eb5a6cf"),
    (0x021C86E8, "82aa7a3122569eaf"),
    (0x021C8738, "39f392f34e6188bc"),
    (0x021C87D4, "67fdd23606d48447"),
    (0x021C8810, "9f9cf44875ac4af1"),
    (0x021C881C, "1a8ab4c066bb58e0"),
    (0x021C8848, "3e877847c9f3965f"),
    (0x021C8868, "0cd031b7d6ce6295"),
    (0x021C88AC, "41c6b3fd65a29b9b"),
    (0x021C88EC, "eb6c00c8a681e730"),
    (0x021C8960, "d80f82f1a3a234c4"),
    (0x021CC544, "6f0ab156f1566a16"),
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
    fn ptr(&self, addr: u32, k: usize) -> u32 {
        panic!("no address for {addr:#x} arg {k} under {:#x}", self.cur)
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

// gen:manual fun_021f36c0
impl Rom<'_> {
    /// `FUN_021F36C0(W)`: under command 421 the spec also logs the `r1` / `r2` the original leaves behind its stubbed
    /// allocator and overlay loader (0x1B4 from clearing W+0x1B4, 1 = the allocator's flag argument).
    fn stub_021f36c0(&mut self, w: u32) {
        let r = if self.cur == 0x021C_881C {
            self.stubs
                .call(0x021F_36C0, &mut [Arg::V(w), Arg::V(0x1B4), Arg::V(1)])
        } else {
            self.stubs.call(0x021F_36C0, &mut [Arg::V(w)])
        };
        self.rets.push((0x021F_36C0, r));
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
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a5e4, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a5e4, r));
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
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        let r = self.stubs.call(0x02159ae8, &mut [Arg::V(vm), Arg::V(ctx)]);
        self.rets.push((0x02159ae8, r));
        r
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        let r = self.stubs.call(0x02159b10, &mut [Arg::V(vm), Arg::V(ctx)]);
        self.rets.push((0x02159b10, r));
        r
    }
    fn fun_02159b38(&mut self, _ctx: u32) -> u32 {
        unreachable!("fun_02159b38: no stub in the spec")
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
    fn fun_0201296c(&mut self, _gamedata: u32) -> u32 {
        unreachable!("fun_0201296c: no stub in the spec")
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
    fn fun_02014314(&mut self, _ew: u32, _flag: u32) {
        unreachable!("fun_02014314: no stub in the spec")
    }
    fn fun_0201433c(&mut self, _ew: u32, _flag: u32) {
        unreachable!("fun_0201433c: no stub in the spec")
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
impl Misc21bHooks for Rom<'_> {
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)]);
        self.rets.push((0x0201210c, r));
        r
    }
    fn fun_02012138(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x02012138, &mut [Arg::V(gamesys)]);
        self.rets.push((0x02012138, r));
        r
    }
    fn fun_020127c4(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x020127c4, &mut [Arg::V(gamedata)]);
        self.rets.push((0x020127c4, r));
        r
    }
    fn fun_020127d0(&mut self, gamedata: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x020127d0, &mut [Arg::V(gamedata), Arg::V(k)]);
        self.rets.push((0x020127d0, r));
        r
    }
    fn fun_020127f0(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x020127f0, &mut [Arg::V(gamedata)]);
        self.rets.push((0x020127f0, r));
        r
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012934, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012934, r));
        r
    }
    fn fun_02012ed0(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ed0, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ed0, r));
        r
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)]);
        self.rets.push((0x02017e1c, r));
        r
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
    fn fun_0201ec64(&mut self, heap: u32) -> u32 {
        let r = self.stubs.call(0x0201ec64, &mut [Arg::V(heap)]);
        self.rets.push((0x0201ec64, r));
        r
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
    fn fun_02034a5c(&mut self, id: u32) {
        let r = self.stubs.call(0x02034a5c, &mut [Arg::V(id)]);
        self.rets.push((0x02034a5c, r));
    }
    fn fun_02034ac4(&mut self, id: u32) {
        let r = self.stubs.call(0x02034ac4, &mut [Arg::V(id)]);
        self.rets.push((0x02034ac4, r));
    }
    fn fun_0203f4e4(&mut self, date: &mut [u32; 3]) {
        let mut b0: Vec<u8> = date.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(0x0203f4e4, &mut [Arg::Out(&mut b0)]);
        self.rets.push((0x0203f4e4, r));
        for (j, w) in date.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b0[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32 {
        let r = self
            .stubs
            .call(0x020457b0, &mut [Arg::V(len), Arg::V(heap)]);
        self.rets.push((0x020457b0, r));
        r
    }
    fn fun_02045b38(&mut self, a: u32, b: u32, c: u32, heap: u32) -> u32 {
        let r = self.stubs.call(
            0x02045b38,
            &mut [Arg::V(a), Arg::V(b), Arg::V(c), Arg::V(heap)],
        );
        self.rets.push((0x02045b38, r));
        r
    }
    fn fun_02158344(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02158344, &mut [Arg::V(x)]);
        self.rets.push((0x02158344, r));
        r
    }
    fn fun_02158434(&mut self, r0: u32) -> u32 {
        match self.cur {
            0x02187098 => {
                let r = self.stubs.call(0x02158434, &mut [Arg::V(r0)]);
                self.rets.push((0x02158434, r));
                r
            }
            _ => {
                let _ = &r0;
                let r = self.stubs.call(0x02158434, &mut []);
                self.rets.push((0x02158434, r));
                r
            }
        }
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        let r = self.stubs.call(0x02158f58, &mut [Arg::V(sw)]);
        self.rets.push((0x02158f58, r));
        r
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        let r = self.stubs.call(0x02158f74, &mut [Arg::V(sw)]);
        self.rets.push((0x02158f74, r));
        r
    }
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a5cc, r));
        r
    }
    fn fun_0215a644(&mut self, ctx: u32, task: u32) {
        let r = self
            .stubs
            .call(0x0215a644, &mut [Arg::V(ctx), Arg::V(task)]);
        self.rets.push((0x0215a644, r));
    }
    fn fun_0215a64c(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a64c, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a64c, r));
        r
    }
    fn fun_021666a4(&mut self, slot: u32) {
        let r = self.stubs.call(0x021666a4, &mut [Arg::V(slot)]);
        self.rets.push((0x021666a4, r));
    }
    fn fun_0216c204(&mut self, gamesys: u32, fm: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216c204, &mut [Arg::V(gamesys), Arg::V(fm)]);
        self.rets.push((0x0216c204, r));
        r
    }
    fn fun_0216d588(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x0216d588, &mut [Arg::V(obj)]);
        self.rets.push((0x0216d588, r));
        r
    }
    fn fun_0216d5f8(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x0216d5f8, &mut [Arg::V(obj)]);
        self.rets.push((0x0216d5f8, r));
        r
    }
    fn fun_0216d88c(&mut self, obj: u32, pos: u32) {
        let mut b1 = self.rd(pos, 12);
        let r = self
            .stubs
            .call(0x0216d88c, &mut [Arg::V(obj), Arg::Out(&mut b1)]);
        self.rets.push((0x0216d88c, r));
        self.wr(pos, &b1);
    }
    fn fun_0216ddc0(&mut self, objsys: u32, x: u32, z: u32, k: u32) -> u32 {
        let r = self.stubs.call(
            0x0216ddc0,
            &mut [Arg::V(objsys), Arg::V(x), Arg::V(z), Arg::V(k)],
        );
        self.rets.push((0x0216ddc0, r));
        r
    }
    fn fun_02186090(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x02186090, &mut [Arg::V(obj)]);
        self.rets.push((0x02186090, r));
        r
    }
    fn fun_02186278(&mut self, data: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x02186278, &mut [Arg::V(data), Arg::V(k)]);
        self.rets.push((0x02186278, r));
        r
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
    fn fun_02188c80(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c80, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c80, r));
        r
    }
    fn fun_02188c84(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c84, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c84, r));
        r
    }
    fn fun_02188c98(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c98, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c98, r));
        r
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188cb4, &mut [Arg::V(fm)]);
        self.rets.push((0x02188cb4, r));
        r
    }
    fn fun_02188cd0(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188cd0, &mut [Arg::V(fm)]);
        self.rets.push((0x02188cd0, r));
        r
    }
    fn fun_02189a00(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02189a00, &mut [Arg::V(fm)]);
        self.rets.push((0x02189a00, r));
        r
    }
    fn fun_0218dc70(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0218dc70, &mut [Arg::V(x)]);
        self.rets.push((0x0218dc70, r));
        r
    }
    fn fun_021938f4(&mut self, e: u32) -> u32 {
        let r = self.stubs.call(0x021938f4, &mut [Arg::V(e)]);
        self.rets.push((0x021938f4, r));
        r
    }
    fn fun_02193904(&mut self, e: u32) {
        let r = self.stubs.call(0x02193904, &mut [Arg::V(e)]);
        self.rets.push((0x02193904, r));
    }
    fn fun_0219390c(&mut self, e: u32, rec: u32) -> u32 {
        let r = self.stubs.call(0x0219390c, &mut [Arg::V(e), Arg::V(rec)]);
        self.rets.push((0x0219390c, r));
        r
    }
    fn fun_0219391c(&mut self, e: u32) -> u32 {
        let r = self.stubs.call(0x0219391c, &mut [Arg::V(e)]);
        self.rets.push((0x0219391c, r));
        r
    }
    fn fun_02193b40(&mut self, e: u32, rec: u32) {
        let r = self.stubs.call(0x02193b40, &mut [Arg::V(e), Arg::V(rec)]);
        self.rets.push((0x02193b40, r));
    }
    fn fun_02193b68(&mut self, e: u32, ws: u32, rec: u32, v: u32, heap: u32) {
        let r = self.stubs.call(
            0x02193b68,
            &mut [Arg::V(e), Arg::V(ws), Arg::V(rec), Arg::V(v), Arg::V(heap)],
        );
        self.rets.push((0x02193b68, r));
    }
    fn fun_02193bc4(&mut self, e: u32, rec: u32) {
        let r = self.stubs.call(0x02193bc4, &mut [Arg::V(e), Arg::V(rec)]);
        self.rets.push((0x02193bc4, r));
    }
    fn fun_02193be4(&mut self, e: u32, rec: u32) -> u32 {
        let r = self.stubs.call(0x02193be4, &mut [Arg::V(e), Arg::V(rec)]);
        self.rets.push((0x02193be4, r));
        r
    }
    fn fun_02193c00(&mut self) -> u32 {
        let r = self.stubs.call(0x02193c00, &mut []);
        self.rets.push((0x02193c00, r));
        r
    }
    fn fun_02193c04(&mut self, person: u32) -> u32 {
        let r = self.stubs.call(0x02193c04, &mut [Arg::V(person)]);
        self.rets.push((0x02193c04, r));
        r
    }
    fn fun_021a3050(&mut self, player: u32) -> u32 {
        let r = self.stubs.call(0x021a3050, &mut [Arg::V(player)]);
        self.rets.push((0x021a3050, r));
        r
    }
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        let r = self.stubs.call(0x021a3074, &mut [Arg::V(player)]);
        self.rets.push((0x021a3074, r));
        r
    }
    fn fun_021a3364(&mut self, player: u32, dir: u32, pos: &mut [u32; 3]) {
        let mut b2: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x021a3364,
            &mut [Arg::V(player), Arg::V(dir), Arg::Out(&mut b2)],
        );
        self.rets.push((0x021a3364, r));
        for (j, w) in pos.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b2[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_021a3460(&mut self, player: u32, x: &mut [u8; 4], y: &mut [u8; 2], z: &mut [u8; 2]) {
        let mut b1: Vec<u8> = x.to_vec();
        let mut b2: Vec<u8> = y.to_vec();
        let mut b3: Vec<u8> = z.to_vec();
        let r = self.stubs.call(
            0x021a3460,
            &mut [
                Arg::V(player),
                Arg::Out(&mut b1),
                Arg::Out(&mut b2),
                Arg::Out(&mut b3),
            ],
        );
        self.rets.push((0x021a3460, r));
        x.copy_from_slice(&b1);
        y.copy_from_slice(&b2);
        z.copy_from_slice(&b3);
    }
    fn fun_021ab0d4(&mut self, map: u32, pos: &[u32; 3]) -> u32 {
        let b1: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x021ab0d4, &mut [Arg::V(map), Arg::In(&b1)]);
        self.rets.push((0x021ab0d4, r));
        r
    }
    fn fun_021c1024(&mut self, player: u32, dir: u32, x: u32, heap: u32) -> u32 {
        let r = self.stubs.call(
            0x021c1024,
            &mut [Arg::V(player), Arg::V(dir), Arg::V(x), Arg::V(heap)],
        );
        self.rets.push((0x021c1024, r));
        r
    }
    fn fun_021c1094(&mut self, task: u32) -> u32 {
        let r = self.stubs.call(0x021c1094, &mut [Arg::V(task)]);
        self.rets.push((0x021c1094, r));
        r
    }
    fn fun_021c10a0(&mut self, task: u32) {
        let r = self.stubs.call(0x021c10a0, &mut [Arg::V(task)]);
        self.rets.push((0x021c10a0, r));
    }
    fn fun_021c1228(&mut self, player: u32, dir: u32, v: u32, heap: u32) -> u32 {
        let r = self.stubs.call(
            0x021c1228,
            &mut [Arg::V(player), Arg::V(dir), Arg::V(v), Arg::V(heap)],
        );
        self.rets.push((0x021c1228, r));
        r
    }
    fn fun_021c12c4(&mut self, task: u32) -> u32 {
        let r = self.stubs.call(0x021c12c4, &mut [Arg::V(task)]);
        self.rets.push((0x021c12c4, r));
        r
    }
    fn fun_021c12d0(&mut self, task: u32) {
        let r = self.stubs.call(0x021c12d0, &mut [Arg::V(task)]);
        self.rets.push((0x021c12d0, r));
    }
    fn fun_021c1cb0(&mut self, gamesys: u32, x: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021c1cb0, &mut [Arg::V(gamesys), Arg::V(x), Arg::V(k)]);
        self.rets.push((0x021c1cb0, r));
        r
    }
    fn fun_021c1d68(&mut self, gamesys: u32, x: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021c1d68, &mut [Arg::V(gamesys), Arg::V(x), Arg::V(k)]);
        self.rets.push((0x021c1d68, r));
        r
    }
    fn fun_021c30b8(&mut self, gamesys: u32, x: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021c30b8, &mut [Arg::V(gamesys), Arg::V(x), Arg::V(k)]);
        self.rets.push((0x021c30b8, r));
        r
    }
    fn fun_021c9550(&mut self, obj: u32, x: u32) {
        let r = self.stubs.call(0x021c9550, &mut [Arg::V(obj), Arg::V(x)]);
        self.rets.push((0x021c9550, r));
    }
    fn fun_021d2418(&mut self, gamesys: u32, fm: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021d2418, &mut [Arg::V(gamesys), Arg::V(fm)]);
        self.rets.push((0x021d2418, r));
        r
    }
    fn fun_021d2440(&mut self, gamesys: u32, fm: u32, k: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021d2440, &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(k)]);
        self.rets.push((0x021d2440, r));
        r
    }
    fn fun_021f36c0(&mut self, w: u32) {
        self.stub_021f36c0(w)
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
        0x0218_67A8 => wfbc_rec_hi4(h, v[0]) as u64,
        0x0218_67B4 => wfbc_rec_near(h, v[0]) as u64,
        0x0218_69AC => wfbc_count_a(h, v[0]) as u64,
        0x0218_69B0 => {
            wfbc_count_a_inc(h, v[0]);
            0
        }
        0x0218_69B8 => wfbc_count_b(h, v[0]) as u64,
        0x0218_69BC => {
            wfbc_count_b_add10(h, v[0]);
            0
        }
        0x0218_69D0 => wfbc_count_b2(h, v[0]) as u64,
        0x0218_69D8 => wfbc_count_a2(h, v[0]) as u64,
        0x0218_70C4 => wfbc_talk_create(h, v[0], v[1]) as u64,
        0x021C_457C => wait_197(h, v[0], v[1]) as u64,
        0x021C_4608 => wait_198(h, v[0], v[1]) as u64,
        0x021C_8608 => boat_alloc(h) as u64,
        0x021C_86E8 => {
            boat_time_add(h, v[0], v[1], v[2]);
            0
        }
        0x021C_8738 => boat_query(h, v[0], v[1]) as u64,
        0x021C_87D4 => {
            boat_set(h, v[0], v[1], v[2]);
            0
        }
        0x021C_8810 => {
            boat_flag(h, v[0]);
            0
        }
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc21b`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc21b",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc21b.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// The ROM build file `name`, or `None` (skip) when it is missing and `REQUIRE_ROM` is not set.
fn rom_file(name: &str) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/{name}")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/{name} is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (overlay 10, 0x0217062C), the key table
/// of command 576 and the event / wait / allocation literals are the build's.
#[test]
fn tables_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0);
    }
    for &(_, a) in COMMANDS {
        assert!(PORTED.contains(&a), "{a:#x}");
    }
    let (Some(ov10), Some(ov20), Some(ov21)) = (
        rom_file("OVY_10.sbin"),
        rom_file("OVY_20.sbin"),
        rom_file("OVY_21.sbin"),
    ) else {
        return;
    };
    let rd = |m: &[u8], base: u32, a: u32| {
        let o = (a - base) as usize;
        u32::from_le_bytes(m[o..o + 4].try_into().unwrap())
    };
    for &(id, a) in COMMANDS {
        assert_eq!(
            rd(&ov10, 0x0215_5160, 0x0217_062C + 4 * u32::from(id)),
            a | 1,
            "command {id}"
        );
    }
    for (k, &(key, kind)) in WIPE_KEYS.iter().enumerate() {
        let w = rd(&ov21, 0x0218_7F00, WIPE_KEYS_ADDR + 4 * k as u32);
        assert_eq!((w as u16, (w >> 16) as u16), (key, kind));
    }
    for (lit, val) in [
        (0x021C_4604, WAIT_197),
        (0x021C_4674, WAIT_198),
        (0x021C_8260, WIPE_KEYS_ADDR),
        (0x021C_8658, FILE_BOAT),
        (0x021C_865C, OVERLAY_BOAT),
        (0x021C_8734, BOAT_TIME_UNIT as u32),
    ] {
        assert_eq!(rd(&ov21, 0x0218_7F00, lit), val, "{lit:#x}");
    }
    assert_eq!(rd(&ov20, 0x0218_3B60, 0x0218_71F0), EVENT_WFBC_TALK);
    assert_eq!(
        rd(&ov20, 0x0218_3B60, 0x0218_69CC),
        u32::from(WFBC_COUNT_B_CAP)
    );
}
