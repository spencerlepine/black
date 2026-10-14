use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B66B8, "218d4cf88b9c08a1"),
    (0x021B6708, "9b50c45f945b7ffd"),
    (0x021B6850, "4f51a6364a281db4"),
    (0x021B6904, "0ac94abd3c0a714b"),
    (0x021B6934, "846d9cb8e78d7727"),
    (0x021B69B0, "8bed1840562f22bb"),
    (0x021B69D8, "b66b980e2b720a2d"),
    (0x021B6A78, "5ab2438470a6001c"),
    (0x021B6AA4, "0033f12168f56983"),
    (0x021B6AAC, "9280e7dec0439b85"),
    (0x021B6AB0, "2719428b6fbba258"),
    (0x021B6AE0, "145202b4656350e1"),
    (0x021B6AFC, "ab5b7e5f84963210"),
    (0x021B6B6C, "77140500a7275268"),
    (0x021B6D0C, "1a4c6fd63fb88631"),
    (0x021B6D30, "e92ffde780d256f0"),
    (0x021B6D64, "73c4dfea93108a1e"),
    (0x021B6D98, "444a1526a1c335fe"),
    (0x021B6DC0, "1295b0945e4ed0a1"),
    (0x021B6DE4, "30694d0f51d4dd9e"),
    (0x021B6E0C, "1823de8a449e2771"),
    (0x021B6E24, "1af12d376907f911"),
    (0x021B6E68, "8be005379b12d0d0"),
    (0x021B6EB0, "0f6b075168531dca"),
    (0x021B6EE0, "ffb0a4e4ea2767f7"),
    (0x021B6F00, "516a1b2ed48615e1"),
    (0x021B6F18, "9b56ef64ebb8c090"),
    (0x021B6F5C, "48b7b80a179d9383"),
    (0x021B6F88, "81d39733d9b8d821"),
    (0x021B6FE8, "d60928b7853503d5"),
    (0x021B6FEC, "debf610d000ff115"),
    (0x021B6FF0, "6ad36426b899e0a6"),
    (0x021B7020, "4d5685a291071bb1"),
    (0x021B7038, "8d1747f8da552654"),
    (0x021B7068, "2db000a27e677b29"),
    (0x021B707C, "debf610d000ff115"),
    (0x021B7080, "4712a2ce9bf0fc3a"),
    (0x021B7098, "43a6087ae6e56935"),
    (0x021B709C, "e065939aabd66492"),
    (0x021B70D4, "2bb8b2bd9465cef5"),
    (0x021B70D8, "eb1c9c024ee4a956"),
    (0x021B716C, "bd1986ede1109e26"),
    (0x021B721C, "2d429b48977722c8"),
    (0x021B72CC, "8b08b63f8adeae11"),
    (0x021B7364, "099c5513ff7f7e5c"),
    (0x021B7390, "959d782480c74255"),
    (0x021B73B8, "865b2074372bbfdb"),
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
impl PokemonFldHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)])
    }
    fn fun_0215a5d0(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5d0, &mut [Arg::V(ctx)])
    }
    fn fun_0215a6c4(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a6c4, &mut [Arg::V(ctx)])
    }
    fn fun_0215a6d0(&mut self, ctx: u32, pc: u32) {
        self.stubs.call(0x0215a6d0, &mut [Arg::V(ctx), Arg::V(pc)]);
    }
    fn fun_0215a7f0(&mut self, ctx: u32, id: u32, x: u32, i: u32, a: u32, b: u32) {
        self.stubs.call(
            0x0215a7f0,
            &mut [
                Arg::V(ctx),
                Arg::V(id),
                Arg::V(x),
                Arg::V(i),
                Arg::V(a),
                Arg::V(b),
            ],
        );
    }
    fn fun_02158f54(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f54, &mut [Arg::V(sw)])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f58, &mut [Arg::V(sw)])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f74, &mut [Arg::V(sw)])
    }
    fn fun_02158f78(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f78, &mut [Arg::V(sw)])
    }
    fn fun_02158f7c(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f7c, &mut [Arg::V(sw)])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_020127a4(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x020127a4, &mut [Arg::V(gamedata)])
    }
    fn fun_02012844(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012844, &mut [Arg::V(gamedata)])
    }
    fn fun_0201292c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201292c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012934, &mut [Arg::V(gamedata)])
    }
    fn fun_0201293c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201293c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012944, &mut [Arg::V(gamedata)])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012ebc, &mut [Arg::V(gamedata)])
    }
    fn fun_02012f08(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f08, &mut [Arg::V(gamedata)])
    }
    fn fun_02008530(&mut self, status: u32) -> u32 {
        self.stubs.call(0x02008530, &mut [Arg::V(status)])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.stubs.call(0x0201aa18, &mut [Arg::V(party)])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.stubs.call(0x0201ab28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn fun_0201ab48(&mut self, party: u32, i: u32, mon: u32) {
        self.stubs
            .call(0x0201ab48, &mut [Arg::V(party), Arg::V(i), Arg::V(mon)]);
    }
    fn fun_0201ab8c(&mut self, party: u32, i: u32, a: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x0201ab8c,
            &mut [Arg::V(party), Arg::V(i), Arg::V(a), Arg::V(heap)],
        )
    }
    fn fun_0201a98c(&mut self, party: u32, mon: u32) {
        self.stubs
            .call(0x0201a98c, &mut [Arg::V(party), Arg::V(mon)]);
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)])
    }
    fn fun_02017e40(&mut self, mon: u32, id: u32, v: u32) {
        self.stubs
            .call(0x02017e40, &mut [Arg::V(mon), Arg::V(id), Arg::V(v)]);
    }
    fn fun_02017988(&mut self, mon: u32, form: u32) {
        self.stubs
            .call(0x02017988, &mut [Arg::V(mon), Arg::V(form)]);
    }
    fn fun_020185c4(&mut self, mon: u32) {
        self.stubs.call(0x020185c4, &mut [Arg::V(mon)]);
    }
    fn fun_020185f0(&mut self, mon: u32) -> u32 {
        self.stubs.call(0x020185f0, &mut [Arg::V(mon)])
    }
    fn fun_0201747c(&mut self, boxmon: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x0201747c, &mut [Arg::V(boxmon), Arg::V(heap)])
    }
    fn fun_02007698(&mut self, bx: u32) -> u32 {
        self.stubs.call(0x02007698, &mut [Arg::V(bx)])
    }
    fn fun_020079bc(&mut self, bx: u32, b: u32, s: u32, id: u32, x: u32) -> u32 {
        self.stubs.call(
            0x020079bc,
            &mut [Arg::V(bx), Arg::V(b), Arg::V(s), Arg::V(id), Arg::V(x)],
        )
    }
    fn fun_02007a18(&mut self, bx: u32, b: u32, s: u32) -> u32 {
        self.stubs
            .call(0x02007a18, &mut [Arg::V(bx), Arg::V(b), Arg::V(s)])
    }
    fn fun_020075b0(&mut self, bx: u32, b: u32, s: u32, boxmon: u32) {
        self.stubs.call(
            0x020075b0,
            &mut [Arg::V(bx), Arg::V(b), Arg::V(s), Arg::V(boxmon)],
        );
    }
    fn fun_02008fec(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02008fec, &mut [Arg::V(gamedata)])
    }
    fn fun_02008f08(&mut self, heap: u32) -> u32 {
        self.stubs.call(0x02008f08, &mut [Arg::V(heap)])
    }
    fn fun_02009020(&mut self, mailbox: u32, x: u32) -> u32 {
        self.stubs
            .call(0x02009020, &mut [Arg::V(mailbox), Arg::V(x)])
    }
    fn fun_0200903c(&mut self, mailbox: u32, x: u32, slot: u32, mail: u32) {
        self.stubs.call(
            0x0200903c,
            &mut [Arg::V(mailbox), Arg::V(x), Arg::V(slot), Arg::V(mail)],
        );
    }
    fn fun_02008eac(&mut self, mail: u32) {
        self.stubs.call(0x02008eac, &mut [Arg::V(mail)]);
    }
    fn fun_0202136c(&mut self, item: u32) -> u32 {
        self.stubs.call(0x0202136c, &mut [Arg::V(item)])
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
    fn fun_02188c84(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c84, &mut [Arg::V(fm)])
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c7c, &mut [Arg::V(fm)])
    }
    fn fun_02188d24(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188d24, &mut [Arg::V(fm)])
    }
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32 {
        self.stubs
            .call(0x021bb990, &mut [Arg::V(proc_id), Arg::V(x), Arg::V(table)])
    }
    fn fun_021854b0(&mut self, gamesys: u32, fm: u32, zone: u32, pos: &[u8; 12], x: u32) -> u32 {
        self.stubs.call(
            0x021854b0,
            &mut [
                Arg::V(gamesys),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(pos),
                Arg::V(x),
            ],
        )
    }
    fn fun_0200a8a0(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200a8a0, &mut [Arg::V(save)])
    }
    fn fun_0200a8ac(&mut self, s: u32) -> u32 {
        self.stubs.call(0x0200a8ac, &mut [Arg::V(s)])
    }
    fn fun_0200a8c8(&mut self, s: u32, i: u32) -> u32 {
        self.stubs.call(0x0200a8c8, &mut [Arg::V(s), Arg::V(i)])
    }
    fn fun_0200a8f8(&mut self, s: u32, i: u32) -> u32 {
        self.stubs.call(0x0200a8f8, &mut [Arg::V(s), Arg::V(i)])
    }
    fn fun_0200a940(&mut self, s: u32, month: u32, day: u32, x: u32) -> u32 {
        self.stubs.call(
            0x0200a940,
            &mut [Arg::V(s), Arg::V(month), Arg::V(day), Arg::V(x)],
        )
    }
    fn fun_021690a4(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x021690a4, &mut [Arg::V(gamedata)])
    }
    fn fun_021690d0(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x021690d0, &mut [Arg::V(gamedata)])
    }
    fn fun_021690e0(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x021690e0, &mut [Arg::V(gamedata)])
    }
    fn fun_02087c88(&mut self, date: &mut [u8; 8]) {
        self.stubs.call(0x02087c88, &mut [Arg::Out(&mut date[..])]);
    }
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32 {
        self.stubs.call(0x020457b0, &mut [Arg::V(n), Arg::V(heap)])
    }
    fn fun_02045808(&mut self, buf: u32) {
        self.stubs.call(0x02045808, &mut [Arg::V(buf)]);
    }
    fn fun_02045924(&mut self, buf: u32, src: u32) {
        self.stubs.call(0x02045924, &mut [Arg::V(buf), Arg::V(src)]);
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
    fn fun_0201eec4(&mut self, ws: u32, i: u32, x: u32) {
        self.stubs
            .call(0x0201eec4, &mut [Arg::V(ws), Arg::V(i), Arg::V(x)]);
    }
    fn fun_0201eed4(&mut self, ws: u32, i: u32, mon: u32) {
        self.stubs
            .call(0x0201eed4, &mut [Arg::V(ws), Arg::V(i), Arg::V(mon)]);
    }
    fn fun_0201ef40(&mut self, ws: u32, i: u32, item: u32) {
        self.stubs
            .call(0x0201ef40, &mut [Arg::V(ws), Arg::V(i), Arg::V(item)]);
    }
    fn fun_0201f02c(&mut self, ws: u32, i: u32, status: u32) {
        self.stubs
            .call(0x0201f02c, &mut [Arg::V(ws), Arg::V(i), Arg::V(status)]);
    }
    fn fun_0201f184(&mut self, ws: u32, i: u32, x: u32) {
        self.stubs
            .call(0x0201f184, &mut [Arg::V(ws), Arg::V(i), Arg::V(x)]);
    }
    fn fun_0204a298(
        &mut self,
        arc: u32,
        member: u32,
        x: u32,
        heap: u32,
        size: &mut [u8; 4],
    ) -> u32 {
        self.stubs.call(
            0x0204a298,
            &mut [
                Arg::V(arc),
                Arg::V(member),
                Arg::V(x),
                Arg::V(heap),
                Arg::Out(&mut size[..]),
            ],
        )
    }
    fn fun_02007f68(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x02007f68,
            &mut [Arg::V(bag), Arg::V(item), Arg::V(n), Arg::V(heap)],
        )
    }
    fn fun_02007e50(&mut self, bag: u32, item: u32, n: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x02007e50,
            &mut [Arg::V(bag), Arg::V(item), Arg::V(n), Arg::V(heap)],
        )
    }
    fn fun_0200c278(&mut self, x: u32, v: u32) {
        self.stubs.call(0x0200c278, &mut [Arg::V(x), Arg::V(v)]);
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0200ca1c, &mut [Arg::V(gamedata)])
    }
    fn fun_0200cde0(&mut self, dex: u32, mon: u32) {
        self.stubs.call(0x0200cde0, &mut [Arg::V(dex), Arg::V(mon)]);
    }
    fn fun_0200be48(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200be48, &mut [Arg::V(save)])
    }
    fn fun_0200bf18(&mut self, block: u32, v: u32) -> u32 {
        self.stubs.call(0x0200bf18, &mut [Arg::V(block), Arg::V(v)])
    }
    fn fun_02157e9c(&mut self, rec: u32, heap: u32, gamedata: u32) -> u32 {
        self.stubs.call(
            0x02157e9c,
            &mut [Arg::V(rec), Arg::V(heap), Arg::V(gamedata)],
        )
    }
    fn fun_0216d588(&mut self, obj: u32) -> u32 {
        self.stubs.call(0x0216d588, &mut [Arg::V(obj)])
    }
    fn fun_0216d598(&mut self, obj: u32) -> u32 {
        self.stubs.call(0x0216d598, &mut [Arg::V(obj)])
    }
    fn fun_02162740(&mut self, s: u32) -> u32 {
        self.stubs.call(0x02162740, &mut [Arg::V(s)])
    }
    fn fun_02162744(&mut self, s: u32) -> u32 {
        self.stubs.call(0x02162744, &mut [Arg::V(s)])
    }
    // the index pointer is a local of the caller (B18b: `ptr`, logged as PTR_LOCAL)
    fn fun_0216dd1c(&mut self, sys: u32, obj: &mut [u8; 4], _index: &mut [u8; 4]) -> u32 {
        self.stubs.call(
            0x0216_DD1C,
            &mut [
                Arg::V(sys),
                Arg::Out(&mut obj[..]),
                Arg::Ptr(black_fndiff::STACK_ZERO.start),
            ],
        )
    }
    fn fun_0201efa4(&mut self, ws: u32, i: u32, n: u32, digits: u32, a: u32, b: u32) {
        self.s(0x0201_EFA4, &[ws, i, n, digits, a, b]);
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address, the function's globals at `globals` and its
/// regions at 0x02F10000 + k * 0x10000; they are put back afterwards.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    enum Slot {
        Buf(usize),
        Global(usize),
        Region(usize),
    }
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push(Slot::Buf(i));
        }
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push(Slot::Global(k));
    }
    for (k, r) in c.regions.iter_mut().enumerate() {
        mem.add_window(0x02F1_0000 + 0x1_0000 * k as u32, std::mem::take(r));
        keep.push(Slot::Region(k));
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for (slot, (_, bytes)) in keep.into_iter().zip(wins) {
        match slot {
            Slot::Buf(i) => c.bufs[i] = bytes,
            Slot::Global(k) => c.globals[k] = bytes,
            Slot::Region(k) => c.regions[k] = bytes,
        }
    }
    r
}

fn run(addr: u32, c: &mut Case) -> u64 {
    with_rom(c, &[], |h, v| match addr {
        0x021B_6708 => {
            swap_with_box(h, v[0], v[1]);
            0
        }
        0x021B_6850 => find_in_box(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_6904 => {
            release_found(h, v[0]);
            0
        }
        0x021B_6934 => find_in_party(h, v[0]) as u64,
        0x021B_6AA4 => record_at(v[0], v[1]) as u64,
        0x021B_6AAC => {
            record_nop(v[0]);
            0
        }
        0x021B_6AB0 => record_count(h, v[0]) as u64,
        0x021B_6D0C => pdw_check(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_6D30 => pdw_name_a(h, v[0], v[1], v[2]) as u64,
        0x021B_6D64 => pdw_name_b(h, v[0], v[1], v[2]) as u64,
        0x021B_6D98 => pdw_give(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_6DC0 => pdw_kind(h, v[0], v[1], v[2], v[3]) as u64,
        0x021B_6DE4 => pdw_obj_or_record(h, v[0], v[1]) as u64,
        0x021B_6E0C => pdw_has_obj(h, v[0]) as u64,
        0x021B_6E24 => pdw_find_obj(h, v[0]) as u64,
        0x021B_6E68 => pdw_find_record(h, v[0]) as u64,
        0x021B_6EB0 => pdw_is_special(h, v[0], v[1]) as u64,
        0x021B_6EE0 => pdw_make_mon(h, v[0], v[1]) as u64,
        0x021B_6F00 => pdw_party_room(h, v[0], v[1]) as u64,
        0x021B_6F18 => {
            pdw_give_mon(h, v[0], v[1], v[2]);
            0
        }
        0x021B_6F5C => pdw_mon_kind(h, v[0], v[1], v[2]) as u64,
        0x021B_6F88 => pdw_mon_names(h, v[0], v[1], v[2]) as u64,
        0x021B_6FE8 => pdw_const_6() as u64,
        0x021B_6FEC | 0x021B_707C => pdw_const_1(addr | 1) as u64,
        0x021B_6FF0 => pdw_give_item(h, v[0], v[1], v[2]) as u64,
        0x021B_7020 => pdw_item_kind(h, v[0], v[1], v[2]) as u64,
        0x021B_7038 => pdw_item_names(h, v[0], v[1], v[2]) as u64,
        0x021B_7068 => pdw_item_name(h, v[0], v[1]) as u64,
        0x021B_7080 => pdw_give_3(h, v[0], v[1], v[2]) as u64,
        0x021B_7098 => pdw_const_5() as u64,
        0x021B_709C => pdw_names_3(h, v[0], v[1], v[2]) as u64,
        0x021B_70D4 => pdw_const_a() as u64,
        0x021B_7390 => is_special_day(v[0], v[1]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// Command 494 (Dream World, X2) is specced but neither ported nor dispatched.
#[test]
fn cmd_494_is_left_out() {
    assert!(!PORTED.contains(&CMD_494_DREAM_WORLD));
    assert!(COMMANDS.iter().all(|c| c.0 != 494));
    let spec =
        include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_pokemon_fld.spec");
    assert!(spec.contains("fn 021b6bdc "));
}

/// The embedded ROM tables and literals are the build's.
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
    for (k, &x) in TABLE_564.iter().enumerate() {
        assert_eq!(rd(TABLE_564_ADDR + 2 * k as u32, 2), u32::from(x));
    }
    for (k, &x) in SPECIAL_DAYS.iter().enumerate() {
        assert_eq!(rd(SPECIAL_DAYS_ADDR + 2 * k as u32, 2), u32::from(x));
    }
    for (k, &x) in DAY_TABLE.iter().enumerate() {
        assert_eq!(rd(DAY_TABLE_ADDR + k as u32, 1), u32::from(x));
    }
    for (k, &x) in ARC208_INDEX.iter().enumerate() {
        assert_eq!(rd(ARC208_INDEX_ADDR + k as u32, 1), u32::from(x));
    }
    for (r, row) in PDW_ROWS.iter().enumerate() {
        for (k, &x) in row.iter().enumerate() {
            assert_eq!(rd(PDW_ROWS_ADDR + 0x18 * r as u32 + 4 * k as u32, 4), x);
        }
    }
    assert_eq!(rd(0x021B_6D08, 4), PDW_ROWS_ADDR);
    assert_eq!(rd(0x021B_684C, 4), FILE);
    assert_eq!(rd(0x021B_6848, 4), 0x4C7);
    assert_eq!(rd(0x021B_6844, 4), SPECIES_581);
    assert_eq!((rd(0x021B_69D0, 4), rd(0x021B_69D4, 4)), PROC_308);
    assert_eq!(rd(0x021B_6A74, 4), STRBUF_471.1);
    assert_eq!(rd(0x021B_7168, 4), ARC208_INDEX_ADDR);
    assert_eq!(rd(0x021B_73FC, 4), DAY_TABLE_ADDR);
    assert_eq!(rd(0x021B_701C, 4), PDW_ITEM_MAX);
    assert_eq!((rd(0x021B_6ED8, 4) as u16, rd(0x021B_6EDC, 4)), PDW_SPECIAL);
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_pokemon_fld`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_pokemon_fld",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_pokemon_fld.spec"),
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
