use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B2830, "7d712a6a2d399450"),
    (0x021B2860, "dbb77b7add5979c4"),
    (0x021B2874, "9647f16bd923b9ce"),
    (0x021B2890, "d055a6b00efd3b36"),
    (0x021B28E0, "ffcbfef3e727e986"),
    (0x021B2910, "695718276e43c6d4"),
    (0x021B2940, "453638ec6df0a295"),
    (0x021B2970, "be12541602b3cf1d"),
    (0x021B29A0, "479f24cc23114644"),
    (0x021B29D0, "210ceea405de5a12"),
    (0x021B2A00, "1bfa8458e6732ed3"),
    (0x021B2A30, "fe369b76ae8b3f75"),
    (0x021B2A60, "57ebf4c610106a21"),
    (0x021B2A90, "c088ccd8bb84ffd7"),
    (0x021B2AA0, "0bf21af974bc1277"),
    (0x021B2AB0, "8c53ca06b0743c21"),
    (0x021B2ADC, "4c28de38c548296d"),
    (0x021B2B24, "ccaae0ca4e819def"),
    (0x021B2B5C, "e831b67a0d9c4c07"),
    (0x021B2B94, "f426613eb23083cd"),
    (0x021B2BD8, "1d2f01377f54b56f"),
    (0x021B2BEC, "64d444f5fdc92c64"),
    (0x021B2C08, "9866de67ba79d78e"),
    (0x021B2C5C, "d278f22474948a95"),
    (0x021B2CA0, "e7936dbfe426b2a7"),
    (0x021B2CE8, "7348dd9a255b95ae"),
    (0x021B2D34, "663313d314a7c532"),
    (0x021B2D98, "449937585cc0ee57"),
    (0x021B2DBC, "44507758dc0570c1"),
    (0x021B2E08, "dc2075aef181674a"),
    (0x021B2E3C, "a3029edfd3dc96ac"),
    (0x021B2E60, "79fc3ed6baea0579"),
    (0x021B2E80, "3f35cca2e6682ca3"),
    (0x021B2EBC, "1809a7444838b98f"),
    (0x021B2F18, "ef5c62eca926919d"),
    (0x021B2FB0, "4615fc8f7d0d3b09"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer (and `global`) of the case in one memory window set (at the emulator's
/// addresses).
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
impl ScreenHooks for Rom<'_> {
    fn fun_02158f50(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F50, &[sw])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02012984(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2984, &[gamedata])
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_02030da8(&mut self, taskman: u32, f: u32, work: u32, prio: u32) {
        self.s(0x0203_0DA8, &[taskman, f, work, prio]);
    }
    fn fun_02030eac(&mut self, task: u32) {
        self.s(0x0203_0EAC, &[task]);
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.s(0x0200_61E4, &[se]);
    }
    fn fun_0204e720(&mut self, mode: u32, start: u32, end: u32, wait: u32) {
        self.s(0x0204_E720, &[mode, start, end, wait]);
    }
    fn fun_0204e804(&mut self) -> u32 {
        self.s(0x0204_E804, &[])
    }
    fn fun_02043658(&mut self, x: u32) -> u32 {
        self.s(0x0204_3658, &[x])
    }
    fn fun_02043534(&mut self, x: u32) {
        self.s(0x0204_3534, &[x]);
    }
    fn fun_02188c98(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C98, &[fm])
    }
    fn fun_02188c9c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C9C, &[fm])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CB4, &[fm])
    }
    fn fun_02188d20(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D20, &[fm])
    }
    fn fun_02188d30(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D30, &[fm])
    }
    fn fun_02188da0(&mut self, x: u32) {
        self.s(0x0218_8DA0, &[x]);
    }
    fn fun_02188db0(&mut self, fm: u32) {
        self.s(0x0218_8DB0, &[fm]);
    }
    fn fun_02189a1c(&mut self, fm: u32, on: u32) {
        self.s(0x0218_9A1C, &[fm, on]);
    }
    fn fun_02189a24(&mut self, fm: u32) -> u32 {
        self.s(0x0218_9A24, &[fm])
    }
    fn fun_0218dc6c(&mut self, x: u32) -> u32 {
        self.s(0x0218_DC6C, &[x])
    }
    fn fun_0218ba98(&mut self, sys: u32, id: u32) -> u32 {
        self.s(0x0218_BA98, &[sys, id])
    }
    fn fun_0218bacc(&mut self, sys: u32, o: u32) -> u32 {
        self.s(0x0218_BACC, &[sys, o])
    }
    fn fun_0218c54c(&mut self, o: u32, v: u32) {
        self.s(0x0218_C54C, &[o, v]);
    }
    fn fun_0218c574(&mut self, sys: u32, o: u32, v: u32) {
        self.s(0x0218_C574, &[sys, o, v]);
    }
    fn fun_0218c654(&mut self, sys: u32, id: u32, pos: &[u8; 12]) -> u32 {
        self.stubs.call(
            0x0218_C654,
            &mut [Arg::V(sys), Arg::V(id), Arg::In(&pos[..])],
        )
    }
    fn fun_0218c69c(&mut self, sys: u32, o: u32, a: u32, b: u32) {
        self.s(0x0218_C69C, &[sys, o, a, b]);
    }
    fn fun_0218c7d8(&mut self, sys: u32, id: u32, pos: &[u8; 12]) -> u32 {
        self.stubs.call(
            0x0218_C7D8,
            &mut [Arg::V(sys), Arg::V(id), Arg::In(&pos[..])],
        )
    }
    fn fun_0218c860(&mut self, o: u32) {
        self.s(0x0218_C860, &[o]);
    }
    fn fun_0218c88c(&mut self, o: u32, a: u32, mode: u32) {
        self.s(0x0218_C88C, &[o, a, mode]);
    }
    fn fun_0218c89c(&mut self, o: u32, v: u32) {
        self.s(0x0218_C89C, &[o, v]);
    }
    fn fun_0218c8d8(&mut self, o: u32) -> u32 {
        self.s(0x0218_C8D8, &[o])
    }
    fn fun_0218c990(&mut self, o: u32, se: &mut [u8; 4]) -> u32 {
        self.stubs
            .call(0x0218_C990, &mut [Arg::V(o), Arg::Out(&mut se[..])])
    }
    fn fun_0218c9a4(&mut self, o: u32) -> u32 {
        self.s(0x0218_C9A4, &[o])
    }
    fn fun_021bc5a0(&mut self, a: u32, b: u32) {
        self.s(0x021B_C5A0, &[a, b]);
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
    fn fun_021c0aa0(&mut self, gamesys: u32, event: u32) -> u32 {
        self.s(0x021C_0AA0, &[gamesys, event])
    }
    fn fun_021c0820(&mut self, gamesys: u32, event: u32, v: u32) -> u32 {
        self.s(0x021C_0820, &[gamesys, event, v])
    }
    fn fun_021c978c(&mut self, event: u32, gamesys: u32, fm: u32) -> u32 {
        self.s(0x021C_978C, &[event, gamesys, fm])
    }
    fn fun_021c9828(&mut self, event: u32, gamesys: u32, fm: u32) -> u32 {
        self.s(0x021C_9828, &[event, gamesys, fm])
    }
    fn fun_021c98f4(&mut self, event: u32, gamesys: u32, fm: u32, v: u32) -> u32 {
        self.s(0x021C_98F4, &[event, gamesys, fm, v])
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
    let globals: &[u32] = if addr == 0x021B_2E60 || addr == 0x021B_2E80 {
        &[WAIT_OBJ_ID]
    } else {
        &[]
    };
    with_rom(c, globals, |h, v| match addr {
        0x021B_2830 => cmd_fade(h, v[0], v[1]) as u64,
        0x021B_2860 => wait_fade(h, v[0], v[1]) as u64,
        0x021B_2874 => wait_field_fade(h, v[0], v[1]) as u64,
        0x021B_2890 => cmd_436(h, v[0], v[1]) as u64,
        0x021B_28E0 => cmd_434(h, v[0], v[1]) as u64,
        0x021B_2910 => cmd_field_fade_429(h, v[0], v[1]) as u64,
        0x021B_2940 => cmd_430(h, v[0], v[1]) as u64,
        0x021B_2970 => cmd_431(h, v[0], v[1]) as u64,
        0x021B_29A0 => cmd_432(h, v[0], v[1]) as u64,
        0x021B_29D0 => cmd_437(h, v[0], v[1]) as u64,
        0x021B_2A00 => cmd_438(h, v[0], v[1]) as u64,
        0x021B_2A30 => cmd_439(h, v[0], v[1]) as u64,
        0x021B_2A60 => cmd_440(h, v[0], v[1]) as u64,
        0x021B_2A90 => cmd_wait_fade(h, v[0], v[1]) as u64,
        0x021B_2AA0 => cmd_wait_field_fade(h, v[0], v[1]) as u64,
        0x021B_2AB0 => cmd_293(h, v[0], v[1]) as u64,
        0x021B_2ADC => cmd_303(h, v[0], v[1]) as u64,
        0x021B_2B24 => cmd_304(h, v[0], v[1]) as u64,
        0x021B_2B5C => cmd_305(h, v[0], v[1]) as u64,
        0x021B_2B94 => cmd_306(h, v[0], v[1]) as u64,
        0x021B_2BD8 => obj_system(h, v[0]) as u64,
        0x021B_2BEC => obj_at(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_2C08 => cmd_299(h, v[0], v[1]) as u64,
        0x021B_2C5C => cmd_302(h, v[0], v[1]) as u64,
        0x021B_2CA0 => cmd_301(h, v[0], v[1]) as u64,
        0x021B_2CE8 => cmd_294(h, v[0], v[1]) as u64,
        0x021B_2D34 => cmd_295(h, v[0], v[1]) as u64,
        0x021B_2D98 => cmd_296(h, v[0], v[1]) as u64,
        0x021B_2DBC => cmd_297(h, v[0], v[1]) as u64,
        0x021B_2E08 => cmd_292(h, v[0], v[1]) as u64,
        0x021B_2E3C => cmd_300(h, v[0], v[1]) as u64,
        0x021B_2E60 => cmd_298(h, v[0], v[1]) as u64,
        0x021B_2E80 => wait_298(h, v[0], v[1]) as u64,
        0x021B_2EBC => {
            field_fade_start(h, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021B_2F18 => {
            field_fade_task(h, v[0], v[1]);
            0
        }
        0x021B_2FB0 => cmd_594(h, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_screen`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_screen",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_screen.spec"),
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

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), every handler is
/// dispatched, and every function of the area is in [`PORTED`].
#[test]
fn commands_match_rom() {
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
