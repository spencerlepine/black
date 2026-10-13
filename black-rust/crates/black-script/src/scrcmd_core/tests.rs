use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02159204, "1f139533d7bc5ad5"),
    (0x02159240, "c83dd6fe2d1c969f"),
    (0x02159244, "85de8472918c878a"),
    (0x02159248, "c06748fbcf66ea2f"),
    (0x02159254, "30ef62d3523da770"),
    (0x02159260, "45a3f16203dd672a"),
    (0x02159284, "b48d42d84a0d43a6"),
    (0x0215929C, "e0e348743fac42ff"),
    (0x021592A8, "38afecd3eee50e9b"),
    (0x021592C0, "5694f1700b733e05"),
    (0x021592D8, "906672486f5a4aec"),
    (0x021592F0, "28c1b43386d0ebcb"),
    (0x0215930C, "5a93c80b15b69ba4"),
    (0x0215932C, "7f0279e49d64740f"),
    (0x02159340, "7c7b433de7f5d033"),
    (0x02159370, "e03e5b977e51250c"),
    (0x02159398, "5908ab7095461dc8"),
    (0x021593B8, "eeb0ac39e8909a44"),
    (0x021593DC, "fed23e364c1502e2"),
    (0x021593F0, "b54077b650bc897c"),
    (0x02159404, "5c19f176c192ee90"),
    (0x02159420, "49ff3e942cffcece"),
    (0x0215942C, "1101979d49589d1e"),
    (0x0215944C, "2946ff26813d6395"),
    (0x0215946C, "8a61530021955efc"),
    (0x0215948C, "7048c91f3bd0b004"),
    (0x021594B0, "b5db122b165c436c"),
    (0x021594E0, "06e24e9cbc8f0be9"),
    (0x02159570, "89dd29848e53cde9"),
    (0x021595A0, "008415e10dbe54a3"),
    (0x021595CC, "eb9e1dcdec3157d7"),
    (0x02159644, "138c84a73f367b90"),
    (0x02159658, "60c7b28f36a0d2be"),
    (0x02159670, "26816fbcf1213dd5"),
    (0x021596B4, "a0d2eca71c0353fc"),
    (0x021596E8, "f42f34c767a7197f"),
    (0x02159708, "0f66949f300be1c8"),
    (0x02159728, "08590891cdf76ed6"),
    (0x02159750, "882c8b03cc56595f"),
    (0x02159778, "ba1d45aa5089266c"),
    (0x021597AC, "f4b9963bfc3b8454"),
    (0x021597CC, "02563ad68f6646e5"),
    (0x021597EC, "5e4d8099d62e8be3"),
    (0x0215980C, "595f91199a8efd5e"),
    (0x02159830, "2b2a48f681ce9351"),
    (0x02159854, "2d1e0d9a327b16ac"),
    (0x02159874, "3b2304e5adbaf643"),
    (0x02159894, "49da92e8a6cdf261"),
    (0x021598AC, "4105461a7d6a2b16"),
    (0x021598C8, "167fefcb5bb07204"),
    (0x021598E4, "258c42e6c6e4f981"),
    (0x021598EC, "23207eaae01eb1ac"),
    (0x02159934, "40d103ac32807b14"),
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

/// Run `f` on the case's memory: every non-empty argument buffer at its address; the buffers are put back afterwards.
fn with_rom(c: &mut Case, f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
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
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for (i, (_, bytes)) in keep.into_iter().zip(wins) {
        c.bufs[i] = bytes;
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(c, |h, v| match addr {
        0x0215_9204 => init_procs_event(h, v[0]) as u64,
        0x0215_9254 => wait_countdown(h, v[0], v[1]) as u64,
        0x0215_932C => compare(v[0], v[1]) as u64,
        0x0215_95A0 => wait_call_script(h, v[0], v[1]) as u64,
        a => dispatch(h, a, v[0], v[1])
            .unwrap_or_else(|| unreachable!("{a:#010x} is PORTED but not wired into the test"))
            as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_core`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_core",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_core.spec"),
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

/// The embedded condition table is the ROM's.
#[test]
fn cond_table_matches_rom() {
    let Some(ovy) = overlay10() else {
        return;
    };
    let o = (COND_ADDR - 0x0215_5160) as usize;
    assert_eq!(&ovy[o..o + COND.len()], &COND);
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), and every handler
/// is dispatched.
#[test]
fn commands_match_rom() {
    let Some(ovy) = overlay10() else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}

#[test]
fn compare_orders() {
    assert_eq!(compare(1, 2), 0);
    assert_eq!(compare(2, 2), 1);
    assert_eq!(compare(3, 2), 2);
}
