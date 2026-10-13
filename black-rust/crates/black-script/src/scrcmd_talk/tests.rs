use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B0084, "44bfafb1cb50c8be"),
    (0x021B03E4, "2d987ccd14ce5ca7"),
    (0x021B0480, "4ff798c83efd316c"),
    (0x021B09EC, "badfd511a420926a"),
    (0x021B1550, "d64c69725ee2986d"),
    (0x021B162C, "5a3de7bce500dc44"),
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

impl TalkHooks for Rom<'_> {
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5DC, &[ctx])
    }
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5F4, &[ctx])
    }
    fn fun_0215a60c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A60C, &[ctx])
    }
    fn fun_0215a614(&mut self, ctx: u32, w: u32) {
        self.s(0x0215_A614, &[ctx, w]);
    }
    fn fun_0215a61c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A61C, &[ctx])
    }
    fn fun_0215a624(&mut self, ctx: u32, v: u32) {
        self.s(0x0215_A624, &[ctx, v]);
    }
    fn fun_0215a630(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A630, &[ctx])
    }
    fn fun_0215a654(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A654, &[ctx])
    }
    fn fun_0215a660(&mut self, ctx: u32, v: u32) {
        self.s(0x0215_A660, &[ctx, v]);
    }
    fn fun_0215a684(&mut self, ctx: u32, n: u32) {
        self.s(0x0215_A684, &[ctx, n]);
    }
    fn fun_0215a690(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A690, &[ctx])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F58, &[sw])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F74, &[sw])
    }
    fn fun_02158f78(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F78, &[sw])
    }
    fn fun_02158f7c(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F7C, &[sw])
    }
    fn fun_02159144(&mut self, bit: u32) {
        self.s(0x0215_9144, &[bit]);
    }
    fn fun_02159168(&mut self, bit: u32) {
        self.s(0x0215_9168, &[bit]);
    }
    fn fun_02159190(&mut self, bit: u32) -> u32 {
        self.s(0x0215_9190, &[bit])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_02188c54(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C54, &[fm])
    }
    fn fun_02190be4(&mut self, msgbg: u32, arg: u32) -> u32 {
        self.s(0x0219_0BE4, &[msgbg, arg])
    }
    fn fun_02190c78(&mut self, menu: u32) -> u32 {
        self.s(0x0219_0C78, &[menu])
    }
    fn fun_021909bc(&mut self, menu: u32) {
        self.s(0x0219_09BC, &[menu]);
    }
    fn fun_02190fa0(&mut self, msgbg: u32, msgdata: u32, style: u32) -> u32 {
        self.s(0x0219_0FA0, &[msgbg, msgdata, style])
    }
    fn fun_02191008(&mut self, window: u32) {
        self.s(0x0219_1008, &[window]);
    }
    fn fun_02191164(&mut self, window: u32) {
        self.s(0x0219_1164, &[window]);
    }
    fn fun_02191134(&mut self, window: u32) {
        self.s(0x0219_1134, &[window]);
    }
    fn fun_02191794(&mut self, window: u32) {
        self.s(0x0219_1794, &[window]);
    }
    fn fun_021914cc(&mut self, window: u32) {
        self.s(0x0219_14CC, &[window]);
    }
    fn fun_0202f038(&mut self, obj: u32) {
        self.s(0x0202_F038, &[obj]);
    }
    fn fun_020362dc(&mut self) -> u32 {
        self.s(0x0203_62DC, &[])
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.s(0x0200_61E4, &[se]);
    }
    fn fun_02045cac(&mut self, msgdata: u32, msg: u32, strbuf: u32) {
        self.s(0x0204_5CAC, &[msgdata, msg, strbuf]);
    }
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32) {
        self.s(0x0201_F250, &[wordset, dst, src]);
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
        0x021B_0084 => yes_no_event(h, v[0], v[1], v[2]) as u64,
        0x021B_03E4 => {
            talk_open(h, v[0], v[1]);
            0
        }
        0x021B_0480 => {
            talk_close(h, v[0]);
            0
        }
        0x021B_09EC => {
            balloon_tail(h, v[0], v[1], v[2]);
            0
        }
        0x021B_1550 => wait_button(h, v[0], v[1]) as u64,
        0x021B_162C => message_text(h, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_talk`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_talk",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_talk.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}
