use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B2FD8, "9680135b94aeb33d"),
    (0x021B30C8, "de51076cc47020d3"),
    (0x021B3158, "baeeaa22250b7fbf"),
    (0x021B3258, "6892310ae428883f"),
    (0x021B3350, "4f196f685167693d"),
    (0x021B3370, "f6e89ac08c292386"),
    (0x021B3568, "6fc15635dfb5f1db"),
    (0x021B3834, "3fe2a81e89252e37"),
    (0x021B38E0, "ebfe3f402fbd30bd"),
    (0x021B39C4, "b48a9b68888864d1"),
    (0x021B3A64, "a8f8a3af2ecc35e8"),
    (0x021B4710, "c86bdd2468b601a5"),
    (0x021B49EC, "341e901db65f8b22"),
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
impl ShopHooks for Rom<'_> {
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_02158fb4(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158fb4, &mut [Arg::V(sw)])
    }
    fn fun_02188c54(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c54, &mut [Arg::V(fm)])
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c58, &mut [Arg::V(fm)])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188cb4, &mut [Arg::V(fm)])
    }
    fn fun_021904ac(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021904ac, &mut [Arg::V(x)])
    }
    fn fun_020127c4(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x020127c4, &mut [Arg::V(gamedata)])
    }
    fn fun_02012f08(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f08, &mut [Arg::V(gamedata)])
    }
    fn fun_0201292c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201292c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)])
    }
    fn fun_020071cc(&mut self, save: u32, id: u32) -> u32 {
        self.stubs.call(0x020071cc, &mut [Arg::V(save), Arg::V(id)])
    }
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f2c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012944, &mut [Arg::V(gamedata)])
    }
    fn fun_021655d4(&mut self) {
        self.stubs.call(0x021655d4, &mut []);
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
    fn fun_021a3050(&mut self, player: u32) -> u32 {
        self.stubs.call(0x021a3050, &mut [Arg::V(player)])
    }
    fn fun_0218ec54(&mut self, e: u32) {
        self.stubs.call(0x0218ec54, &mut [Arg::V(e)]);
    }
    fn fun_0218ec60(&mut self, e: u32) {
        self.stubs.call(0x0218ec60, &mut [Arg::V(e)]);
    }
    fn fun_0218ecb4(&mut self, e: u32, t: &mut [u8; 12]) {
        self.stubs
            .call(0x0218ecb4, &mut [Arg::V(e), Arg::Out(&mut t[..])]);
    }
    fn fun_0218ece4(&mut self, e: u32, a: &mut [u8; 12]) {
        self.stubs
            .call(0x0218ece4, &mut [Arg::V(e), Arg::Out(&mut a[..])]);
    }
    fn fun_0218f05c(&mut self, e: u32) {
        self.stubs.call(0x0218f05c, &mut [Arg::V(e)]);
    }
    fn fun_0218f0a8(&mut self, e: u32) {
        self.stubs.call(0x0218f0a8, &mut [Arg::V(e)]);
    }
    fn fun_0218f0e8(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218f0e8, &mut [Arg::V(e)])
    }
    fn fun_0218f1ac(&mut self, e: u32, v: u32) {
        self.stubs.call(0x0218f1ac, &mut [Arg::V(e), Arg::V(v)]);
    }
    fn fun_0218f1b4(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218f1b4, &mut [Arg::V(e)])
    }
    fn fun_0218f848(&mut self, e: u32) {
        self.stubs.call(0x0218f848, &mut [Arg::V(e)]);
    }
    fn fun_0218f878(&mut self, e: u32) {
        self.stubs.call(0x0218f878, &mut [Arg::V(e)]);
    }
    fn fun_0218f918(&mut self, e: u32, rec: &[u8; 0x48], frames: u32) {
        self.stubs
            .call(0x0218f918, &mut [Arg::V(e), Arg::In(rec), Arg::V(frames)]);
    }
    fn fun_0218f9c4(&mut self, e: u32, rec: &[u8; 0x18], frames: u32) {
        self.stubs
            .call(0x0218f9c4, &mut [Arg::V(e), Arg::In(rec), Arg::V(frames)]);
    }
    fn fun_0218fe50(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218fe50, &mut [Arg::V(e)])
    }
    fn fun_02193060(&mut self, on: u32) -> u32 {
        self.stubs.call(0x02193060, &mut [Arg::V(on)])
    }
    fn fun_021b35a4(&mut self, gamesys: u32, w2: u32, kind: u32, b: u32) -> u32 {
        self.stubs.call(
            0x021b35a4,
            &mut [Arg::V(gamesys), Arg::V(w2), Arg::V(kind), Arg::V(b)],
        )
    }
    fn fun_021b3b14(&mut self, w2: u32, kind: u32) -> u32 {
        self.stubs.call(0x021b3b14, &mut [Arg::V(w2), Arg::V(kind)])
    }
    fn fun_021b331c(&mut self, w2: u32, bp: u32) {
        self.stubs.call(0x021b331c, &mut [Arg::V(w2), Arg::V(bp)]);
    }
    fn fun_021b3bd8(&mut self, w2: u32) {
        self.stubs.call(0x021b3bd8, &mut [Arg::V(w2)]);
    }
    fn fun_021b3fa0(&mut self, w2: u32) {
        self.stubs.call(0x021b3fa0, &mut [Arg::V(w2)]);
    }
    fn fun_021b3f00(&mut self, w2: u32) {
        self.stubs.call(0x021b3f00, &mut [Arg::V(w2)]);
    }
    fn fun_021b40d4(&mut self, w2: u32) {
        self.stubs.call(0x021b40d4, &mut [Arg::V(w2)]);
    }
    fn fun_021b43ec(&mut self, w2: u32) -> u32 {
        self.stubs.call(0x021b43ec, &mut [Arg::V(w2)])
    }
    fn fun_021b385c(&mut self, w2: u32, row: u32, flag: u32, item: u32, msg: u32) {
        self.stubs.call(
            0x021b385c,
            &mut [
                Arg::V(w2),
                Arg::V(row),
                Arg::V(flag),
                Arg::V(item),
                Arg::V(msg),
            ],
        );
    }
    fn fun_021b4714(&mut self, w2: u32, msg: u32, item: u32, x: u32) {
        self.stubs.call(
            0x021b4714,
            &mut [Arg::V(w2), Arg::V(msg), Arg::V(item), Arg::V(x)],
        );
    }
    fn fun_0200c258(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200c258, &mut [Arg::V(x)])
    }
    fn fun_0200c274(&mut self, money: u32) -> u32 {
        self.stubs.call(0x0200c274, &mut [Arg::V(money)])
    }
    fn fun_0200dabc(&mut self, bp: u32) -> u32 {
        self.stubs.call(0x0200dabc, &mut [Arg::V(bp)])
    }
    fn fun_0200c29c(&mut self, money: u32, n: u32) {
        self.stubs.call(0x0200c29c, &mut [Arg::V(money), Arg::V(n)]);
    }
    fn fun_0200da98(&mut self, bp: u32, n: u32) {
        self.stubs.call(0x0200da98, &mut [Arg::V(bp), Arg::V(n)]);
    }
    fn fun_02008dac(&mut self, records: u32, id: u32, n: u32) {
        self.stubs
            .call(0x02008dac, &mut [Arg::V(records), Arg::V(id), Arg::V(n)]);
    }
    fn fun_02008df0(&mut self, records: u32, id: u32) -> u32 {
        self.stubs
            .call(0x02008df0, &mut [Arg::V(records), Arg::V(id)])
    }
    fn fun_02007fb8(&mut self, bag: u32, item: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x02007fb8, &mut [Arg::V(bag), Arg::V(item), Arg::V(heap)])
    }
    fn fun_02007e3c(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x02007e3c,
            &mut [Arg::V(bag), Arg::V(item), Arg::V(n), Arg::V(heap)],
        )
    }
    fn fun_0200823c(&mut self, item: u32) -> u32 {
        self.stubs.call(0x0200823c, &mut [Arg::V(item)])
    }
    fn fun_02021288(&mut self, item: u32) -> u32 {
        self.stubs.call(0x02021288, &mut [Arg::V(item)])
    }
    fn fun_02020fe4(&mut self, item: u32, x: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x02020fe4, &mut [Arg::V(item), Arg::V(x), Arg::V(heap)])
    }
    fn fun_0201102c(&mut self, data: u32) -> u32 {
        self.stubs.call(0x0201102c, &mut [Arg::V(data)])
    }
    fn fun_02045b38(&mut self, a: u32, b: u32, arc: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x02045b38,
            &mut [Arg::V(a), Arg::V(b), Arg::V(arc), Arg::V(heap)],
        )
    }
    fn fun_02045c04(&mut self, msg: u32) {
        self.stubs.call(0x02045c04, &mut [Arg::V(msg)]);
    }
    fn fun_0201f81c(&mut self, n: u32, heap: u32) -> u32 {
        self.stubs.call(0x0201f81c, &mut [Arg::V(n), Arg::V(heap)])
    }
    fn fun_0201f878(&mut self, row: u32, a: u32, b: u32, c: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x0201f878,
            &mut [Arg::V(row), Arg::V(a), Arg::V(b), Arg::V(c), Arg::V(heap)],
        )
    }
    fn fun_02186670(&mut self, x: u32, _list: u32, count: u32, n: u32, heap: u32) {
        let mut b2: Vec<u8> = (0..0x2).map(|i| self.mem.r8(count + i)).collect();
        self.stubs.call(
            0x02186670,
            &mut [
                Arg::V(x),
                Arg::P,
                Arg::Out(&mut b2),
                Arg::V(n),
                Arg::V(heap),
            ],
        );
        for (i, x) in b2.iter().enumerate() {
            self.mem.w8(count + i as u32, *x);
        }
    }
    fn fun_021b4804(&mut self, w2: u32, flag: u32) {
        self.s(0x021B_4804, &[w2, flag]);
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
    with_rom(c, &[], |h, v| match addr {
        0x021B_30C8 => {
            shop_camera(h, v[0], v[1], v[2]);
            0
        }
        0x021B_3158 => wait_329(h, v[0], v[1]) as u64,
        0x021B_3258 => {
            shop_init(h, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_3350 => currency(h, v[0]) as u64,
        0x021B_3370 => {
            buy_check(h, v[0]);
            0
        }
        0x021B_3568 => {
            pay(h, v[0], v[1]);
            0
        }
        0x021B_3834 => list_flag(h, v[0], v[1]) as u64,
        0x021B_38E0 => {
            mart_list(h, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_39C4 => {
            special_list(h, v[0], v[1]);
            0
        }
        0x021B_3A64 => {
            dyn_list(h, v[0], v[1]);
            0
        }
        0x021B_4710 => list_cb_nop() as u64,
        0x021B_49EC => count_step(h, v[0], v[1], v[2]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The wait callback, file string and the embedded ROM tables are the build's.
#[test]
fn tables_match_rom() {
    let Some(ovy) = overlay21() else {
        return;
    };
    let rd = |a: u32, n: usize| {
        let o = (a - 0x0218_7F00) as usize;
        let mut w = [0u8; 4];
        w[..n].copy_from_slice(&ovy[o..o + n]);
        u32::from_le_bytes(w)
    };
    assert_eq!(rd(0x021B_30C4, 4), WAIT_329);
    assert_eq!(rd(0x021B_30C0, 4), FILE);
    assert_eq!(rd(0x021B_39B4, 4), MART_BY_BADGES_ADDR);
    assert_eq!(rd(0x021B_39B8, 4), MART_LISTS_ADDR);
    assert_eq!(rd(0x021B_39C0, 4), MART_COUNTS_ADDR);
    assert_eq!(rd(0x021B_3A5C, 4), SPECIAL_LISTS_ADDR);
    assert_eq!(rd(0x021B_3A60, 4), SPECIAL_COUNTS_ADDR);
    assert_eq!(rd(0x021B_314C, 4), CAMERA_OFFSETS_ADDR);
    assert_eq!(rd(0x021B_3254, 4), CAMERA_BACK_ADDR);
    for (i, &b) in MART_BY_BADGES.iter().enumerate() {
        assert_eq!(rd(MART_BY_BADGES_ADDR + i as u32, 1), u32::from(b));
    }
    for (i, l) in MART_LISTS.iter().enumerate() {
        assert_eq!(
            rd(MART_COUNTS_ADDR + i as u32, 1),
            l.len() as u32,
            "count {i}"
        );
        let p = rd(MART_LISTS_ADDR + 4 * i as u32, 4);
        for (j, &item) in l.iter().enumerate() {
            assert_eq!(
                rd(p + 2 * j as u32, 2),
                u32::from(item),
                "list {i} item {j}"
            );
        }
    }
    for (k, l) in SPECIAL_LISTS.iter().enumerate() {
        assert_eq!(rd(SPECIAL_COUNTS_ADDR + k as u32, 1), l.len() as u32);
        let p = rd(SPECIAL_LISTS_ADDR + 4 * k as u32, 4);
        for (j, &(item, price)) in l.iter().enumerate() {
            assert_eq!(rd(p + 8 * j as u32, 2), u32::from(item));
            assert_eq!(rd(p + 8 * j as u32 + 4, 4), price);
        }
    }
    for (d, o) in CAMERA_OFFSETS.iter().enumerate() {
        for (k, &v) in o.iter().enumerate() {
            assert_eq!(rd(CAMERA_OFFSETS_ADDR + 12 * d as u32 + 4 * k as u32, 4), v);
        }
    }
    for (k, &v) in CAMERA_BACK.iter().enumerate() {
        assert_eq!(rd(CAMERA_BACK_ADDR + 4 * k as u32, 4), v);
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_shop`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_shop",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_shop.spec"),
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
