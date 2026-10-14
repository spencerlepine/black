use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02166658, "b0fbb028a6893ff2"),
    (0x0216666C, "b5f6e320aa1aaa18"),
    (0x021666B8, "6fa7eec0dd550e55"),
    (0x0216670C, "4d631892010c14a5"),
    (0x02166714, "c2c32965306900c1"),
    (0x02166734, "67b0180a8c025fcb"),
    (0x021667E0, "a2c3270315dbb2bb"),
    (0x02166824, "a6390818ca50fa24"),
    (0x02166834, "69d653f3bb76cf76"),
    (0x02166854, "5896aa861effb790"),
    (0x02166908, "a4652aadb1f86e60"),
    (0x02166980, "2415c38bd2535e02"),
    (0x021669C0, "a09e6da031b99837"),
    (0x02166A00, "deed87ed5f007798"),
    (0x02166D30, "15fc30e845a11ce7"),
    (0x02166E50, "c5482eca5bd1a504"),
    (0x02166F1C, "c402a6540b1396ef"),
    (0x0216AC40, "da299b93513d1305"),
    (0x0216AC90, "c693f9ab11682ffd"),
    (0x0216ACCC, "7467db16a7c88470"),
    (0x0216AD1C, "1e03a52c9b66f9cf"),
    (0x0216AD70, "204514684012f9bd"),
    (0x0216AE10, "5bd4bef1e3252813"),
    (0x0216AED0, "36050586d384c5db"),
    (0x0216AEE8, "0c5504f720b1f6d2"),
    (0x0216AF1C, "113fe82b6848346a"),
    (0x0216AF3C, "aadb8ae15f3a3d12"),
    (0x0216AF74, "e7a6f8e3784ffb43"),
    (0x0216B004, "c287542be2f9cc8e"),
    (0x0216B24C, "5b6dea7a5d4a110d"),
    (0x0216B280, "a8be31bb21061706"),
    (0x0216B2E4, "b22c394b1871c2fa"),
    (0x0216B340, "4b57daf93a809e80"),
    (0x0216B3A8, "2e0dcc312a656799"),
    (0x0216B3EC, "42256872b07f5f5a"),
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
    /// The n-th answer of stub `addr` so far.
    fn ret_of(&self, addr: u32, n: usize) -> u32 {
        self.rets
            .iter()
            .filter(|r| r.0 == addr)
            .nth(n)
            .map_or(0, |r| r.1)
    }
    /// The address the original passes for a record the port keeps in a local (`&u32` / `&mut u32` parameter `k` of the
    /// stub at `addr` that the spec logs as a value): the argument buffer of the function under test, or the stack slot
    /// of the original's frame (entry sp 0x02F00000; Z19).
    fn ptr(&self, addr: u32, k: usize) -> u32 {
        const SP: u32 = 0x02F0_0000;
        match (self.cur, addr, k) {
            // FUN_0216AEE8(ctx, &word): arg 1
            (0x0216_AEE8, 0x0217_C274, 0) => 0x02E2_0000,
            // FUN_0216AF1C(ctx, obj, heap, &word): arg 3
            (0x0216_AF1C, 0x0217_C274, 0) | (0x0216_AF1C, 0x0217_C2A0, 2) => 0x02E4_0000,
            // command 557: push {r3-r7, lr}; the word at sp + 0
            (0x0216_AD1C, 0x0217_C274, 0) | (0x0216_AD1C, 0x0217_C2A0, 2) => SP - 0x18,
            // command 555: push {r4-r7, lr}; sub sp, 0xC; sub-command 3 uses sp + 8, 4 uses sp + 4
            (0x0216_AD70, 0x0217_C274, 0) => {
                if self.ret_of(0x0215_9B10, 0) == 3 {
                    SP - 0x20 + 8
                } else {
                    SP - 0x20 + 4
                }
            }
            _ => panic!("no address for {:#x} arg {k} under {:#x}", addr, self.cur),
        }
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

// gen:rom-begin (tools: /tmp/blk/f8f/gen.py, from the trait and the spec's stub lines)
impl CoreHooks for Rom<'_> {
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        let r = self.stubs.call(0x02011330, &mut [Arg::V(vm)]);
        self.rets.push((0x02011330, r));
        r
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
impl Misc10bHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        let r = self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)]);
        self.rets.push((0x0215a5cc, r));
        r
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        let r = self.stubs.call(0x02158f74, &mut [Arg::V(sw)]);
        self.rets.push((0x02158f74, r));
        r
    }
    fn fun_0215901c(&mut self, sw: u32, gd: u32, id: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0215901c, &mut [Arg::V(sw), Arg::V(gd), Arg::V(id)]);
        self.rets.push((0x0215901c, r));
        r
    }
    fn fun_02158344(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02158344, &mut [Arg::V(x)]);
        self.rets.push((0x02158344, r));
        r
    }
    fn fun_02161cdc(&mut self, e: u32, gamesys: u32, a: u32, b: u32) {
        let r = self.stubs.call(
            0x02161cdc,
            &mut [Arg::V(e), Arg::V(gamesys), Arg::V(a), Arg::V(b)],
        );
        self.rets.push((0x02161cdc, r));
    }
    fn fun_0216628c(&mut self, y: u32, v: u32) -> u32 {
        let r = self.stubs.call(0x0216628c, &mut [Arg::V(y), Arg::V(v)]);
        self.rets.push((0x0216628c, r));
        r
    }
    fn fun_021664cc(&mut self, heap: u32, gamesys: u32, out: &mut u32) -> u32 {
        let mut b2: Vec<u8> = out.to_le_bytes().to_vec();
        let r = self.stubs.call(
            0x021664cc,
            &mut [Arg::V(heap), Arg::V(gamesys), Arg::Out(&mut b2)],
        );
        self.rets.push((0x021664cc, r));
        *out = u32::from_le_bytes(b2[..4].try_into().unwrap());
        r
    }
    fn fun_021665d0(&mut self, gamesys: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021665d0, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x021665d0, r));
        r
    }
    fn fun_021665f0(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021665f0, &mut [Arg::V(gamesys), Arg::V(a), Arg::V(b)]);
        self.rets.push((0x021665f0, r));
        r
    }
    fn fun_02166618(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02166618, &mut [Arg::V(x)]);
        self.rets.push((0x02166618, r));
        x
    }
    fn fun_0216662c(&mut self, x: u32) -> u32 {
        match self.cur {
            0x0216AD70 => {
                let r = self.stubs.call(0x0216662c, &mut [Arg::V(x)]);
                self.rets.push((0x0216662c, r));
                r
            }
            _ => {
                let _ = &x;
                let r = self.stubs.call(0x0216662c, &mut []);
                self.rets.push((0x0216662c, r));
                r
            }
        }
    }
    fn fun_0216d888(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x0216d888, &mut [Arg::V(x)]);
        self.rets.push((0x0216d888, r));
        r
    }
    fn fun_0216d9a0(&mut self, objsys: u32) {
        let r = self.stubs.call(0x0216d9a0, &mut [Arg::V(objsys)]);
        self.rets.push((0x0216d9a0, r));
    }
    fn fun_0216d9e4(&mut self, objsys: u32) {
        let r = self.stubs.call(0x0216d9e4, &mut [Arg::V(objsys)]);
        self.rets.push((0x0216d9e4, r));
    }
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0216de94, &mut [Arg::V(objsys), Arg::V(id)]);
        self.rets.push((0x0216de94, r));
        r
    }
    fn fun_020061e4(&mut self, se: u32) {
        let r = self.stubs.call(0x020061e4, &mut [Arg::V(se)]);
        self.rets.push((0x020061e4, r));
    }
    fn fun_02007e50(&mut self, zukan: u32, id: u32, a: u32, b: u32) {
        let r = self.stubs.call(
            0x02007e50,
            &mut [Arg::V(zukan), Arg::V(id), Arg::V(a), Arg::V(b)],
        );
        self.rets.push((0x02007e50, r));
    }
    fn fun_02007f1c(&mut self, zukan: u32, id: u32, a: u32, b: u32) {
        let r = self.stubs.call(
            0x02007f1c,
            &mut [Arg::V(zukan), Arg::V(id), Arg::V(a), Arg::V(b)],
        );
        self.rets.push((0x02007f1c, r));
    }
    fn fun_0200856c(&mut self, rec: u32) -> u32 {
        let r = self.stubs.call(0x0200856c, &mut [Arg::V(rec)]);
        self.rets.push((0x0200856c, r));
        r
    }
    fn fun_0200873c(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200873c, &mut [Arg::V(save)]);
        self.rets.push((0x0200873c, r));
        r
    }
    fn fun_02009470(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x02009470, &mut [Arg::V(save)]);
        self.rets.push((0x02009470, r));
        r
    }
    fn fun_020095c8(&mut self, block: u32) -> u32 {
        let r = self.stubs.call(0x020095c8, &mut [Arg::V(block)]);
        self.rets.push((0x020095c8, r));
        r
    }
    fn fun_0200973c(&mut self, block: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x0200973c, &mut [Arg::V(block), Arg::V(i)]);
        self.rets.push((0x0200973c, r));
        r
    }
    fn fun_02009794(&mut self, block: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x02009794, &mut [Arg::V(block), Arg::V(i)]);
        self.rets.push((0x02009794, r));
        r
    }
    fn fun_0200ca10(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200ca10, &mut [Arg::V(save)]);
        self.rets.push((0x0200ca10, r));
        r
    }
    fn fun_0200e0b0(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200e0b0, &mut [Arg::V(save)]);
        self.rets.push((0x0200e0b0, r));
        r
    }
    fn fun_0200e18c(&mut self, y: u32, v: u32) {
        let r = self.stubs.call(0x0200e18c, &mut [Arg::V(y), Arg::V(v)]);
        self.rets.push((0x0200e18c, r));
    }
    fn fun_0200e2bc(&mut self, save: u32, word: &u32, species: u32, x: u32) -> u32 {
        let b1: Vec<u8> = word.to_le_bytes().to_vec();
        let r = self.stubs.call(
            0x0200e2bc,
            &mut [Arg::V(save), Arg::In(&b1), Arg::V(species), Arg::V(x)],
        );
        self.rets.push((0x0200e2bc, r));
        r
    }
    fn fun_0200e6f0(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200e6f0, &mut [Arg::V(save)]);
        self.rets.push((0x0200e6f0, r));
        r
    }
    fn fun_0200e6fc(&mut self, x: u32, k: u32, out: &mut [u32; 3]) {
        let mut b2: Vec<u8> = out.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x0200e6fc, &mut [Arg::V(x), Arg::V(k), Arg::Out(&mut b2)]);
        self.rets.push((0x0200e6fc, r));
        for (j, w) in out.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b2[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_0200e714(&mut self, x: u32, k: u32, rec: &[u32; 3]) {
        let b2: Vec<u8> = rec.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x0200e714, &mut [Arg::V(x), Arg::V(k), Arg::In(&b2)]);
        self.rets.push((0x0200e714, r));
    }
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
    fn fun_02012374(&mut self, event: u32, child: u32) {
        let r = self
            .stubs
            .call(0x02012374, &mut [Arg::V(event), Arg::V(child)]);
        self.rets.push((0x02012374, r));
    }
    fn fun_02012444(&mut self, event: u32, id: u32, data: u32, work: u32) {
        match self.cur {
            0x02166908 | 0x02166E50 | 0x0216B004 => {
                let r = self.stubs.call(
                    0x02012444,
                    &mut [Arg::V(event), Arg::V(id), Arg::V(data), Arg::V(work)],
                );
                self.rets.push((0x02012444, r));
            }
            _ => {
                let _ = &work;
                let r = self
                    .stubs
                    .call(0x02012444, &mut [Arg::V(event), Arg::V(id), Arg::V(data)]);
                self.rets.push((0x02012444, r));
            }
        }
    }
    fn fun_020124e4(&mut self, event: u32) -> u32 {
        let r = self.stubs.call(0x020124e4, &mut [Arg::V(event)]);
        self.rets.push((0x020124e4, r));
        r
    }
    fn fun_0201292c(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x0201292c, &mut [Arg::V(gamedata)]);
        self.rets.push((0x0201292c, r));
        r
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012944, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012944, r));
        r
    }
    fn fun_020129a0(&mut self, gamedata: u32, weather: u32) {
        let r = self
            .stubs
            .call(0x020129a0, &mut [Arg::V(gamedata), Arg::V(weather)]);
        self.rets.push((0x020129a0, r));
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ebc, r));
        r
    }
    fn fun_02012ed0(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ed0, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ed0, r));
        r
    }
    fn fun_02012f38(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012f38, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012f38, r));
        r
    }
    fn fun_02012fa4(&mut self, gamedata: u32, x: u32) {
        let r = self
            .stubs
            .call(0x02012fa4, &mut [Arg::V(gamedata), Arg::V(x)]);
        self.rets.push((0x02012fa4, r));
    }
    fn fun_02012fb0(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012fb0, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012fb0, r));
        r
    }
    fn fun_020144dc(&mut self, gamesys: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x020144dc, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x020144dc, r));
        r
    }
    fn fun_020145cc(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x020145cc, &mut [Arg::V(gamesys)]);
        self.rets.push((0x020145cc, r));
        r
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)]);
        self.rets.push((0x02017e1c, r));
        r
    }
    fn fun_0201ee84(&mut self, ws: u32, i: u32, x: u32) {
        let r = self
            .stubs
            .call(0x0201ee84, &mut [Arg::V(ws), Arg::V(i), Arg::V(x)]);
        self.rets.push((0x0201ee84, r));
    }
    fn fun_02024044(&mut self, a: u32, b: u32, c: u32, d: u32, save: u32, e: u32) -> u32 {
        let r = self.stubs.call(
            0x02024044,
            &mut [
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(save),
                Arg::V(e),
            ],
        );
        self.rets.push((0x02024044, r));
        r
    }
    fn fun_020240fc(&mut self, rec: u32) {
        let r = self.stubs.call(0x020240fc, &mut [Arg::V(rec)]);
        self.rets.push((0x020240fc, r));
    }
    fn fun_02024108(&mut self, rec: u32, v: &[u32; 3]) {
        let b1: Vec<u8> = v.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x02024108, &mut [Arg::V(rec), Arg::In(&b1)]);
        self.rets.push((0x02024108, r));
    }
    fn fun_0202411c(&mut self, rec: u32) -> u32 {
        let r = self.stubs.call(0x0202411c, &mut [Arg::V(rec)]);
        self.rets.push((0x0202411c, r));
        r
    }
    fn fun_02024124(&mut self, rec: u32) -> u32 {
        let r = self.stubs.call(0x02024124, &mut [Arg::V(rec)]);
        self.rets.push((0x02024124, r));
        r
    }
    fn fun_02024128(&mut self, rec: u32, out: &mut u32) {
        let mut b1: Vec<u8> = out.to_le_bytes().to_vec();
        let r = self
            .stubs
            .call(0x02024128, &mut [Arg::V(rec), Arg::Out(&mut b1)]);
        self.rets.push((0x02024128, r));
        *out = u32::from_le_bytes(b1[..4].try_into().unwrap());
    }
    fn fun_02024134(&mut self, rec: u32, out: &mut [u32; 3]) {
        let mut b1: Vec<u8> = out.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x02024134, &mut [Arg::V(rec), Arg::Out(&mut b1)]);
        self.rets.push((0x02024134, r));
        for (j, w) in out.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b1[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_02029bcc(&mut self, gamesys: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02029bcc, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x02029bcc, r));
        r
    }
    fn fun_02029c00(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x02029c00, &mut [Arg::V(gamesys)]);
        self.rets.push((0x02029c00, r));
        gamesys
    }
    fn fun_02029cc0(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x02029cc0, &mut [Arg::V(gamesys)]);
        self.rets.push((0x02029cc0, r));
        gamesys
    }
    fn fun_0202c794(&mut self, gamesys: u32, zone: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0202c794, &mut [Arg::V(gamesys), Arg::V(zone)]);
        self.rets.push((0x0202c794, r));
        r
    }
    fn fun_020307b0(&mut self, p: u32) {
        let r = self.stubs.call(0x020307b0, &mut [Arg::V(p)]);
        self.rets.push((0x020307b0, r));
    }
    fn fun_02034a5c(&mut self, id: u32) {
        let r = self.stubs.call(0x02034a5c, &mut [Arg::V(id)]);
        self.rets.push((0x02034a5c, r));
    }
    fn fun_02034ac4(&mut self, id: u32) {
        let r = self.stubs.call(0x02034ac4, &mut [Arg::V(id)]);
        self.rets.push((0x02034ac4, r));
    }
    fn fun_0203f1fc(&mut self, mon: u32, limit: u32) -> u32 {
        let r = self
            .stubs
            .call(0x0203f1fc, &mut [Arg::V(mon), Arg::V(limit)]);
        self.rets.push((0x0203f1fc, r));
        r
    }
    fn fun_021767a0(&mut self, x: u32, gd: u32) -> u32 {
        let r = self.stubs.call(0x021767a0, &mut [Arg::V(x), Arg::V(gd)]);
        self.rets.push((0x021767a0, r));
        r
    }
    fn fun_021767f0(&mut self, x: u32, gd: u32, pos: &mut [u32; 3]) {
        let mut b2: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x021767f0, &mut [Arg::V(x), Arg::V(gd), Arg::Out(&mut b2)]);
        self.rets.push((0x021767f0, r));
        for (j, w) in pos.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b2[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_02176850(&mut self, comm: u32, x: u32) -> u32 {
        let r = self.stubs.call(0x02176850, &mut [Arg::V(comm), Arg::V(x)]);
        self.rets.push((0x02176850, r));
        r
    }
    fn fun_02176870(&mut self, comm: u32) -> u32 {
        let r = self.stubs.call(0x02176870, &mut [Arg::V(comm)]);
        self.rets.push((0x02176870, r));
        r
    }
    fn fun_0217688c(&mut self, comm: u32, x: u32) {
        match self.cur {
            0x0216AC90 | 0x0216ACCC => {
                let r = self.stubs.call(0x0217688c, &mut [Arg::V(comm), Arg::V(x)]);
                self.rets.push((0x0217688c, r));
            }
            _ => {
                let r = self.stubs.call(0x0217688c, &mut [Arg::V(comm), Arg::V(x)]);
                self.rets.push((0x0217688c, r));
            }
        }
    }
    fn fun_0217c274(&mut self, word: &mut u32, obj: u32) {
        match self.cur {
            0x0216AD1C | 0x0216AD70 => {
                let _ = &word;
                let r = self.stubs.call(
                    0x0217c274,
                    &mut [Arg::V(self.ptr(0x0217c274, 0)), Arg::V(obj)],
                );
                self.rets.push((0x0217c274, r));
            }
            _ => {
                let _ = &word;
                let r = self.stubs.call(
                    0x0217c274,
                    &mut [Arg::V(self.ptr(0x0217c274, 0)), Arg::V(obj)],
                );
                self.rets.push((0x0217c274, r));
            }
        }
    }
    fn fun_0217c294(&mut self, obj: u32) -> u32 {
        let r = self.stubs.call(0x0217c294, &mut [Arg::V(obj)]);
        self.rets.push((0x0217c294, r));
        r
    }
    fn fun_0217c2a0(&mut self, k: u32, gd: u32, word: &u32) -> u32 {
        let _ = &word;
        let r = self.stubs.call(
            0x0217c2a0,
            &mut [Arg::V(k), Arg::V(gd), Arg::V(self.ptr(0x0217c2a0, 2))],
        );
        self.rets.push((0x0217c2a0, r));
        r
    }
    fn fun_0217c37c(&mut self, fm: u32, w: u32, rec: u32, a: u32, b: u32) {
        let r = self.stubs.call(
            0x0217c37c,
            &mut [Arg::V(fm), Arg::V(w), Arg::V(rec), Arg::V(a), Arg::V(b)],
        );
        self.rets.push((0x0217c37c, r));
    }
    fn fun_02184d38(
        &mut self,
        gamesys: u32,
        fm: u32,
        zone: u32,
        pos: &[u32; 3],
        a: u32,
        b: u32,
    ) -> u32 {
        let b3: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x02184d38,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(&b3),
                Arg::V(a),
                Arg::V(b),
            ],
        );
        self.rets.push((0x02184d38, r));
        r
    }
    fn fun_021854b0(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u32; 3], dir: u32) -> u32 {
        let b3: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x021854b0,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(&b3),
                Arg::V(dir),
            ],
        );
        self.rets.push((0x021854b0, r));
        r
    }
    fn fun_02188c6c(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188c6c, &mut [Arg::V(fm)]);
        self.rets.push((0x02188c6c, r));
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
    fn fun_02189a08(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02189a08, &mut [Arg::V(x)]);
        self.rets.push((0x02189a08, r));
        r
    }
    fn fun_0219c630(&mut self, objsys: u32) -> u32 {
        let r = self.stubs.call(0x0219c630, &mut [Arg::V(objsys)]);
        self.rets.push((0x0219c630, r));
        r
    }
    fn fun_021a1bdc(&mut self, weather: u32, w: u32) {
        let r = self
            .stubs
            .call(0x021a1bdc, &mut [Arg::V(weather), Arg::V(w)]);
        self.rets.push((0x021a1bdc, r));
    }
    fn fun_021a1be4(&mut self, weather: u32, w: u32) {
        let r = self
            .stubs
            .call(0x021a1be4, &mut [Arg::V(weather), Arg::V(w)]);
        self.rets.push((0x021a1be4, r));
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
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        let r = self.stubs.call(0x021a3074, &mut [Arg::V(player)]);
        self.rets.push((0x021a3074, r));
        r
    }
    fn fun_021aa304(&mut self, rec: u32, gd: u32, x: u32) {
        let r = self
            .stubs
            .call(0x021aa304, &mut [Arg::V(rec), Arg::V(gd), Arg::V(x)]);
        self.rets.push((0x021aa304, r));
    }
    fn fun_021b6100(
        &mut self,
        gd: u32,
        mon: u32,
        rec: u32,
        status: u32,
        zukan: u32,
        a: u32,
        b: u32,
        species: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x021b6100,
            &mut [
                Arg::V(gd),
                Arg::V(mon),
                Arg::V(rec),
                Arg::V(status),
                Arg::V(zukan),
                Arg::V(a),
                Arg::V(b),
                Arg::V(species),
            ],
        );
        self.rets.push((0x021b6100, r));
        r
    }
    fn fun_021b6158(&mut self, work: u32) -> u32 {
        let r = self.stubs.call(0x021b6158, &mut [Arg::V(work)]);
        self.rets.push((0x021b6158, r));
        r
    }
    fn fun_021b95a0(&mut self, gd: u32, result: u32, mon: u32, species: u32) -> u32 {
        let r = self.stubs.call(
            0x021b95a0,
            &mut [Arg::V(gd), Arg::V(result), Arg::V(mon), Arg::V(species)],
        );
        self.rets.push((0x021b95a0, r));
        r
    }
    fn fun_021b95c8(&mut self, work: u32) {
        let r = self.stubs.call(0x021b95c8, &mut [Arg::V(work)]);
        self.rets.push((0x021b95c8, r));
    }
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32 {
        let r = self.stubs.call(
            0x021be8d4,
            &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(a), Arg::V(b)],
        );
        self.rets.push((0x021be8d4, r));
        r
    }
    fn fun_021be92c(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32 {
        let r = self.stubs.call(
            0x021be92c,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
            ],
        );
        self.rets.push((0x021be92c, r));
        r
    }
    fn fun_021c87a0(&mut self, w: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x021c87a0, &mut [Arg::V(w), Arg::V(i)]);
        self.rets.push((0x021c87a0, r));
        w
    }
    fn fun_021c87ac(&mut self, w: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x021c87ac, &mut [Arg::V(w), Arg::V(i)]);
        self.rets.push((0x021c87ac, r));
        w
    }
    fn fun_021c87b8(&mut self, w: u32, i: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021c87b8, &mut [Arg::V(w), Arg::V(i), Arg::V(x)]);
        self.rets.push((0x021c87b8, r));
        w
    }
    fn fun_021c87c8(&mut self, w: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x021c87c8, &mut [Arg::V(w), Arg::V(i)]);
        self.rets.push((0x021c87c8, r));
        r
    }
    fn fun_021cf498(&mut self, x: u32, event: u32, a: u32) {
        let r = self
            .stubs
            .call(0x021cf498, &mut [Arg::V(x), Arg::V(event), Arg::V(a)]);
        self.rets.push((0x021cf498, r));
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
        0x0216_6658 => boat_slot_word(h, v[0], v[1]) as u64,
        0x0216_666C => boat_event_create(h, v[0], v[1], v[2]) as u64,
        0x0216_66B8 => event_boat(h, v[0], v[1], v[2]) as u64,
        0x0216_670C => boat_time(h, v[0]) as u64,
        0x0216_6714 => {
            boat_time_set(h, v[0], v[1]);
            0
        }
        0x0216_6824 => rec20_byte(h, v[0], v[1]) as u64,
        0x0216_6834 => rec20_field(h, v[0], v[1]) as u64,
        0x0216_6854 => event_480_create(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0216_6908 => event_480(h, v[0], v[1], v[2]) as u64,
        0x0216_6E50 => event_493(h, v[0], v[1], v[2]) as u64,
        0x0216_AED0 => symbol_obj(h, v[0], v[1]) as u64,
        0x0216_AEE8 => {
            let mut w = h.r32(v[1]);
            symbol_word_of_talker(h, v[0], &mut w);
            h.w32(v[1], w);
            0
        }
        0x0216_AF1C => {
            let mut w = h.r32(v[3]);
            let r = symbol_record(h, v[0], v[1], v[2], &mut w);
            h.w32(v[3], w);
            r as u64
        }
        0x0216_AF3C => {
            symbol_notify(h, v[0]);
            0
        }
        0x0216_AF74 => symbol_battle_create(h, v[0], v[1], v[2], v[3], v[4], v[5]) as u64,
        0x0216_B004 => event_symbol_battle(h, v[0], v[1], v[2]) as u64,
        0x0216_B24C => intrude_wait_create(h, v[0], v[1], v[2]) as u64,
        0x0216_B280 => event_intrude_wait(h, v[0], v[1], v[2]) as u64,
        0x0216_B2E4 => {
            symbol_warp_pos(h, v[0], v[1], v[2]);
            0
        }
        0x0216_B340 => {
            let pos = [h.r32(v[2]), h.r32(v[2] + 4), h.r32(v[2] + 8)];
            forest_warp_create(h, v[0], v[1], &pos, v[3], v[4]) as u64
        }
        0x0216_B3A8 => forest_warp_start(h, v[0], v[1], v[2]) as u64,
        0x0216_B3EC => event_forest_warp(h, v[0], v[1], v[2]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10b`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10b",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10b.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// Overlay 10 of the ROM build (`$BLACK_ROM_BUILD/OVY_10.sbin`, loaded at 0x02155160), or `None` (skip) when it is
/// missing and `REQUIRE_ROM` is not set.
fn overlay10() -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_10.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_10.sbin is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), every handler is ported
/// and dispatched, and the event / proc / id literals are the build's.
#[test]
fn commands_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0);
    }
    for &(_, a) in COMMANDS {
        assert!(PORTED.contains(&a), "{a:#x}");
    }
    let Some(ovy) = overlay10() else {
        return;
    };
    let rd = |a: u32| {
        let o = (a - 0x0215_5160) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    for &(id, a) in COMMANDS {
        assert_eq!(rd(0x0217_062C + 4 * u32::from(id)), a | 1, "command {id}");
    }
    for (lit, val) in [
        (0x0216_66A0, EVENT_BOAT),
        (0x0216_6904, EVENT_480),
        (0x0216_6E4C, EVENT_493),
        (0x0216_AFF8, EVENT_SYMBOL_BATTLE),
        (0x0216_B27C, EVENT_INTRUDE_WAIT),
        (0x0216_B3A4, EVENT_FOREST_WARP),
        (0x0216_6704, PROC_BOAT.0),
        (0x0216_6708, PROC_BOAT.1),
        (0x0216_6978, PROC_480.0),
        (0x0216_697C, PROC_480.1),
        (0x0216_6F14, PROC_493.0),
        (0x0216_6F18, PROC_493.1),
        (0x0216_B240, PROC_SYMBOL_A),
        (0x0216_B248, PROC_SYMBOL_B),
        (0x0216_B230, SYMBOL_BATTLE_ID),
        (0x0216_B23C, OVERLAY_SYMBOL_A),
        (0x0216_B244, OVERLAY_SYMBOL_B),
        (0x0216_B500, SE_FOREST_WARP),
        (0x0216_AECC, ZONE_554),
        (0x0216_AF18, VAR_SYMBOL_OBJ),
        (0x0216_B000, SYMBOL_DEFAULT_LOW),
        (0x0216_6F98, WEATHER_ZONE),
    ] {
        assert_eq!(rd(lit), val, "{lit:#x}");
    }
}
