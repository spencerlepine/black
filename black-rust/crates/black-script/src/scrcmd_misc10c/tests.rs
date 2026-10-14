use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02164778, "8b2e6f0cc5138d46"),
    (0x02164910, "ccd24f4f7e25859a"),
    (0x02164978, "e8112b7203cb64a6"),
    (0x02164994, "0bdd859e482d2628"),
    (0x021649E8, "b368c16ee8279fa9"),
    (0x0216A3F8, "c0fc237b4ffdc6c9"),
    (0x0216B86C, "31058d3724025929"),
    (0x0216B8B8, "b93ac5ce28975633"),
    (0x0216B91C, "98ceaf3ff0499f9a"),
    (0x0216B970, "a6738f3ebaf313f2"),
    (0x0216B994, "0bb3b239900a213b"),
    (0x0216B9D8, "beecb1be746f452f"),
    (0x0216BA08, "fa469571b04010a9"),
    (0x0216BA40, "26b541fe1a9d4aec"),
    (0x0216BA88, "b4fea4573a99aa02"),
    (0x0216BAE8, "2b2aa67dec3b0d98"),
    (0x0216BB18, "22ea554492dd8df2"),
    (0x0216BB70, "3f67f062d99e4b29"),
    (0x0216BB80, "65da52ef591a7548"),
    (0x0216BB90, "e2d8a51677efd8d3"),
    (0x0216BBB4, "584971c4771d2f9f"),
    (0x0216BBC4, "d0d179ed6c872155"),
    (0x0216BBD4, "02be886297305b15"),
    (0x0216BC88, "daa95f34b445ea56"),
    (0x0216BCE4, "1b28645f14dee54c"),
    (0x0216BD1C, "50da74a94ada9a8c"),
    (0x0216BD54, "b65feff119d710e5"),
    (0x0216BE20, "9165c747bdc766e9"),
    (0x0216BEC0, "a8fe982ffcd9f6f0"),
    (0x0216BF38, "0e94fa5ecdae8aa9"),
    (0x0216BF74, "3a90d55a1d43f336"),
    (0x0216CC30, "1a202f04fb3322ba"),
    (0x0216CC40, "ece4f2a0aedfd201"),
    (0x0216F8B8, "b08c7bed33f9b897"),
    (0x0216F8C8, "d7479ab9c56f29ab"),
    (0x0216F8DC, "44ce90cf567d1408"),
    (0x0216F900, "d25564c4f3b96454"),
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
    fn ptr(&self, addr: u32, k: usize) -> u32 {
        panic!("no address for {addr:#x} arg {k} under {:#x}", self.cur)
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
    fn fun_020124e8(&mut self, _event: u32) -> u32 {
        unreachable!("fun_020124e8: no stub in the spec")
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
impl Misc10cHooks for Rom<'_> {
    fn fun_02008530(&mut self, x: u32) -> u32 {
        let r = self.stubs.call(0x02008530, &mut [Arg::V(x)]);
        self.rets.push((0x02008530, r));
        r
    }
    fn fun_0200b3a4(&mut self, ew: u32, id: u32) -> u32 {
        let r = self.stubs.call(0x0200b3a4, &mut [Arg::V(ew), Arg::V(id)]);
        self.rets.push((0x0200b3a4, r));
        r
    }
    fn fun_0200c0e8(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200c0e8, &mut [Arg::V(save)]);
        self.rets.push((0x0200c0e8, r));
        r
    }
    fn fun_0200c0f0(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200c0f0, &mut [Arg::V(save)]);
        self.rets.push((0x0200c0f0, r));
        r
    }
    fn fun_0200c32c(&mut self, s: u32) -> u32 {
        let r = self.stubs.call(0x0200c32c, &mut [Arg::V(s)]);
        self.rets.push((0x0200c32c, r));
        r
    }
    fn fun_0200c34c(&mut self, s: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x0200c34c, &mut [Arg::V(s), Arg::V(i)]);
        self.rets.push((0x0200c34c, r));
        r
    }
    fn fun_0200c37c(&mut self, s: u32) -> u64 {
        let r = self.stubs.call64(0x0200c37c, &mut [Arg::V(s)]);
        self.rets.push((0x0200c37c, r as u32));
        r
    }
    fn fun_0200c38c(&mut self, s: u32, k: u32, v: u32) {
        let r = self
            .stubs
            .call(0x0200c38c, &mut [Arg::V(s), Arg::V(k), Arg::V(v)]);
        self.rets.push((0x0200c38c, r));
    }
    fn fun_0200d418(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200d418, &mut [Arg::V(save)]);
        self.rets.push((0x0200d418, r));
        r
    }
    fn fun_0200d468(&mut self, x: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x0200d468, &mut [Arg::V(x), Arg::V(i)]);
        self.rets.push((0x0200d468, r));
        r
    }
    fn fun_0200e394(&mut self, save: u32) -> u32 {
        let r = self.stubs.call(0x0200e394, &mut [Arg::V(save)]);
        self.rets.push((0x0200e394, r));
        r
    }
    fn fun_0200e458(&mut self, t: u32, id: u32) -> u32 {
        let r = self.stubs.call(0x0200e458, &mut [Arg::V(t), Arg::V(id)]);
        self.rets.push((0x0200e458, r));
        r
    }
    fn fun_0200e47c(&mut self, t: u32, id: u32) -> u32 {
        let r = self.stubs.call(0x0200e47c, &mut [Arg::V(t), Arg::V(id)]);
        self.rets.push((0x0200e47c, r));
        r
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)]);
        self.rets.push((0x0201210c, r));
        r
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        let r = self
            .stubs
            .call(0x02012374, &mut [Arg::V(event), Arg::V(child)]);
        self.rets.push((0x02012374, r));
    }
    fn fun_020124e4(&mut self, event: u32) -> u32 {
        let r = self.stubs.call(0x020124e4, &mut [Arg::V(event)]);
        self.rets.push((0x020124e4, r));
        r
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012934, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012934, r));
        r
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ebc, r));
        r
    }
    fn fun_02012ec4(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012ec4, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012ec4, r));
        r
    }
    fn fun_02012f50(&mut self, gamedata: u32) -> u32 {
        let r = self.stubs.call(0x02012f50, &mut [Arg::V(gamedata)]);
        self.rets.push((0x02012f50, r));
        r
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)]);
        self.rets.push((0x02017e1c, r));
        r
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        let r = self.stubs.call(0x0201aa18, &mut [Arg::V(party)]);
        self.rets.push((0x0201aa18, r));
        r
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        let r = self.stubs.call(0x0201ab28, &mut [Arg::V(party), Arg::V(i)]);
        self.rets.push((0x0201ab28, r));
        r
    }
    fn fun_0202f488(&mut self, mon: u32, k: u32) -> u32 {
        let r = self.stubs.call(0x0202f488, &mut [Arg::V(mon), Arg::V(k)]);
        self.rets.push((0x0202f488, r));
        r
    }
    fn fun_0202f630(&mut self, mon: u32, k: u32) {
        let r = self.stubs.call(0x0202f630, &mut [Arg::V(mon), Arg::V(k)]);
        self.rets.push((0x0202f630, r));
    }
    fn fun_02034a5c(&mut self, id: u32) {
        let r = self.stubs.call(0x02034a5c, &mut [Arg::V(id)]);
        self.rets.push((0x02034a5c, r));
    }
    fn fun_02034ac4(&mut self, id: u32) {
        let r = self.stubs.call(0x02034ac4, &mut [Arg::V(id)]);
        self.rets.push((0x02034ac4, r));
    }
    fn fun_0203f524(&mut self) -> u64 {
        let r = self.stubs.call64(0x0203f524, &mut []);
        self.rets.push((0x0203f524, r as u32));
        r
    }
    fn fun_020891e0(&mut self, rest: &mut [u32; 3], days: &mut [u32; 3], lo: u32, hi: u32) {
        let mut b0: Vec<u8> = rest.iter().flat_map(|w| w.to_le_bytes()).collect();
        let mut b1: Vec<u8> = days.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x020891e0,
            &mut [Arg::Out(&mut b0), Arg::Out(&mut b1), Arg::V(lo), Arg::V(hi)],
        );
        self.rets.push((0x020891e0, r));
        for (j, w) in rest.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b0[4 * j..4 * j + 4].try_into().unwrap());
        }
        for (j, w) in days.iter_mut().enumerate() {
            *w = u32::from_le_bytes(b1[4 * j..4 * j + 4].try_into().unwrap());
        }
    }
    fn fun_02159144(&mut self, f: u32) {
        let r = self.stubs.call(0x02159144, &mut [Arg::V(f)]);
        self.rets.push((0x02159144, r));
    }
    fn fun_02159168(&mut self, f: u32) {
        let r = self.stubs.call(0x02159168, &mut [Arg::V(f)]);
        self.rets.push((0x02159168, r));
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        let r = self.stubs.call(0x02188cb4, &mut [Arg::V(fm)]);
        self.rets.push((0x02188cb4, r));
        r
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
    fn fun_021d096c(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x021d096c, &mut [Arg::V(gamesys)]);
        self.rets.push((0x021d096c, r));
        r
    }
    fn fun_021f3800(&mut self, gamesys: u32) {
        let r = self.stubs.call(0x021f3800, &mut [Arg::V(gamesys)]);
        self.rets.push((0x021f3800, r));
    }
    fn fun_021f385c(&mut self, fm: u32) {
        let r = self.stubs.call(0x021f385c, &mut [Arg::V(fm)]);
        self.rets.push((0x021f385c, r));
    }
    fn fun_021f38bc(&mut self, gamesys: u32) -> u32 {
        let r = self.stubs.call(0x021f38bc, &mut [Arg::V(gamesys)]);
        self.rets.push((0x021f38bc, r));
        r
    }
    fn fun_021f39b8(
        &mut self,
        gamesys: u32,
        mode: u32,
        from: &[u32; 3],
        to: &[u32; 3],
        frames: u32,
        x: u32,
    ) -> u32 {
        let b2: Vec<u8> = from.iter().flat_map(|w| w.to_le_bytes()).collect();
        let b3: Vec<u8> = to.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self.stubs.call(
            0x021f39b8,
            &mut [
                Arg::V(gamesys),
                Arg::V(mode),
                Arg::In(&b2),
                Arg::In(&b3),
                Arg::V(frames),
                Arg::V(x),
            ],
        );
        self.rets.push((0x021f39b8, r));
        r
    }
    fn fun_021f3aa0(&mut self, gamesys: u32, a: u32, pos: &[u32; 3]) {
        let b2: Vec<u8> = pos.iter().flat_map(|w| w.to_le_bytes()).collect();
        let r = self
            .stubs
            .call(0x021f3aa0, &mut [Arg::V(gamesys), Arg::V(a), Arg::In(&b2)]);
        self.rets.push((0x021f3aa0, r));
    }
    fn fun_021f3b40(&mut self, gamesys: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021f3b40, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x021f3b40, r));
        r
    }
    fn fun_021f3b68(&mut self, gamesys: u32, x: u32) -> u32 {
        let r = self
            .stubs
            .call(0x021f3b68, &mut [Arg::V(gamesys), Arg::V(x)]);
        self.rets.push((0x021f3b68, r));
        r
    }
    fn fun_021f5260(&mut self, a: u32, b: u32, c: u32, gamesys: u32, ctx: u32) -> u32 {
        let r = self.stubs.call(
            0x021f5260,
            &mut [
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(gamesys),
                Arg::V(ctx),
            ],
        );
        self.rets.push((0x021f5260, r));
        r
    }
    fn fun_021f5550(&mut self, gamesys: u32) -> u32 {
        let _ = &gamesys;
        let r = self.stubs.call(0x021f5550, &mut []);
        self.rets.push((0x021f5550, r));
        r
    }
    fn fun_021f558c(&mut self, fm: u32) {
        let r = self.stubs.call(0x021f558c, &mut [Arg::V(fm)]);
        self.rets.push((0x021f558c, r));
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
        0x0216_49E8 => event_468(h, v[0], v[1], v[2]) as u64,
        0x0216_B8B8 => {
            challenge_state(h, v[0], v[1]);
            0
        }
        0x0216_B91C => {
            challenge_state_of(h, v[0], v[1], v[2]);
            0
        }
        0x0216_B970 => record_sum(h, v[0], v[1]) as u64,
        0x0216_B994 => goal_reached(h, v[0], v[1]) as u64,
        0x0216_B9D8 => time_running(h, v[0]) as u64,
        0x0216_BA08 => all_steps_done(h, v[0]) as u64,
        0x0216_BA40 => records_full(h, v[0], v[1]) as u64,
        0x0216_BA88 => goal_left(h, v[0]) as u64,
        0x0216_BAE8 => days_left(h, v[0]) as u64,
        0x0216_BB18 => days_elapsed(h, v[0]) as u64,
        0x0216_BB70 => chal_kind(v[0]) as u64,
        0x0216_BB80 => chal_count(v[0]) as u64,
        0x0216_BB90 => {
            let mut ids = [0u8; 3];
            chal_ids(v[0], &mut ids);
            for (k, b) in ids.into_iter().enumerate() {
                h.w8(v[1] + k as u32, b);
            }
            0
        }
        0x0216_BBB4 => chal_a(v[0]) as u64,
        0x0216_BBC4 => chal_b(v[0]) as u64,
        0x0216_BE20 => find_mon(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0216_BF74 => {
            set_key(h, v[0], v[1], v[2]);
            0
        }
        0x0216_CC40 => {
            set_flag_572(h, v[0]);
            0
        }
        0x0216_F900 => {
            set_flag_590(h, v[0]);
            0
        }
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc10c`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc10c",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc10c.spec"),
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
/// and dispatched, and the tables / literals are the build's.
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
    let o = |a: u32| (a - 0x0215_5160) as usize;
    let rd = |a: u32| u32::from_le_bytes(ovy[o(a)..o(a) + 4].try_into().unwrap());
    for &(id, a) in COMMANDS {
        assert_eq!(rd(0x0217_062C + 4 * u32::from(id)), a | 1, "command {id}");
    }
    for (k, rec) in CHALLENGES.iter().enumerate() {
        let a = o(CHALLENGES_ADDR) + 7 * k;
        assert_eq!(&ovy[a..a + 7], rec, "challenge {k}");
    }
    for (k, &key) in KEYS.iter().enumerate() {
        assert_eq!(rd(KEYS_ADDR + 4 * k as u32), key);
    }
    for (lit, val) in [
        (0x0216_49C4, EVENT_468),
        (0x0216_479C, OVERLAY_458),
        (0x0216_4974, OVERLAY_467),
        (0x0216_4A28, OVERLAY_467),
        (0x0216_BA84, RECORD_FULL),
        (0x0216_BAE4, RECORD_MIN_START),
        (0x0216_BB7C, CHALLENGES_ADDR),
        (0x0216_BF98, KEYS_ADDR),
    ] {
        assert_eq!(rd(lit), val, "{lit:#x}");
    }
}
