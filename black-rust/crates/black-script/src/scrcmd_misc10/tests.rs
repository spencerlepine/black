use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216A420, "21e6ccfe4b7cd07f"),
    (0x0216A440, "172ae6fae57fb563"),
    (0x0216A470, "dea7479789f72729"),
    (0x0216A4CC, "cf437bbfb860f073"),
    (0x0216A510, "8dc5cfcedaa77c7f"),
    (0x0216A560, "7c17765e1abe233c"),
    (0x0216A594, "fcc447355894d854"),
    (0x0216A5C4, "ed770c8315130991"),
    (0x0216A5FC, "b34208797f501b1b"),
    (0x0216A634, "610cf0f39d715a01"),
    (0x0216A674, "4c819e9950fd1965"),
    (0x0216A6B4, "9dae2be03528e1e6"),
    (0x0216A6EC, "fadaf32eedd12b36"),
    (0x0216A72C, "edc98982c4a2c90b"),
    (0x0216A774, "abf05a7ea6cbd3d2"),
    (0x0216A7A0, "1c925a7d2bbc49f7"),
    (0x0216A838, "09ff5a60f11888a6"),
    (0x0216A884, "7cd84e486884d466"),
    (0x0216A8F0, "ad3674bdb7d0ec2b"),
    (0x0216AAE4, "dc043948ac8ac24c"),
    (0x0216ABD8, "08d838466216a9a4"),
    (0x0216B504, "da23d51c05a9d4a0"),
    (0x0216B538, "67e732fc1407b9a5"),
    (0x0216B578, "25e7f058718e202b"),
    (0x0216B5AC, "370f953244d6d584"),
    (0x0216B60C, "61eb5c35306ee3d7"),
    (0x0216B63C, "471214cfba744e06"),
    (0x0216B6DC, "06ba1370a5d08090"),
    (0x0216B720, "2b1e1979717b9704"),
    (0x0216B7B0, "c1f7de15cec79b9f"),
    (0x0216B7F8, "e7f2f6e316cfb296"),
    (0x0216B81C, "25a20d39603c390d"),
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
    /// A stub `args=v..,out:N` whose out pointer is the address `out` in the memory window: the stub fills N bytes there.
    fn s_out<const N: usize>(&mut self, addr: u32, vals: &[u32], out: u32) -> u32 {
        let mut buf = [0u8; N];
        let r = {
            let mut a: Vec<Arg> = vals.iter().map(|&v| Arg::V(v)).collect();
            a.push(Arg::Out(&mut buf));
            self.stubs.call(addr, &mut a)
        };
        for (i, b) in buf.iter().enumerate() {
            self.mem.w8(out + i as u32, *b);
        }
        r
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
impl Misc10Hooks for Rom<'_> {
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2EBC, &[gamedata])
    }
    fn fun_02012f50(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F50, &[gamedata])
    }
    fn fun_02008c58(&mut self, save: u32) -> u32 {
        self.s(0x0200_8C58, &[save])
    }
    fn fun_02008e68(&mut self, block: u32, v: u32) {
        self.s(0x0200_8E68, &[block, v]);
    }
    fn fun_02008e78(&mut self, block: u32) -> u32 {
        self.s(0x0200_8E78, &[block])
    }
    fn fun_0200c0e8(&mut self, save: u32) -> u32 {
        self.s(0x0200_C0E8, &[save])
    }
    fn fun_0200c0f0(&mut self, save: u32) -> u32 {
        self.s(0x0200_C0F0, &[save])
    }
    fn fun_0200c224(&mut self, block: u32) -> u32 {
        self.s(0x0200_C224, &[block])
    }
    fn fun_0200c22c(&mut self, block: u32, n: u32) {
        self.s(0x0200_C22C, &[block, n]);
    }
    fn fun_0200c32c(&mut self, block: u32) -> u32 {
        self.s(0x0200_C32C, &[block])
    }
    fn fun_0200c334(&mut self, block: u32, v: u32) {
        self.s(0x0200_C334, &[block, v]);
    }
    fn fun_0200c33c(&mut self, block: u32, i: u32) -> u32 {
        self.s(0x0200_C33C, &[block, i])
    }
    fn fun_0200c344(&mut self, block: u32, i: u32, v: u32) {
        self.s(0x0200_C344, &[block, i, v]);
    }
    fn fun_0200c358(&mut self, block: u32, i: u32, v: u32) {
        self.s(0x0200_C358, &[block, i, v]);
    }
    fn fun_0200c384(&mut self, block: u32, lo: u32, hi: u32) {
        self.s(0x0200_C384, &[block, lo, hi]);
    }
    fn fun_0200e394(&mut self, save: u32) -> u32 {
        self.s(0x0200_E394, &[save])
    }
    fn fun_0200e3a0(&mut self, block: u32) -> u32 {
        self.s(0x0200_E3A0, &[block])
    }
    fn fun_0200e3e8(&mut self, x: u32, a: u32, b: u32) {
        self.s(0x0200_E3E8, &[x, a, b]);
    }
    fn fun_0200e4b0(&mut self, block: u32, c: u32, k: u32) -> u32 {
        self.s(0x0200_E4B0, &[block, c, k])
    }
    fn fun_0200e4f8(&mut self, block: u32, c: u32, k: u32) -> u32 {
        self.s(0x0200_E4F8, &[block, c, k])
    }
    fn fun_02027100(&mut self, x: u32) {
        self.s(0x0202_7100, &[x]);
    }
    fn fun_02027138(&mut self, n: u32) {
        self.s(0x0202_7138, &[n]);
    }
    fn fun_0203f524(&mut self) -> u64 {
        self.stubs.call64(0x0203_F524, &mut [])
    }
    fn fun_0200ca1c(&mut self, x: u32) -> u32 {
        self.s(0x0200_CA1C, &[x])
    }
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32) {
        self.s(0x0200_CFA0, &[dex, mon]);
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA18, &[party])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.s(0x0201_AB28, &[party, i])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.s(0x0201_7E1C, &[mon, id, x])
    }
    fn fun_02017988(&mut self, mon: u32, form: u32) {
        self.s(0x0201_7988, &[mon, form]);
    }
    fn fun_021670f8(&mut self, gamesys: u32) -> u32 {
        self.s(0x0216_70F8, &[gamesys])
    }
    fn fun_02168bf0(&mut self, gamesys: u32, rec: u32, a: u32, b: u32, var: u32) -> u32 {
        self.s(0x0216_8BF0, &[gamesys, rec, a, b, var])
    }
    fn fun_02168cf4(&mut self, gamesys: u32, rec: u32) -> u32 {
        self.s(0x0216_8CF4, &[gamesys, rec])
    }
    fn fun_02168e5c(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        self.s(0x0216_8E5C, &[gamesys, a, b])
    }
    fn fun_0216d568(&mut self, mmdl: u32, flags: u32) {
        self.s(0x0216_D568, &[mmdl, flags]);
    }
    fn fun_0216d88c(&mut self, mmdl: u32, out: u32) {
        self.s_out::<12>(0x0216_D88C, &[mmdl], out);
    }
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]) {
        self.stubs
            .call(0x0216_D89C, &mut [Arg::V(mmdl), Arg::In(pos)]);
    }
    fn fun_0216de94(&mut self, sys: u32, id: u32) -> u32 {
        self.s(0x0216_DE94, &[sys, id])
    }
    fn fun_0216aed0(&mut self, ctx: u32, v: u32) -> u32 {
        self.s(0x0216_AED0, &[ctx, v])
    }
    fn fun_0216af1c(&mut self, ctx: u32, p: u32, n: u32, k: &mut [u8; 4]) -> u32 {
        self.stubs.call(
            0x0216_AF1C,
            &mut [Arg::V(ctx), Arg::V(p), Arg::V(n), Arg::Out(k)],
        )
    }
    fn fun_0216af74(&mut self, gamesys: u32, fm: u32, q: u32, k: u32, var: u32, n: u32) -> u32 {
        self.s(0x0216_AF74, &[gamesys, fm, q, k, var, n])
    }
    fn fun_0216b8b8(&mut self, save: u32, var: u32) {
        self.s(0x0216_B8B8, &[save, var]);
    }
    fn fun_0216b91c(&mut self, save: u32, v: u32, var: u32) {
        self.s(0x0216_B91C, &[save, v, var]);
    }
    fn fun_0216bb18(&mut self, save: u32) -> u32 {
        self.s(0x0216_BB18, &[save])
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C7C, &[fm])
    }
    fn fun_021d0a48(&mut self, gamesys: u32, rec: u32) {
        self.s(0x021D_0A48, &[gamesys, rec]);
    }
    fn fun_021d0a74(&mut self, rec: u32, v: u32) {
        self.s(0x021D_0A74, &[rec, v]);
    }
    fn fun_021d0ab8(&mut self, rec: u32, v: u32) {
        self.s(0x021D_0AB8, &[rec, v]);
    }
    fn fun_021d0ac0(&mut self, gamesys: u32, rec: u32, v: u32) -> u32 {
        self.s(0x021D_0AC0, &[gamesys, rec, v])
    }
    fn fun_021d0bd8(&mut self, gamesys: u32, rec: u32, a: u32, b: u32) -> u32 {
        self.s(0x021D_0BD8, &[gamesys, rec, a, b])
    }
    fn fun_021d0be4(&mut self, gamesys: u32, rec: u32, var: u32) -> u32 {
        self.s(0x021D_0BE4, &[gamesys, rec, var])
    }
    fn fun_021d0db0(&mut self, gamesys: u32, rec: u32, a: u32, b: u32) {
        self.s(0x021D_0DB0, &[gamesys, rec, a, b]);
    }
    fn fun_021d0fe0(&mut self, gamesys: u32, rec: u32) -> u32 {
        self.s(0x021D_0FE0, &[gamesys, rec])
    }
    fn fun_021d0fe8(&mut self, gamesys: u32, rec: u32, v: u32) -> u32 {
        self.s(0x021D_0FE8, &[gamesys, rec, v])
    }
    fn fun_021d1028(&mut self, gamesys: u32) -> u32 {
        self.s(0x021D_1028, &[gamesys])
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
        0x0216_A838 => event_549(h, v[0], v[1], v[2]) as u64,
        0x0216_A8F0 => event_550(h, v[0], v[1], v[2]) as u64,
        0x0216_AAE4 => {
            update_season_forms(h, v[0], v[1], v[2]);
            0
        }
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// The ROM build file `name` (`$BLACK_ROM_BUILD/<name>`), or `None` (skip) when it is missing and `REQUIRE_ROM` is not set.
fn rom_file(name: &str) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/{name}")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/{name} is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C) and every handler is
/// ported; command 359 is the musical handler and stays out; the event functions are the ROM's literals.
#[test]
fn commands_match_rom() {
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    assert!(!PORTED.contains(&CMD_359_MUSICAL));
    assert!(COMMANDS.iter().all(|c| c.0 != 359));
    let Some(ovy) = rom_file("OVY_10.sbin") else {
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
    assert_eq!(
        word(crate::script_sys::CMD_TABLE + 4 * 359),
        CMD_359_MUSICAL | 1
    );
    // literal pools of 0216a7a0 / 0216a884
    assert_eq!(word(0x0216_A834), EVENT_549);
    assert_eq!(word(0x0216_A8EC), EVENT_550);
}

/// The embedded command-532 tables are the ROM's (overlay 10).
#[test]
fn tables_532_match_rom() {
    let Some(ovy) = rom_file("OVY_10.sbin") else {
        return;
    };
    let at = |a: u32, n: usize| {
        let o = (a - 0x0215_5160) as usize;
        ovy[o..o + n].to_vec()
    };
    assert_eq!(at(COUNTS_532_ADDR, 32), COUNTS_532);
    assert_eq!(at(ROWS_532_ADDR, 17 * 30), ROWS_532.concat());
}

/// [`fx_sin`] / [`fx_cos`] are `FX_SinCosTable_` (main 0x020A1B38, `$BLACK_ROM_BUILD/main.sbin` loaded at 0x02004000).
#[test]
fn sincos_matches_rom() {
    assert_eq!((fx_sin(0), fx_cos(0)), (0, 4096));
    assert_eq!((fx_sin(1024), fx_cos(1024)), (4096, 0));
    let Some(main) = rom_file("main.sbin") else {
        return;
    };
    let o = (0x020A_1B38 - 0x0200_4000) as usize;
    for i in 0..4096u32 {
        let e = o + 4 * i as usize;
        let s = i16::from_le_bytes([main[e], main[e + 1]]);
        let c = i16::from_le_bytes([main[e + 2], main[e + 3]]);
        assert_eq!((fx_sin(i), fx_cos(i)), (s, c), "entry {i}");
    }
}
