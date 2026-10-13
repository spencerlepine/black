use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B5700, "1d5f7702ee143f59"),
    (0x021B574C, "dd3a221b5acb69e0"),
    (0x021B57CC, "3b678b22497e2fe1"),
    (0x021B5824, "e4b9cdee3902c9fd"),
    (0x021B58B0, "b15267b78449b13f"),
    (0x021B59B0, "7a0a3aea7e3a6e6a"),
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
impl FldBattleHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f74, &mut [Arg::V(sw)])
    }
    fn fun_0201ee60(&mut self, list: u32, i: u32, v: u32) {
        self.stubs
            .call(0x0201ee60, &mut [Arg::V(list), Arg::V(i), Arg::V(v)]);
    }
    fn fun_0201a354(&mut self, v: u32, heap: u32) -> u32 {
        self.stubs.call(0x0201a354, &mut [Arg::V(v), Arg::V(heap)])
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0200ca1c, &mut [Arg::V(gamedata)])
    }
    fn fun_0200ced4(&mut self, dex: u32, id: u32) -> u32 {
        self.stubs.call(0x0200ced4, &mut [Arg::V(dex), Arg::V(id)])
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
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012934, &mut [Arg::V(gamedata)])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)])
    }
    fn fun_0200bac8(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200bac8, &mut [Arg::V(save)])
    }
    fn fun_0200bba8(&mut self, block: u32) -> u32 {
        self.stubs.call(0x0200bba8, &mut [Arg::V(block)])
    }
    fn fun_0200baf4(&mut self, block: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x0200baf4, &mut [Arg::V(block), Arg::V(heap)])
    }
    fn fun_02019ed4(&mut self, reg: u32, party: u32) -> u32 {
        self.stubs
            .call(0x02019ed4, &mut [Arg::V(reg), Arg::V(party)])
    }
    fn fun_0201a090(&mut self, reg: u32, party: u32, _x: u32) -> u32 {
        self.stubs
            .call(0x0201a090, &mut [Arg::V(reg), Arg::V(party), Arg::P])
    }
    fn fun_021cc720(&mut self, fm: u32, party: u32, bx: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x021cc720,
            &mut [
                Arg::V(fm),
                Arg::V(party),
                Arg::V(bx),
                Arg::V(a),
                Arg::V(b),
                Arg::V(heap),
            ],
        )
    }
    fn fun_021cc810(&mut self, sel: u32) -> u32 {
        self.stubs.call(0x021cc810, &mut [Arg::V(sel)])
    }
    fn fun_021cc7b4(&mut self, sel: u32, cancel: &mut [u8; 4]) -> u32 {
        self.stubs
            .call(0x021cc7b4, &mut [Arg::V(sel), Arg::Out(&mut cancel[..])])
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
        0x021B_57CC => count_mons(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_5824 => collect(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_58B0 => regulation_event(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021B_59B0 => select_event(h, v[0], v[1], v[2]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The event function and the file string are the ROM's literals.
#[test]
fn literals_match_rom() {
    let Some(ovy) = overlay21() else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0218_7F00) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    assert_eq!(word(0x021B_59AC), EVENT_SELECT);
    assert_eq!(word(0x021B_58A8), FILE);
    assert_eq!(word(0x021B_58AC), COLLECT_IDS);
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_fld_battle`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_fld_battle",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_fld_battle.spec"),
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
