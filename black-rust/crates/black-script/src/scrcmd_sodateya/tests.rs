use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215CE98, "aedb3c6d0c351d93"),
    (0x0215CEBC, "1a76a0685250c05b"),
    (0x0215CEE8, "6bb2ef6d63043fee"),
    (0x0215CF00, "10154dba2d5b23c2"),
    (0x0215CF54, "ba259903630231ad"),
    (0x0215CF84, "ccaa81367c8e7718"),
    (0x0215CFB4, "ca79b7c650191c7c"),
    (0x0215CFD8, "699dc5563511bf76"),
    (0x0215CFFC, "8033c18dd85e49f5"),
    (0x0215D034, "6466cc6e9fb736a6"),
    (0x0215D06C, "25f472ea20b5e0af"),
    (0x0215D0A4, "bdaba35b2f932575"),
    (0x0215D0D4, "c68e3d407a1f5595"),
    (0x0215D110, "ad660169234c229b"),
    (0x0215D14C, "5861e3016c1de71b"),
    (0x0215D198, "7f3652f705cf42cf"),
    (0x0215D1E4, "e220737c4e5b1863"),
    (0x0215D2A4, "04562f9e2cf6c7b9"),
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
impl SodateyaHooks for Rom<'_> {
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2934, &[gamedata])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2EBC, &[gamedata])
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        self.s(0x0201_2374, &[event, child]);
    }
    fn fun_0200bdb0(&mut self, save: u32) {
        self.s(0x0200_BDB0, &[save]);
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0200_CA1C, &[gamedata])
    }
    fn fun_0200ca38(&mut self, x: u32) -> u32 {
        self.s(0x0200_CA38, &[x])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA18, &[party])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.s(0x0201_7E1C, &[mon, id, x])
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32 {
        self.s(0x0201_44DC, &[gamesys, fm])
    }
    fn fun_02014910(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32 {
        self.s(0x0201_4910, &[gamesys, fm, a, b])
    }
    fn fun_020145cc(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_45CC, &[gamesys])
    }
    fn fun_02188d50(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D50, &[fm])
    }
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32 {
        self.s(0x021B_E8D4, &[gamesys, fm, a, b])
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
        self.s(0x021B_E92C, &[gamesys, fm, a, b, c, d, e])
    }
    fn fun_021c3204(&mut self, daycare: u32, party: u32, v: u32) {
        self.s(0x021C_3204, &[daycare, party, v]);
    }
    fn fun_021c3270(&mut self, daycare: u32, v: u32, party: u32) {
        self.s(0x021C_3270, &[daycare, v, party]);
    }
    fn fun_021c32f4(&mut self, daycare: u32, party: u32) {
        self.s(0x021C_32F4, &[daycare, party]);
    }
    fn fun_021c335c(&mut self, daycare: u32) {
        self.s(0x021C_335C, &[daycare]);
    }
    fn fun_021c3400(&mut self, daycare: u32) -> u32 {
        self.s(0x021C_3400, &[daycare])
    }
    fn fun_021c340c(&mut self, daycare: u32) -> u32 {
        self.s(0x021C_340C, &[daycare])
    }
    fn fun_021c341c(&mut self, daycare: u32, slot: u32) -> u32 {
        self.s(0x021C_341C, &[daycare, slot])
    }
    fn fun_021c3490(&mut self, daycare: u32, slot: u32) -> u32 {
        self.s(0x021C_3490, &[daycare, slot])
    }
    fn fun_021c34b4(&mut self, daycare: u32, slot: u32) -> u32 {
        self.s(0x021C_34B4, &[daycare, slot])
    }
    fn fun_021c34c4(&mut self, daycare: u32) -> u32 {
        self.s(0x021C_34C4, &[daycare])
    }
    fn fun_021c34f8(&mut self, daycare: u32, slot: u32) -> u32 {
        self.s(0x021C_34F8, &[daycare, slot])
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
        0x0215_D198 => gender_mark(h, v[0]) as u64,
        0x0215_D1E4 => event_247(h, v[0], v[1], v[2]) as u64,
        0x0215_D2A4 => party_select_event(h, v[0], v[1], v[2]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_sodateya`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_sodateya",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_sodateya.spec"),
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

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), and every handler is
/// ported; the event function and the file string are the ROM's literals.
#[test]
fn commands_match_rom() {
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    let Some(ovy) = overlay10() else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0215_5160) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    for &(id, addr) in COMMANDS {
        assert_eq!(
            word(crate::script_sys::CMD_TABLE + 4 * id as u32),
            addr | 1,
            "command {id}"
        );
    }
    // literal pool of 0215d2a4
    assert_eq!(word(0x0215_D344), FILE);
    assert_eq!(word(0x0215_D348), EVENT_247);
}
