use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5A18, "84a5caa2b5cd2fde"),
    (0x021B5B0C, "c41ece2167f0ab83"),
    (0x021B5B4C, "1602793c83a84be9"),
    (0x021B5B8C, "cd96c708611d9a2b"),
    (0x021B5BFC, "4f31802305f27d9d"),
    (0x021B5C80, "2146d81d124714ec"),
    (0x021B5CE8, "9aa06767897a9bcb"),
    (0x021B5DD0, "ce8039f09390e2e4"),
    (0x021B5DF8, "2a512042b6c3b2d5"),
    (0x021B5EB8, "f7c86bcd2f518c06"),
    (0x021B5F24, "62c5d01797a17168"),
    (0x021B5FB4, "86b19dfc98aa8e45"),
    (0x021B5FB8, "7aa213794048d09d"),
    (0x021B6048, "1354ec539822ba5e"),
    (0x021B60F4, "131d783add5bec4a"),
    (0x021B6124, "c0f7e824bf166ff6"),
    (0x021B6164, "7e147b07741fe9b5"),
    (0x021B61E8, "f059ba29bebfdfac"),
    (0x021B627C, "314311aa93747bae"),
    (0x021B62C8, "409860c0d2b84bca"),
    (0x021B631C, "3531d5018b6bf824"),
    (0x021B636C, "7499d3f5ff0a8298"),
    (0x021B63D4, "7a54d06df388cd63"),
    (0x021B647C, "997bf4385e4330a0"),
    (0x021B654C, "42ec84d83f156aa4"),
    (0x021B6590, "5ce076a95115b5df"),
    (0x021B65D8, "1249220520d7dcae"),
    (0x021B6628, "bdc9f9216f0c1ec0"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer of the case in one memory window set (at the emulator's addresses).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl Rom<'_> {
    fn s(&mut self, addr: u32, args: &[u32]) -> u32 {
        let mut a: Vec<Arg> = args.iter().map(|&v| Arg::V(v)).collect();
        self.stubs.call(addr, &mut a)
    }
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl CoreHooks for Rom<'_> {
    fn fun_02011290(&mut self, vm: u32) {
        self.s(0x0201_1290, &[vm]);
    }
    fn fun_02011330(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1330, &[vm])
    }
    fn fun_0201134c(&mut self, vm: u32) -> u32 {
        self.s(0x0201_134C, &[vm])
    }
    fn fun_0201137c(&mut self, vm: u32, v: u32) {
        self.s(0x0201_137C, &[vm, v]);
    }
    fn fun_02011398(&mut self, vm: u32) -> u32 {
        self.s(0x0201_1398, &[vm])
    }
    fn fun_020113b0(&mut self, vm: u32, pc: u32) {
        self.s(0x0201_13B0, &[vm, pc]);
    }
    fn fun_020113b4(&mut self, vm: u32, pc: u32) {
        self.s(0x0201_13B4, &[vm, pc]);
    }
    fn fun_020113c4(&mut self, vm: u32) {
        self.s(0x0201_13C4, &[vm]);
    }
    fn fun_020113d0(&mut self, vm: u32, f: u32) {
        self.s(0x0201_13D0, &[vm, f]);
    }
    fn fun_0215a5d4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5D4, &[ctx])
    }
    fn fun_0215a5d8(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5D8, &[ctx])
    }
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5E4, &[ctx])
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5F4, &[ctx])
    }
    fn fun_021587e0(&mut self, sw: u32, slot: u32) -> u32 {
        self.s(0x0215_87E0, &[sw, slot])
    }
    fn fun_0215897c(&mut self, sw: u32, child: u32) {
        self.s(0x0215_897C, &[sw, child]);
    }
    fn fun_021589a0(&mut self, sw: u32, zone: u32, id: u32) -> u32 {
        self.s(0x0215_89A0, &[sw, zone, id])
    }
    fn fun_02158a34(&mut self, gamesys: u32, sw: u32, id: u32, mode: u32) {
        self.s(0x0215_8A34, &[gamesys, sw, id, mode]);
    }
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9AE8, &[vm, ctx])
    }
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x0215_9B10, &[vm, ctx])
    }
    fn fun_02159b38(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_9B38, &[ctx])
    }
    fn fun_0215a684(&mut self, ctx: u32, n: u32) {
        self.s(0x0215_A684, &[ctx, n]);
    }
    fn fun_0215a690(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A690, &[ctx])
    }
    fn fun_0215aa24(&mut self, ctx: u32) {
        self.s(0x0215_AA24, &[ctx]);
    }
    fn fun_0215aa34(&mut self, ctx: u32) {
        self.s(0x0215_AA34, &[ctx]);
    }
    fn fun_0215aa48(&mut self, ctx: u32, slot: u32) {
        self.s(0x0215_AA48, &[ctx, slot]);
    }
    fn fun_0215aa4c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_AA4C, &[ctx])
    }
    fn fun_020120f4(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_20F4, &[gamesys])
    }
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32 {
        self.s(0x0201_22C0, &[gamesys, parent, f, size])
    }
    fn fun_020124e8(&mut self, event: u32) -> u32 {
        self.s(0x0201_24E8, &[event])
    }
    fn fun_0201296c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_296C, &[gamedata])
    }
    fn fun_02012a30(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2A30, &[gamedata])
    }
    fn fun_02012b38(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2B38, &[gamedata])
    }
    fn fun_020142e8(&mut self, ew: u32, flag: u32) -> u32 {
        self.s(0x0201_42E8, &[ew, flag])
    }
    fn fun_02014314(&mut self, ew: u32, flag: u32) {
        self.s(0x0201_4314, &[ew, flag]);
    }
    fn fun_0201433c(&mut self, ew: u32, flag: u32) {
        self.s(0x0201_433C, &[ew, flag]);
    }
    fn fun_02014460(&mut self, x: u32) -> u32 {
        self.s(0x0201_4460, &[x])
    }
    fn fun_02014474(&mut self, x: u32, v: u32) {
        self.s(0x0201_4474, &[x, v]);
    }
    fn fun_020288c4(&mut self, x: u32) -> u32 {
        self.s(0x0202_88C4, &[x])
    }
    fn fun_0202972c(&mut self, gamesys: u32, a: u32) -> u32 {
        self.s(0x0202_972C, &[gamesys, a])
    }
    fn fun_021b1bb8(&mut self, vm: u32, ctx: u32) -> u32 {
        self.s(0x021B_1BB8, &[vm, ctx])
    }
    fn fun_021b1be8(&mut self, ctx: u32) {
        self.s(0x021B_1BE8, &[ctx]);
    }
}

#[allow(clippy::too_many_arguments)]
impl C03Hooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f74, &mut [Arg::V(sw)])
    }
    fn fun_02158f7c(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f7c, &mut [Arg::V(sw)])
    }
    fn fun_0215b4ac(&mut self, ctx: u32, slot: u32, mon: &mut [u8; 4]) -> u32 {
        self.stubs.call(
            0x0215b4ac,
            &mut [Arg::V(ctx), Arg::V(slot), Arg::Out(&mut mon[..])],
        )
    }
    fn fun_0215b500(&mut self, mon: u32, mv: u32) -> u32 {
        self.stubs.call(0x0215b500, &mut [Arg::V(mon), Arg::V(mv)])
    }
    fn fun_0215a07c(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        self.stubs
            .call(0x0215a07c, &mut [Arg::V(gamesys), Arg::V(a), Arg::V(b)])
    }
    fn fun_0215a0e8(&mut self, gamesys: u32, a: u32, b: u32, mask: u32, mv: u32) -> u32 {
        self.stubs.call(
            0x0215a0e8,
            &mut [
                Arg::V(gamesys),
                Arg::V(a),
                Arg::V(b),
                Arg::V(mask),
                Arg::V(mv),
            ],
        )
    }
    fn fun_0215a2e4(
        &mut self,
        gamesys: u32,
        fm: u32,
        c: u32,
        a: u32,
        b: u32,
        z: u32,
        d: u32,
    ) -> u32 {
        self.stubs.call(
            0x0215a2e4,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(c),
                Arg::V(a),
                Arg::V(b),
                Arg::V(z),
                Arg::V(d),
            ],
        )
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c58, &mut [Arg::V(fm)])
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c7c, &mut [Arg::V(fm)])
    }
    fn fun_02188c84(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c84, &mut [Arg::V(fm)])
    }
    fn fun_02188d30(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188d30, &mut [Arg::V(fm)])
    }
    fn fun_0218ed50(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218ed50, &mut [Arg::V(e)])
    }
    fn fun_0218fe70(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218fe70, &mut [Arg::V(e)])
    }
    fn fun_0216cdb8(&mut self, x: u32, angle: u32) {
        self.stubs.call(0x0216cdb8, &mut [Arg::V(x), Arg::V(angle)]);
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32 {
        self.stubs.call(
            0x02030734,
            &mut [
                Arg::V(heap),
                Arg::V(size),
                Arg::V(from_end),
                Arg::V(file),
                Arg::V(line),
            ],
        )
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.stubs.call(0x020307b0, &mut [Arg::V(block)]);
    }
    fn fun_02030da8(&mut self, tcbsys: u32, f: u32, work: u32, pri: u32) {
        self.stubs.call(
            0x02030da8,
            &mut [Arg::V(tcbsys), Arg::V(f), Arg::V(work), Arg::V(pri)],
        );
    }
    fn fun_02030eac(&mut self, tcb: u32) -> u32 {
        self.stubs.call(0x02030eac, &mut [Arg::V(tcb)])
    }
    fn fun_02012f08(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f08, &mut [Arg::V(gamedata)])
    }
    fn fun_02012fd8(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012fd8, &mut [Arg::V(gamedata)])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012934, &mut [Arg::V(gamedata)])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012944, &mut [Arg::V(gamedata)])
    }
    fn fun_0201292c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201292c, &mut [Arg::V(gamedata)])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.stubs.call(0x0201aa18, &mut [Arg::V(party)])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.stubs.call(0x0201ab28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)])
    }
    fn fun_0201891c(&mut self, mon: u32, k: u32) -> u32 {
        self.stubs.call(0x0201891c, &mut [Arg::V(mon), Arg::V(k)])
    }
    fn fun_020188ac(&mut self, mon: u32, data: u32) -> u32 {
        self.stubs
            .call(0x020188ac, &mut [Arg::V(mon), Arg::V(data)])
    }
    fn fun_02018790(&mut self, mon: u32) -> u32 {
        self.stubs.call(0x02018790, &mut [Arg::V(mon)])
    }
    fn fun_0202130c(&mut self, item: u32) -> u32 {
        self.stubs.call(0x0202130c, &mut [Arg::V(item)])
    }
    fn fun_0200c11c(&mut self, z: u32, species: u32, form: u32, egg: u32, gender: u32) {
        self.stubs.call(
            0x0200c11c,
            &mut [
                Arg::V(z),
                Arg::V(species),
                Arg::V(form),
                Arg::V(egg),
                Arg::V(gender),
            ],
        );
    }
    fn fun_0200c2b0(&mut self, z: u32, v: u32, arr: &[u8; 12]) {
        self.stubs
            .call(0x0200c2b0, &mut [Arg::V(z), Arg::V(v), Arg::In(arr)]);
    }
    fn fun_0200c2d0(&mut self, z: u32, v: u32, arr: &mut [u8; 12]) {
        self.stubs.call(
            0x0200c2d0,
            &mut [Arg::V(z), Arg::V(v), Arg::Out(&mut arr[..])],
        );
    }
    fn fun_0201ee60(&mut self, list: u32, i: u32, v: u32) {
        self.stubs
            .call(0x0201ee60, &mut [Arg::V(list), Arg::V(i), Arg::V(v)]);
    }
    fn fun_0201ee0c(&mut self, ws: u32, i: u32, buf: u32, a: u32, b: u32, c: u32) {
        self.stubs.call(
            0x0201ee0c,
            &mut [
                Arg::V(ws),
                Arg::V(i),
                Arg::V(buf),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
            ],
        );
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0200ca1c, &mut [Arg::V(gamedata)])
    }
    fn fun_0200ca38(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200ca38, &mut [Arg::V(x)])
    }
    fn fun_0200cf08(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x0200cf08, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_0200cf54(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x0200cf54, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_0200d0f8(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x0200d0f8, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_0200d298(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200d298, &mut [Arg::V(x)])
    }
    fn fun_0200d2b0(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x0200d2b0, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_0200d2c4(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x0200d2c4, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_0200e740(&mut self, z: u32, i: u32) -> u32 {
        self.stubs.call(0x0200e740, &mut [Arg::V(z), Arg::V(i)])
    }
    fn fun_0200e7ac(&mut self, z: u32, i: u32) -> u32 {
        self.stubs.call(0x0200e7ac, &mut [Arg::V(z), Arg::V(i)])
    }
    fn fun_0200e7bc(&mut self, z: u32, i: u32, mon: u32) {
        self.stubs
            .call(0x0200e7bc, &mut [Arg::V(z), Arg::V(i), Arg::V(mon)]);
    }
    fn fun_02045924(&mut self, buf: u32, src: u32) {
        let b1: Vec<u8> = (0..0x2).map(|i| self.mem.r8(src + i)).collect();
        self.stubs
            .call(0x02045924, &mut [Arg::V(buf), Arg::In(&b1)]);
    }
    fn fun_02045a98(&mut self, r0: u32) -> u32 {
        self.stubs.call(0x02045a98, &mut [Arg::V(r0)])
    }
    fn fun_02007fa0(&mut self, bag: u32, pocket: u32) -> u32 {
        self.stubs
            .call(0x02007fa0, &mut [Arg::V(bag), Arg::V(pocket)])
    }
    fn fun_020081dc(&mut self, bag: u32, pocket: u32, i: u32) -> u32 {
        self.stubs
            .call(0x020081dc, &mut [Arg::V(bag), Arg::V(pocket), Arg::V(i)])
    }
    fn fun_020212b0(&mut self, item: u32) -> u32 {
        self.stubs.call(0x020212b0, &mut [Arg::V(item)])
    }
    fn fun_020056ec(&mut self, r0: u32) -> u32 {
        self.stubs.call(0x020056ec, &mut [Arg::V(r0)])
    }
    fn fun_021c5aac(&mut self, gamesys: u32, var: u32, a: u32, b: u32) -> u32 {
        self.stubs.call(
            0x021c5aac,
            &mut [Arg::V(gamesys), Arg::V(var), Arg::V(a), Arg::V(b)],
        )
    }
    fn fun_02166c4c(&mut self, gamesys: u32, fm: u32, mon: u32) -> u32 {
        self.stubs
            .call(0x02166c4c, &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(mon)])
    }
    fn fun_021cbfb0(&mut self, heap: u32, i: u32) -> u32 {
        self.stubs.call(0x021cbfb0, &mut [Arg::V(heap), Arg::V(i)])
    }
    fn fun_021cc040(&mut self, t: u32) {
        self.stubs.call(0x021cc040, &mut [Arg::V(t)]);
    }
    fn fun_021cc060(&mut self, t: u32) -> u32 {
        self.stubs.call(0x021cc060, &mut [Arg::V(t)])
    }
    fn fun_021cc068(&mut self, t: u32) -> u32 {
        self.stubs.call(0x021cc068, &mut [Arg::V(t)])
    }
    fn fun_021cc500(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        self.stubs
            .call(0x021cc500, &mut [Arg::V(gamesys), Arg::V(a), Arg::V(b)])
    }
    fn fun_02045924_str(&mut self, buf: u32, s: &[u8; 2]) {
        self.stubs.call(0x0204_5924, &mut [Arg::V(buf), Arg::In(s)]);
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address and the function's globals at `globals`; the
/// buffers are put back afterwards.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(Some(i));
        }
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push(None);
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    let mut k = 0;
    for (slot, (_, bytes)) in keep.into_iter().zip(wins) {
        match slot {
            Some(i) => c.bufs[i] = bytes,
            None => {
                c.globals[k] = bytes;
                k += 1;
            }
        }
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    let globals: &[u32] = match addr {
        0x021B_5A18 | 0x021B_5B0C => &[TURN_ANGLE],
        _ => &[],
    };
    with_rom(c, globals, |h, v| match addr {
        0x021B_5B0C => {
            turn_task(h, v[0], v[1]);
            0
        }
        0x021B_5DD0 => table_lookup(h, v[0], v[1]) as u64,
        0x021B_5F24 => tutor_move(h, v[0], v[1]) as u64,
        0x021B_5FB4 => tutor_max_level(v[0]) as u64,
        0x021B_5FB8 => tutor_can_learn(h, v[0], v[1]) as u64,
        0x021B_6048 => tutor_party_check(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The TCB function, file string, the BSS word and the lookup tables are the ROM's.
#[test]
fn literals_match_rom() {
    let Some(ovy) = overlay21() else {
        return;
    };
    let rd = |a: u32, n: usize| {
        let o = (a - 0x0218_7F00) as usize;
        let mut w = [0u8; 4];
        w[..n].copy_from_slice(&ovy[o..o + n]);
        u32::from_le_bytes(w)
    };
    assert_eq!(rd(0x021B_5AFC, 4), FILE);
    assert_eq!(rd(0x021B_5B00, 4), TURN_ANGLE);
    assert_eq!(rd(0x021B_5B08, 4), TURN_TASK);
    assert_eq!(rd(0x021B_5EAC, 4), TABLE_463_3_ADDR);
    assert_eq!(rd(0x021B_5EB0, 4), TABLE_463_2_ADDR);
    assert_eq!(rd(0x021B_5EB4, 4), TABLE_463_1_ADDR);
    assert_eq!(rd(0x021B_6314, 4), ITEM_566);
    assert_eq!(rd(0x021B_6270, 4), MOVES_487[1]);
    assert_eq!(rd(0x021B_6274, 4), MOVES_487[2]);
    assert_eq!(rd(0x021B_6278, 4), MOVES_487[0]);
    for (base, t) in [
        (TABLE_463_1_ADDR, &TABLE_463_1[..]),
        (TABLE_463_2_ADDR, &TABLE_463_2[..]),
        (TABLE_463_3_ADDR, &TABLE_463_3[..]),
    ] {
        for (k, &(key, val)) in t.iter().enumerate() {
            assert_eq!(rd(base + 4 * k as u32, 2), u32::from(key));
            assert_eq!(rd(base + 4 * k as u32 + 2, 2), u32::from(val));
        }
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_c03center`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_c03center",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_c03center.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// Overlay 10 of the ROM build (`$BLACK_ROM_BUILD/OVY_10.sbin`, loaded at 0x02155160), or `None` (skip) when it is
/// missing and `REQUIRE_ROM` is not set.
fn overlay10() -> Option<Vec<u8>> {
    rom_file("OVY_10.sbin")
}

/// Overlay 21 of the ROM build (`$BLACK_ROM_BUILD/OVY_21.sbin`, loaded at 0x02187F00), or `None` (skip).
#[allow(dead_code)]
fn overlay21() -> Option<Vec<u8>> {
    rom_file("OVY_21.sbin")
}

fn rom_file(name: &str) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/{name}")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/{name} is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), the ids are unique and
/// every handler is ported and dispatched.
#[test]
fn commands_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0, "ids not sorted / unique at {}", w[1].0);
    }
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    let Some(ovy) = overlay10() else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}
