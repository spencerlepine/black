use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AE948, "8fefcde34498767e"),
    (0x021AE994, "1b23ea7d09cccdf4"),
    (0x021AE9CC, "f216e18e4c8ee070"),
    (0x021AE9F0, "8954224c60fe8fa4"),
    (0x021AEA3C, "88f91701dbfd64b4"),
    (0x021AEA8C, "c940c7f295fab21d"),
    (0x021AEABC, "b91553ae2bcc73a8"),
    (0x021AEB3C, "b659ca1ffd7688f9"),
    (0x021AEBB0, "98c632fa69d7b9a3"),
    (0x021AEC1C, "0ae8c8c8661f2c1a"),
    (0x021AEC40, "ae362cbf4e53d0dc"),
    (0x021AEC84, "11d690c132c5f439"),
    (0x021AECC0, "f97daa2de9a27361"),
    (0x021AED00, "53d2f23b74d9d72e"),
    (0x021AED24, "7195484d08f651de"),
    (0x021AEDAC, "49daf6b288d7d3b2"),
    (0x021AEDE0, "e2a32126a8915b50"),
    (0x021AEE28, "4bbbbdb3aa6b3a5e"),
    (0x021AEE78, "c66e7716066e29fe"),
    (0x021AEEC0, "0e642e900674cf4e"),
    (0x021AEEE0, "88b810d018b8d54f"),
    (0x021AEF00, "5faca38a519f7e33"),
    (0x021AEF10, "d057d06604b8a9a7"),
    (0x021AEF60, "76cd1560f1be1cfb"),
    (0x021AEFA4, "712dca3063e7d6fc"),
    (0x021AEFD8, "66aa7f4cee16c82a"),
    (0x021AF044, "f8da3223cdca0fc5"),
    (0x021AF0BC, "fb83cd0c6a190b53"),
    (0x021AF0F8, "92f9b108b7f49e91"),
    (0x021AF130, "85312a8c5fe68efd"),
    (0x021AF174, "38f0c55ccf7e3777"),
    (0x021AF1B0, "6ab782dc9861ae71"),
    (0x021AF1D0, "4c42cadb934bac22"),
    (0x021AF1E8, "7ab6ebe94e5ef410"),
    (0x021AF21C, "717ec5548adca135"),
    (0x021AF23C, "da55186a902a6a8f"),
    (0x021AF26C, "a7e3970329f1c37a"),
    (0x021AF294, "5fac9f25136037b9"),
    (0x021AF2BC, "e8be821592660e38"),
    (0x021AF2CC, "39a04f6c28cdc73c"),
    (0x021AF2E4, "c44af348e5046974"),
    (0x021AF2F4, "e68e88390eeaa82e"),
    (0x021AF324, "93eda88dfb913c50"),
    (0x021AF370, "b4477cc12f820d87"),
    (0x021AF380, "8e4034d4dea1e725"),
    (0x021AF3D8, "3981fc7f750d9f6f"),
    (0x021AF430, "ab18e30734c3a9a0"),
    (0x021AF448, "5d545913de233682"),
    (0x021AF458, "1df4ec0012ddc184"),
    (0x021AF484, "bcf5e23b7f9a44df"),
    (0x021AF4B0, "8a9beed7d73fdcc9"),
    (0x021AF4E8, "b3ff3cf2e05d04e6"),
    (0x021AF544, "c813f7cca91bd9eb"),
    (0x021AF594, "55faf4b9b4da5558"),
    (0x021AF5E4, "e9fe4b7de55ca6e6"),
    (0x021AF634, "b6dc7518f0e875b4"),
    (0x021AF684, "95e37e7548c2fc43"),
    (0x021AF6C4, "7c7151560e51cdf5"),
    (0x021AF6E8, "12cf5d0dc336cb2f"),
    (0x021AF71C, "7056dc0bfa7f78e9"),
    (0x021AF750, "8da219c4fb5bf9e0"),
    (0x021AF75C, "3b2f18fff40f5e22"),
    (0x021AF794, "78f7b940572fb799"),
    (0x021AF7C4, "df7425d57cb05301"),
    (0x021AF7D4, "00a9ee6e9b6dc787"),
    (0x021AF7E4, "c196caca10a63faa"),
    (0x021AF844, "d3862a9e6f1d6458"),
    (0x021AF878, "ad7196800da39e6c"),
    (0x021AF8A8, "e3c83246e70a1206"),
    (0x021AF8D8, "c674707147b70da8"),
    (0x021AF908, "1065565a0d74e0d0"),
    (0x021AF918, "5bfd07d4a2d391bf"),
    (0x021AF928, "09dd91a7d3be5ddd"),
    (0x021AF958, "05dc2620a0112c0e"),
    (0x021AF9A4, "25711ba25199570a"),
    (0x021AF9F0, "23a73384c017990f"),
    (0x021AFA38, "b01ac9e1580d2e34"),
    (0x021AFA80, "3d32cb3fe354a578"),
    (0x021AFAC8, "c3133a09950fcb9b"),
    (0x021AFAFC, "068240788fd8675c"),
    (0x021AFB2C, "1eb15d5173ddf53d"),
    (0x021AFB5C, "5009ee7b233641fc"),
    (0x021AFB90, "903776109665f868"),
    (0x021AFBC4, "94a66a06f4b070c2"),
    (0x021AFBEC, "0a42de038da66664"),
    (0x021AFC18, "e8e5a67579af5bcb"),
    (0x021AFC28, "1cdff890160b32e7"),
    (0x021AFC38, "1c892137d17f431d"),
    (0x021AFC88, "cd3cf951faf8f457"),
    (0x021AFCD8, "147eff471d16da53"),
    (0x021AFD44, "017f91fa4b308338"),
    (0x021AFDAC, "91f5bc65395ad237"),
    (0x021AFE64, "bbf9f117a99f2a71"),
    (0x021AFECC, "28b318f82040a5b4"),
    (0x021AFEFC, "90b936315c05d09a"),
    (0x021AFF7C, "0bffae204189c4e6"),
    (0x021B0000, "1d96eea1a7fa3002"),
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
    fn a(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.stubs.call(addr, args)
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

impl FldHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5CC, &[ctx])
    }
    fn fun_02158f54(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F54, &[sw])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F58, &[sw])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F74, &[sw])
    }
    fn fun_02158f88(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F88, &[sw])
    }
    fn fun_02158f8c(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F8C, &[sw])
    }
    fn fun_02159124(&mut self, sw: u32, i: u32) -> u32 {
        self.s(0x0215_9124, &[sw, i])
    }
    fn fun_02159144(&mut self, bit: u32) {
        self.s(0x0215_9144, &[bit]);
    }
    fn fun_02159168(&mut self, bit: u32) {
        self.s(0x0215_9168, &[bit]);
    }
    fn fun_0215a460(&mut self, id: u32) -> u32 {
        self.s(0x0215_A460, &[id])
    }
    fn fun_0215a4a0(&mut self, id: u32) -> u32 {
        self.s(0x0215_A4A0, &[id])
    }
    fn fun_0215a4b4(&mut self, id: u32) -> u32 {
        self.s(0x0215_A4B4, &[id])
    }
    fn fun_0215a4c8(&mut self, id: u32) -> u32 {
        self.s(0x0215_A4C8, &[id])
    }
    fn fun_0215a6ac(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A6AC, &[ctx])
    }
    fn fun_0215a6b8(&mut self, ctx: u32, v: u32) {
        self.s(0x0215_A6B8, &[ctx, v]);
    }
    fn fun_0216d588(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D588, &[mmdl])
    }
    fn fun_0216e818(&mut self, gs: u32, fm: u32, a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
        self.s(0x0216_E818, &[gs, fm, a, b, c, d, e])
    }
    fn fun_0216eab0(
        &mut self,
        gs: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
    ) -> u32 {
        self.s(0x0216_EAB0, &[gs, fm, a, b, c, d, e, f])
    }
    fn fun_021ae430(&mut self, gs: u32, ev0: u32, ev1: u32) -> u32 {
        self.s(0x021A_E430, &[gs, ev0, ev1])
    }
    fn fun_021ae454(&mut self, heap: u32, fm: u32, rec: u32, n: u32) -> u32 {
        self.s(0x021A_E454, &[heap, fm, rec, n])
    }
    fn fun_021be92c(&mut self, gs: u32, fm: u32, a: u32, b: u32, c: u32, d: u32, e: u32) -> u32 {
        self.s(0x021B_E92C, &[gs, fm, a, b, c, d, e])
    }
    fn fun_02188c9c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C9C, &[fm])
    }
    fn fun_02188c5c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C5C, &[fm])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CB4, &[fm])
    }
    fn fun_02188cd0(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CD0, &[fm])
    }
    fn fun_02188d50(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D50, &[fm])
    }
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]) {
        self.a(0x021A_3038, &mut [Arg::V(player), Arg::Out(pos)]);
    }
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        self.s(0x021A_3074, &[player])
    }
    fn fun_021bb3e8(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32, loc: &mut [u8; 12], e: u32) {
        self.a(
            0x021B_B3E8,
            &mut [
                Arg::V(x),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::Out(loc),
                Arg::V(e),
            ],
        );
    }
    fn fun_021bb578(&mut self, x: u32, d: u32, a: u32, b: u32) {
        self.s(0x021B_B578, &[x, d, a, b]);
    }
    fn fun_021c34f8(&mut self, daycare: u32, i: u32) -> u32 {
        self.s(0x021C_34F8, &[daycare, i])
    }
    fn fun_02184d38(&mut self, gs: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32, e: u32) -> u32 {
        self.a(
            0x0218_4D38,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(pos),
                Arg::V(d),
                Arg::V(e),
            ],
        )
    }
    fn fun_02184d90(&mut self, gs: u32, fm: u32, zone: u32, loc: &[u8; 12], d: u32, e: u32) -> u32 {
        self.a(
            0x0218_4D90,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(loc),
                Arg::V(d),
                Arg::V(e),
            ],
        )
    }
    fn fun_02184de8(&mut self, gs: u32, fm: u32, zone: u32, loc: &[u8; 12], d: u32, e: u32) -> u32 {
        self.a(
            0x0218_4DE8,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(loc),
                Arg::V(d),
                Arg::V(e),
            ],
        )
    }
    fn fun_02184e40(&mut self, gs: u32, fm: u32, p2: &[u8; 12], zone: u32, pos: &[u8; 12]) -> u32 {
        self.a(
            0x0218_4E40,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::In(p2),
                Arg::V(zone),
                Arg::In(pos),
            ],
        )
    }
    fn fun_02185050(&mut self, gs: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32 {
        self.a(
            0x0218_5050,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(pos),
                Arg::V(d),
            ],
        )
    }
    fn fun_021850a8(&mut self, gs: u32, fm: u32) -> u32 {
        self.s(0x0218_50A8, &[gs, fm])
    }
    fn fun_021854b0(&mut self, gs: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32 {
        self.a(
            0x0218_54B0,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(pos),
                Arg::V(d),
            ],
        )
    }
    fn fun_02185534(&mut self, gs: u32, fm: u32, zone: u32, pos: &[u8; 12], d: u32) -> u32 {
        self.a(
            0x0218_5534,
            &mut [
                Arg::V(gs),
                Arg::V(fm),
                Arg::V(zone),
                Arg::In(pos),
                Arg::V(d),
            ],
        )
    }
    fn fun_0216282c(&mut self, x: u32, ew: u32, pos: &[u8; 12], n: u32) -> u32 {
        self.a(
            0x0216_282C,
            &mut [Arg::V(x), Arg::V(ew), Arg::In(pos), Arg::V(n)],
        )
    }
    fn fun_02162a6c(&mut self, entry: u32, p2: &mut [u8; 12]) {
        self.a(0x0216_2A6C, &mut [Arg::V(entry), Arg::Out(p2)]);
    }
    fn fun_02012104(&mut self, gs: u32) -> u32 {
        self.s(0x0201_2104, &[gs])
    }
    fn fun_0201210c(&mut self, gs: u32) -> u32 {
        self.s(0x0201_210C, &[gs])
    }
    fn fun_02012134(&mut self, gs: u32) -> u32 {
        self.s(0x0201_2134, &[gs])
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        self.s(0x0201_2374, &[event, child]);
    }
    fn fun_02012844(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2844, &[gd])
    }
    fn fun_0201292c(&mut self, gd: u32) -> u32 {
        self.s(0x0201_292C, &[gd])
    }
    fn fun_02012934(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2934, &[gd])
    }
    fn fun_02012944(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2944, &[gd])
    }
    fn fun_02012984(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2984, &[gd])
    }
    fn fun_02012ebc(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2EBC, &[gd])
    }
    fn fun_02012f2c(&mut self, gd: u32) -> u32 {
        self.s(0x0201_2F2C, &[gd])
    }
    fn fun_02008df0(&mut self, x: u32, v: u32) {
        self.s(0x0200_8DF0, &[x, v]);
    }
    fn fun_02014024(&mut self, x: u32) -> u32 {
        self.s(0x0201_4024, &[x])
    }
    fn fun_02013e4c(&mut self, v: u32) -> u32 {
        self.s(0x0201_3E4C, &[v])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.s(0x0201_AB28, &[party, i])
    }
    fn fun_0200c0f0(&mut self, x: u32) -> u32 {
        self.s(0x0200_C0F0, &[x])
    }
    fn fun_0200c1c4(&mut self, x: u32) -> u32 {
        self.s(0x0200_C1C4, &[x])
    }
    fn fun_0200c1dc(&mut self, x: u32) -> u32 {
        self.s(0x0200_C1DC, &[x])
    }
    fn fun_02028a58(&mut self, x: u32, a: u32, b: u32) {
        self.s(0x0202_8A58, &[x, a, b]);
    }
    fn fun_020289b8(&mut self, x: u32) -> u32 {
        self.s(0x0202_89B8, &[x])
    }
    fn fun_02028ba0(&mut self, x: u32, v: u32) {
        self.s(0x0202_8BA0, &[x, v]);
    }
    fn fun_02028bdc(&mut self, x: u32, v: u32) {
        self.s(0x0202_8BDC, &[x, v]);
    }
    fn fun_020295a4(&mut self, gs: u32, v: u32) -> u32 {
        self.s(0x0202_95A4, &[gs, v])
    }
    fn fun_02029620(&mut self, gs: u32, a: u32, b: u32) -> u32 {
        self.s(0x0202_9620, &[gs, a, b])
    }
    fn fun_02029830(&mut self, gs: u32, v: u32) -> u32 {
        self.s(0x0202_9830, &[gs, v])
    }
    fn fun_020298a0(&mut self, gs: u32, x: u32, a: u32, b: u32) -> u32 {
        self.s(0x0202_98A0, &[gs, x, a, b])
    }
    fn fun_02029984(&mut self, gs: u32, v: u32) -> u32 {
        self.s(0x0202_9984, &[gs, v])
    }
    fn fun_02029a04(&mut self, gs: u32, a: u32, b: u32) -> u32 {
        self.s(0x0202_9A04, &[gs, a, b])
    }
    fn fun_02029a74(&mut self, gs: u32, v: u32) -> u32 {
        self.s(0x0202_9A74, &[gs, v])
    }
    fn fun_02029af4(&mut self, gs: u32, v: u32) -> u32 {
        self.s(0x0202_9AF4, &[gs, v])
    }
    fn fun_02029df4(&mut self, gd: u32, a: u32, b: u32) -> u32 {
        self.s(0x0202_9DF4, &[gd, a, b])
    }
    fn fun_02029e20(&mut self, v: u32) -> u32 {
        self.s(0x0202_9E20, &[v])
    }
    fn fun_0202a230(&mut self, v: u32, k: u32) -> u32 {
        self.s(0x0202_A230, &[v, k])
    }
    fn fun_0202b578(&mut self, x: u32) -> u32 {
        self.s(0x0202_B578, &[x])
    }
    fn fun_0202c0d0(&mut self, x: u32, v: u32) {
        self.s(0x0202_C0D0, &[x, v]);
    }
    fn fun_0202c0e4(&mut self, x: u32, v: u32) {
        self.s(0x0202_C0E4, &[x, v]);
    }
    fn fun_0202c0f8(&mut self, x: u32, v: u32) -> u32 {
        self.s(0x0202_C0F8, &[x, v])
    }
    fn fun_02005c64(&mut self) -> u32 {
        self.s(0x0200_5C64, &[])
    }
    fn fun_02005f38(&mut self) -> u32 {
        self.s(0x0200_5F38, &[])
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.s(0x0200_61E4, &[se]);
    }
    fn fun_020061f8(&mut self, i: u32) {
        self.s(0x0200_61F8, &[i]);
    }
    fn fun_02006148(&mut self, x: u32) -> u32 {
        self.s(0x0200_6148, &[x])
    }
    fn fun_02006224(&mut self, i: u32) -> u32 {
        self.s(0x0200_6224, &[i])
    }
    fn fun_02006984(
        &mut self,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
        _tmp: Option<&[u8; 4]>,
    ) -> u32 {
        self.a(
            0x0200_6984,
            &mut [
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
                Arg::V(f),
                Arg::V(g),
                Arg::P,
            ],
        )
    }
    fn fun_02006bd0(&mut self, handle: u32) -> u32 {
        self.s(0x0200_6BD0, &[handle])
    }
    fn fun_02006cac(&mut self, tmp: &mut [u8; 4]) {
        self.a(0x0200_6CAC, &mut [Arg::Out(tmp)]);
    }
    fn fun_02007e3c(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0200_7E3C, &[x, a, b, heap])
    }
    fn fun_02007e50(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0200_7E50, &[x, a, b, heap])
    }
    fn fun_02007f1c(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0200_7F1C, &[x, a, b, heap])
    }
    fn fun_02007f68(&mut self, x: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0200_7F68, &[x, a, b, heap])
    }
    fn fun_02007fb8(&mut self, x: u32, a: u32, heap: u32) -> u32 {
        self.s(0x0200_7FB8, &[x, a, heap])
    }
    fn fun_02020fe4(&mut self, item: u32, field: u32, heap: u32) -> u32 {
        self.s(0x0202_0FE4, &[item, field, heap])
    }
    fn fun_02021288(&mut self, x: u32) -> u32 {
        self.s(0x0202_1288, &[x])
    }
    fn fun_020212b0(&mut self, x: u32) -> u32 {
        self.s(0x0202_12B0, &[x])
    }
    fn fun_0201ee84(&mut self, ws: u32, idx: u32, mon: u32) {
        self.s(0x0201_EE84, &[ws, idx, mon]);
    }
    fn fun_0201eef4(&mut self, ws: u32, idx: u32, mon: u32) {
        self.s(0x0201_EEF4, &[ws, idx, mon]);
    }
    fn fun_0201ef34(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_EF34, &[ws, idx, v]);
    }
    fn fun_0201ef40(&mut self, ws: u32, idx: u32, v: u32) -> u32 {
        self.s(0x0201_EF40, &[ws, idx, v])
    }
    fn fun_0201ef4c(&mut self, ws: u32, idx: u32, v: u32) -> u32 {
        self.s(0x0201_EF4C, &[ws, idx, v])
    }
    fn fun_0201ef5c(&mut self, ws: u32, idx: u32, v: u32) -> u32 {
        self.s(0x0201_EF5C, &[ws, idx, v])
    }
    fn fun_0201efa4(&mut self, ws: u32, idx: u32, v: u32, n: u32, a: u32, b: u32) {
        self.s(0x0201_EFA4, &[ws, idx, v, n, a, b]);
    }
    fn fun_0201efc8(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_EFC8, &[ws, idx, v]);
    }
    fn fun_0201f020(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F020, &[ws, idx, v]);
    }
    fn fun_0201f02c(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F02C, &[ws, idx, v]);
    }
    fn fun_0201f048(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F048, &[ws, idx, v]);
    }
    fn fun_0201f14c(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F14C, &[ws, idx, v]);
    }
    fn fun_0201f158(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F158, &[ws, idx, v]);
    }
    fn fun_0201f184(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F184, &[ws, idx, v]);
    }
    fn fun_0201f198(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F198, &[ws, idx, v]);
    }
    fn fun_0201f1b0(&mut self, ws: u32, idx: u32, v: u32) {
        self.s(0x0201_F1B0, &[ws, idx, v]);
    }
    fn call_wordset_fn(&mut self, f: u32, ws: u32, idx: u32, v: u32) {
        // The spec stubs the targets the thunks load with the arguments it logs: 0201EE60 / 0201EEC4 / 0201F0F8 / 0201F124
        // log the WORDSET only; 0201EF40 / 0201EF4C and the fake callback 0x02EE0011 of the bodies' own blocks all three.
        let a = f & !1;
        match a {
            0x0201_EE60 | 0x0201_EEC4 | 0x0201_F0F8 | 0x0201_F124 => {
                self.s(a, &[ws]);
            }
            _ => {
                self.s(a, &[ws, idx, v]);
            }
        }
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
    with_rom(c, |h, v| {
        // the handlers by address, then the functions of other shapes
        if COMMANDS.iter().any(|c| c.1 == addr) {
            return dispatch(h, addr, v[0], v[1]).unwrap() as u64;
        }
        match addr {
            0x021A_EDAC => {
                after_battle_start(h, v[0]);
                0
            }
            0x021A_EEE0 => wait_160(h, v[0], v[1]) as u64,
            0x021A_EFA4 => init_proc_12(h, v[0], v[1]) as u64,
            0x021A_F174 => init_proc_10(h, v[0], v[1]) as u64,
            0x021A_F1D0 => init_proc_13(h, v[0], v[1]) as u64,
            0x021A_F1E8 => {
                sound_flags_update(h, v[0]);
                0
            }
            0x021A_F21C => {
                sound_flags_add(h, v[0], v[1]);
                0
            }
            0x021A_F23C => sound_flags_busy(h, v[0]) as u64,
            0x021A_F26C => {
                sound_flags_stop(h, v[0]);
                0
            }
            0x021A_F2CC => wait_168(h, v[0], v[1]) as u64,
            0x021A_F324 => wait_170(h, v[0], v[1]) as u64,
            0x021A_F430 => wait_172(h, v[0], v[1]) as u64,
            0x021A_F4E8 => event_171(h, v[0], v[1], v[2]) as u64,
            0x021A_F750 => item_word_021af750(h, v[0]) as u64,
            0x021A_F794 => wordset_body_77(h, v[0], v[1], v[2]) as u64,
            0x021A_F8D8 => wordset_body_87(h, v[0], v[1], v[2]) as u64,
            0x021A_FBEC => wordset_body_97(h, v[0], v[1], v[2]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_fld`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_fld",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_fld.spec"),
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
/// dispatched, and the ids are unique.
#[test]
fn commands_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0, "ids not sorted / unique at {}", w[1].0);
    }
    assert_eq!(COMMANDS.len(), 80);
    let Some(ovy) = overlay10() else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}
