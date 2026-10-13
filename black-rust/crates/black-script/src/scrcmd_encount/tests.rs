use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5190, "973bdfb11a1397e6"),
    (0x021B51BC, "1cfa351efc04e1c7"),
    (0x021B52A4, "03e417120450f948"),
    (0x021B52E0, "fb4d354d773fe148"),
    (0x021B52FC, "e16e3a22cb8dfbdc"),
    (0x021B536C, "d3f1a3125562931b"),
    (0x021B538C, "b62de9bb6a813ae6"),
    (0x021B53A4, "9fca424462a9b6cb"),
    (0x021B53BC, "a3afab73b572491c"),
    (0x021B542C, "bdfd3a0cbaf8584a"),
    (0x021B5498, "f57a9509a5151216"),
    (0x021B54D4, "fa26913b9271077c"),
    (0x021B54FC, "dc09c49f289ac637"),
    (0x021B5528, "622d00907c5d8220"),
    (0x021B55C8, "f00abce46bbf884d"),
    (0x021B55F0, "f009640c96059e7c"),
    (0x021B5610, "91230241681bcf52"),
    (0x021B5650, "62bac2809d072a68"),
    (0x021B568C, "d80e1be15de293b3"),
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
impl EncountHooks for Rom<'_> {
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_020124e4(&mut self, event: u32) -> u32 {
        self.stubs.call(0x020124e4, &mut [Arg::V(event)])
    }
    fn fun_02012984(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012984, &mut [Arg::V(gamedata)])
    }
    fn fun_020490f4(&mut self, arc: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x020490f4, &mut [Arg::V(arc), Arg::V(heap)])
    }
    fn fun_02049238(&mut self, handle: u32) -> u32 {
        self.stubs.call(0x02049238, &mut [Arg::V(handle)])
    }
    fn fun_02048f4c(&mut self, t: &mut [u8; 0x2C], arc: u32, member: u32) {
        self.stubs.call(
            0x02048f4c,
            &mut [Arg::Out(&mut t[..]), Arg::V(arc), Arg::V(member)],
        );
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
    fn fun_02005728(&mut self, n: u32) -> u32 {
        self.stubs.call(0x02005728, &mut [Arg::V(n)])
    }
    fn fun_0215e2b8(&mut self, rec: &mut [u8; 0xE8], arc: u32, id: u32, zone: u32) -> u32 {
        self.stubs.call(
            0x0215e2b8,
            &mut [
                Arg::Out(&mut rec[..]),
                Arg::V(arc),
                Arg::V(id),
                Arg::V(zone),
            ],
        )
    }
    fn fun_02159144(&mut self, x: u32) {
        self.stubs.call(0x02159144, &mut [Arg::V(x)]);
    }
    fn fun_02159168(&mut self, x: u32) {
        self.stubs.call(0x02159168, &mut [Arg::V(x)]);
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c58, &mut [Arg::V(fm)])
    }
    fn fun_02188c5c(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c5c, &mut [Arg::V(fm)])
    }
    fn fun_02188cfc(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188cfc, &mut [Arg::V(fm)])
    }
    fn fun_02188d24(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188d24, &mut [Arg::V(fm)])
    }
    fn fun_021bb260(&mut self, x: u32, on: u32) {
        self.stubs.call(0x021bb260, &mut [Arg::V(x), Arg::V(on)]);
    }
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32 {
        self.stubs
            .call(0x021bb990, &mut [Arg::V(proc_id), Arg::V(x), Arg::V(table)])
    }
    fn fun_0218e17c(&mut self, e: u32, x: u32) {
        self.stubs.call(0x0218e17c, &mut [Arg::V(e), Arg::V(x)]);
    }
    fn fun_0218ec54(&mut self, e: u32) {
        self.stubs.call(0x0218ec54, &mut [Arg::V(e)]);
    }
    fn fun_0218ec60(&mut self, e: u32) {
        self.stubs.call(0x0218ec60, &mut [Arg::V(e)]);
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
    fn fun_0218f0f8(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218f0f8, &mut [Arg::V(e)])
    }
    fn fun_0218f818(&mut self, e: u32) {
        self.stubs.call(0x0218f818, &mut [Arg::V(e)]);
    }
    fn fun_0218f878(&mut self, e: u32) {
        self.stubs.call(0x0218f878, &mut [Arg::V(e)]);
    }
    fn fun_0218f900(&mut self, e: u32, rec: &[u8; 0x48], n: u32) {
        self.stubs
            .call(0x0218f900, &mut [Arg::V(e), Arg::In(rec), Arg::V(n)]);
    }
    fn fun_0218f9c4(&mut self, e: u32, rec: &[u8; 0x18], n: u32) {
        self.stubs
            .call(0x0218f9c4, &mut [Arg::V(e), Arg::In(rec), Arg::V(n)]);
    }
    fn fun_0218fa40(&mut self, e: u32, n: u32) {
        self.stubs.call(0x0218fa40, &mut [Arg::V(e), Arg::V(n)]);
    }
    fn fun_0218fe50(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218fe50, &mut [Arg::V(e)])
    }
    fn fun_021d2d50(&mut self, gamesys: u32, rec: &[u8; 0x18]) -> u32 {
        self.stubs
            .call(0x021d2d50, &mut [Arg::V(gamesys), Arg::In(rec)])
    }
    fn fun_021f36c0(&mut self, gamesys: u32, fm: u32, n: u32) -> u32 {
        // `ret=none` stub: r0 keeps the first argument
        self.stubs
            .call(0x021f36c0, &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(n)]);
        gamesys
    }
    fn fun_021f36c8(&mut self, n: u32) -> u32 {
        self.stubs.call(0x021f36c8, &mut [Arg::V(n)])
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
        0x021B_5190 => add_unique(h, v[0], v[1], v[2]) as u64,
        0x021B_52A4 => {
            effect_off(h, v[0], v[1]);
            0
        }
        0x021B_52E0 => init_proc(h, v[0], v[1]) as u64,
        0x021B_55C8 => event_319(h, v[0]) as u64,
        0x021B_55F0 => wait_325(h, v[0]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The init proc, event functions, file string and tables are the ROM's.
#[test]
fn literals_match_rom() {
    assert_eq!(crate::script_sys::INIT_PROCS[0], INIT_PROC);
    let Some(ovy) = overlay21() else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0218_7F00) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    assert_eq!(word(0x021B_5368), EVENT_319);
    assert_eq!(word(0x021B_5524), EVENT_325);
    assert_eq!(word(0x021B_529C), FILE);
    assert_eq!(word(0x021B_52A0), TABLE_561_ADDR);
    assert_eq!(word(0x021B_56EC), PROC_420);
    for (k, t) in TABLES_420.iter().enumerate() {
        assert_eq!(word(0x021B_56F0 + 4 * k as u32), *t);
    }
    for (k, id) in TABLE_561.iter().enumerate() {
        assert_eq!(word(TABLE_561_ADDR + 2 * k as u32) & 0xFFFF, u32::from(*id));
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_encount",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_encount.spec"),
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
