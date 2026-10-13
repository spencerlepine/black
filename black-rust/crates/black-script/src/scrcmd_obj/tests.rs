use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B1668, "63cfde56f815f7bf"),
    (0x021B1698, "559364a07f7e0db6"),
    (0x021B16DC, "70c3acd0b842fa0c"),
    (0x021B16F0, "e8ee006c0ddfce81"),
    (0x021B1700, "bf4708722c73ef11"),
    (0x021B1738, "666918f7fd57ce92"),
    (0x021B176C, "8ea9ed5c55d1e842"),
    (0x021B17B4, "7c5bb3167f3e52c4"),
    (0x021B17F8, "d82d69911990e25d"),
    (0x021B181C, "a66db36cbf07ca6e"),
    (0x021B18B8, "137b2c7239013dd9"),
    (0x021B18E4, "7e18ef6eef45ec67"),
    (0x021B1950, "83b672e76f4dffb9"),
    (0x021B19B0, "1e1f65106866e65c"),
    (0x021B19E0, "3fe7d2bc945f6274"),
    (0x021B1A20, "067fe11a4be2a7ca"),
    (0x021B1A44, "9f9a59d55336f2a3"),
    (0x021B1B08, "466334e82c143eb8"),
    (0x021B1BB8, "d7a32b3250b2efd7"),
    (0x021B1BE8, "7d7a4a6cca962c11"),
    (0x021B1BF4, "0568f011a37ba0ee"),
    (0x021B1C2C, "107f595d97efa5c1"),
    (0x021B1CB0, "f49f5a81739c722f"),
    (0x021B1CFC, "37f0070d809bc6d0"),
    (0x021B1D20, "0bd4fe63059e7fe9"),
    (0x021B1D64, "26c66cd454dff53e"),
    (0x021B1DEC, "95821867dc894684"),
    (0x021B1E74, "46872edb6315e03d"),
    (0x021B1E80, "7ab754819c8ecb1e"),
    (0x021B1F60, "0c248ff0a649875f"),
    (0x021B1FB8, "5babe9d4463d2cfa"),
    (0x021B2040, "4ecaca25384938c3"),
    (0x021B2070, "d73d93756febb18b"),
    (0x021B20B0, "a1e1cae91757a04e"),
    (0x021B2104, "0c74e198443fd4ac"),
    (0x021B2154, "5da0f3fc23dfc521"),
    (0x021B21AC, "847f71195bf975de"),
    (0x021B2218, "6d1b15793a916bc4"),
    (0x021B229C, "d08e96c9668df227"),
    (0x021B2380, "59ddad85102d4320"),
    (0x021B23B8, "b1a36cbde437053c"),
    (0x021B23E0, "6ea17e59d58f22f9"),
    (0x021B23F0, "3131379471ffc0f4"),
    (0x021B240C, "887d7b5010a082b0"),
    (0x021B2444, "d1b46f884c74d0ce"),
    (0x021B247C, "0c308a169eaf6188"),
    (0x021B24B8, "164946a0c51ba30c"),
    (0x021B24F4, "6edbf2ddba88c3cc"),
    (0x021B252C, "678cf18879c3575b"),
    (0x021B2568, "da89d599ba637914"),
    (0x021B25A4, "0bf1ade896b2334e"),
    (0x021B25DC, "63b18d24c130b023"),
    (0x021B2614, "fb10021234a45266"),
    (0x021B2650, "cbd0a6b5a9347eac"),
    (0x021B268C, "1432dfbe9843cdb5"),
    (0x021B269C, "c9836dba51cfa198"),
    (0x021B26AC, "8f7ed8ff65474bcf"),
    (0x021B26E4, "4385f500346c5bff"),
    (0x021B271C, "b273019edbd28f92"),
    (0x021B273C, "64a48a74a150a3f9"),
    (0x021B277C, "c1de67d2329ac183"),
    (0x021B27B4, "2fa15299e28605eb"),
    (0x021B27F8, "3b9fefa02ee2b76b"),
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

impl ObjHooks for Rom<'_> {
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5EC, &[ctx])
    }
    fn fun_0215a6dc(&mut self, ctx: u32, m: u32) {
        self.s(0x0215_A6DC, &[ctx, m]);
    }
    fn fun_0215a6fc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A6FC, &[ctx])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F58, &[sw])
    }
    fn fun_02158f90(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F90, &[sw])
    }
    fn fun_0216ce6c(
        &mut self,
        mmdls: u32,
        x: u32,
        z: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
        g: u32,
    ) -> u32 {
        self.s(0x0216_CE6C, &[mmdls, x, z, c, d, e, f, g])
    }
    fn fun_0216cee4(&mut self, mmdls: u32, a: u32, zone: u32, data: u32, ew: u32, id: u32) -> u32 {
        self.s(0x0216_CEE4, &[mmdls, a, zone, data, ew, id])
    }
    fn fun_0216cf2c(&mut self, mmdl: u32) {
        self.s(0x0216_CF2C, &[mmdl]);
    }
    fn fun_0216d454(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D454, &[mmdl])
    }
    fn fun_0216d588(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D588, &[mmdl])
    }
    fn fun_0216d5a0(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D5A0, &[mmdl])
    }
    fn fun_0216d5b0(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D5B0, &[mmdl])
    }
    fn fun_0216d5e0(&mut self, mmdl: u32, d: u32) {
        self.s(0x0216_D5E0, &[mmdl, d]);
    }
    fn fun_0216d5f8(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D5F8, &[mmdl])
    }
    fn fun_0216d63c(&mut self, mmdl: u32, n: u32) -> u32 {
        self.s(0x0216_D63C, &[mmdl, n])
    }
    fn fun_0216d674(&mut self, mmdl: u32, v: u32) {
        self.s(0x0216_D674, &[mmdl, v]);
    }
    fn fun_0216d83c(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D83C, &[mmdl])
    }
    fn fun_0216d86c(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D86C, &[mmdl])
    }
    fn fun_0216d88c(&mut self, mmdl: u32, pos: &mut [u8; 12]) {
        self.a(0x0216_D88C, &mut [Arg::V(mmdl), Arg::Out(pos)]);
    }
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]) {
        self.a(0x0216_D89C, &mut [Arg::V(mmdl), Arg::In(pos)]);
    }
    fn fun_0216d8c0(&mut self, mmdl: u32, v: &[u8; 12]) {
        self.a(0x0216_D8C0, &mut [Arg::V(mmdl), Arg::In(v)]);
    }
    fn fun_0216d9a0(&mut self, mmdls: u32) {
        self.s(0x0216_D9A0, &[mmdls]);
    }
    fn fun_0216d9e4(&mut self, mmdls: u32) -> u32 {
        self.s(0x0216_D9E4, &[mmdls])
    }
    fn fun_0216da5c(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_DA5C, &[mmdl])
    }
    fn fun_0216daf8(&mut self, mmdl: u32) {
        self.s(0x0216_DAF8, &[mmdl]);
    }
    fn fun_0216db04(&mut self, mmdl: u32) {
        self.s(0x0216_DB04, &[mmdl]);
    }
    fn fun_0216db80(&mut self, mmdl: u32, v: u32) {
        self.s(0x0216_DB80, &[mmdl, v]);
    }
    fn fun_0216dddc(&mut self, x: u32, gx: u32, gz: u32, y: u32, a: u32, b: u32) -> u32 {
        self.s(0x0216_DDDC, &[x, gx, gz, y, a, b])
    }
    fn fun_0216de54(&mut self, mmdls: u32, id: u32) -> u32 {
        self.s(0x0216_DE54, &[mmdls, id])
    }
    fn fun_0216de94(&mut self, mmdls: u32, id: u32) -> u32 {
        self.s(0x0216_DE94, &[mmdls, id])
    }
    fn fun_0216dee0(&mut self, mmdls: u32) -> u32 {
        self.s(0x0216_DEE0, &[mmdls])
    }
    fn fun_0216e084(&mut self, mmdl: u32, x: u32, y: u32, z: u32, d: u32) {
        self.s(0x0216_E084, &[mmdl, x, y, z, d]);
    }
    fn fun_0216e0e8(&mut self, mmdl: u32, v: u32) {
        self.s(0x0216_E0E8, &[mmdl, v]);
    }
    fn fun_0216e3d0(&mut self, ew: u32, v: u32) -> u32 {
        self.s(0x0216_E3D0, &[ew, v])
    }
    fn fun_021618a0(&mut self, gamesys: u32, fm: u32, a: u32, rec: &[u8; 8]) -> u32 {
        self.a(
            0x0216_18A0,
            &mut [Arg::V(gamesys), Arg::V(fm), Arg::V(a), Arg::In(rec)],
        )
    }
    fn fun_02161ac0(&mut self, gamesys: u32, mmdl: u32, from: &[u8; 12], to: &[u8; 12]) -> u32 {
        self.a(
            0x0216_1AC0,
            &mut [Arg::V(gamesys), Arg::V(mmdl), Arg::In(from), Arg::In(to)],
        )
    }
    fn fun_02162740(&mut self, x: u32) -> u32 {
        self.s(0x0216_2740, &[x])
    }
    fn fun_02162744(&mut self, x: u32) -> u32 {
        self.s(0x0216_2744, &[x])
    }
    fn fun_021627c8(&mut self, x: u32, ew: u32, pos: &[u8; 12], dir: u32) -> u32 {
        self.a(
            0x0216_27C8,
            &mut [Arg::V(x), Arg::V(ew), Arg::In(pos), Arg::V(dir)],
        )
    }
    fn fun_02163e58(&mut self, mmdl: u32, pos: &[u8; 12], h: &mut [u8; 4]) {
        self.a(0x0216_3E58, &mut [Arg::V(mmdl), Arg::In(pos), Arg::Out(h)]);
    }
    fn fun_021642a4(&mut self, d: u32) -> u32 {
        self.s(0x0216_42A4, &[d])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_020127a4(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_27A4, &[gamedata])
    }
    fn fun_02012844(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2844, &[gamedata])
    }
    fn fun_02012ac8(&mut self, x: u32) -> u32 {
        self.s(0x0201_2AC8, &[x])
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.s(0x0200_61E4, &[se]);
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C7C, &[fm])
    }
    fn fun_02188c9c(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C9C, &[fm])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CB4, &[fm])
    }
    fn fun_02188d38(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D38, &[fm])
    }
    fn fun_02197d3c(&mut self, mmdl: u32, list: u32) -> u32 {
        self.s(0x0219_7D3C, &[mmdl, list])
    }
    fn fun_0219bba4(&mut self, mmdl: u32) -> u32 {
        self.s(0x0219_BBA4, &[mmdl])
    }
    fn fun_0219e2c0(&mut self, mmdl: u32, t: &[u8; 8]) {
        self.a(0x0219_E2C0, &mut [Arg::V(mmdl), Arg::In(t)]);
    }
    fn fun_0219e348(&mut self, mmdl: u32) -> u32 {
        self.s(0x0219_E348, &[mmdl])
    }
    fn fun_021a2f48(&mut self, player: u32, v: u32) {
        self.s(0x021A_2F48, &[player, v]);
    }
    fn fun_021a2f54(&mut self, player: u32) -> u32 {
        self.s(0x021A_2F54, &[player])
    }
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]) {
        self.a(0x021A_3038, &mut [Arg::V(player), Arg::Out(pos)]);
    }
    fn fun_021a3050(&mut self, player: u32) -> u32 {
        self.s(0x021A_3050, &[player])
    }
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        self.s(0x021A_3074, &[player])
    }
    fn fun_021a3098(&mut self, player: u32) -> u32 {
        self.s(0x021A_3098, &[player])
    }
    fn fun_021a3238(&mut self, player: u32) -> u32 {
        self.s(0x021A_3238, &[player])
    }
    fn fun_021a32a8(&mut self, player: u32, v: u32) -> u32 {
        self.s(0x021A_32A8, &[player, v])
    }
    fn fun_021b7dc0(&mut self, x: u32, t: &mut [u8; 8]) {
        self.a(0x021B_7DC0, &mut [Arg::V(x), Arg::Out(t)]);
    }
    fn fun_021b83fc(&mut self, x: u32, v1: u32, v2: u32, v3: u32) {
        self.s(0x021B_83FC, &[x, v1, v2, v3]);
    }
    fn fun_021b8484(&mut self, x: u32, a: u32, b: u32, c: u32) {
        self.s(0x021B_8484, &[x, a, b, c]);
    }
    fn fun_021c474c(&mut self, gamesys: u32, fm: u32, mmdl: u32) -> u32 {
        self.s(0x021C_474C, &[gamesys, fm, mmdl])
    }
    fn fun_021c6de0(&mut self, tasks: u32, task: u32, a: u32) {
        self.s(0x021C_6DE0, &[tasks, task, a]);
    }
    fn fun_021c7220(&mut self, fm: u32, a: u32, b: u32, mmdl: u32) -> u32 {
        self.s(0x021C_7220, &[fm, a, b, mmdl])
    }
    fn fun_021c7684(&mut self, fm: u32, mmdl: u32, a: u32, b: u32) -> u32 {
        self.s(0x021C_7684, &[fm, mmdl, a, b])
    }
    fn fun_021c7878(&mut self, fm: u32, a: u32, v: &[u8; 12], mmdl: u32) -> u32 {
        self.a(
            0x021C_7878,
            &mut [Arg::V(fm), Arg::V(a), Arg::In(v), Arg::V(mmdl)],
        )
    }
    fn fun_021c78dc(&mut self, fm: u32, a: u32, v: &[u8; 12], mmdl: u32) -> u32 {
        self.a(
            0x021C_78DC,
            &mut [Arg::V(fm), Arg::V(a), Arg::In(v), Arg::V(mmdl)],
        )
    }
    fn fun_021f3770(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3770, &[gamesys, v])
    }
    fn fun_021f37a4(&mut self, fm: u32) {
        self.s(0x021F_37A4, &[fm]);
    }
    fn fun_021f37c8(&mut self, fm: u32) -> u32 {
        self.s(0x021F_37C8, &[fm])
    }
    fn fun_021f37ec(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_37EC, &[gamesys, v])
    }
    fn fun_021f3834(&mut self, gamesys: u32, v: u32, flag: u32) -> u32 {
        self.s(0x021F_3834, &[gamesys, v, flag])
    }
    fn fun_021f384c(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_384C, &[gamesys, v])
    }
    fn fun_021f3888(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3888, &[gamesys, v])
    }
    fn fun_021f38c4(&mut self, gamesys: u32, v: u32) {
        self.s(0x021F_38C4, &[gamesys, v]);
    }
    fn fun_021f3970(&mut self, gamesys: u32) {
        self.s(0x021F_3970, &[gamesys]);
    }
    fn fun_021f398c(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_398C, &[gamesys, v])
    }
    fn fun_021f3a28(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3A28, &[gamesys, v])
    }
    fn fun_021f3a88(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        self.s(0x021F_3A88, &[gamesys, a, b])
    }
    fn fun_021f3ad0(&mut self, gamesys: u32, a: u32, b: u32) -> u32 {
        self.s(0x021F_3AD0, &[gamesys, a, b])
    }
    fn fun_021f3b18(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3B18, &[gamesys, v])
    }
    fn fun_021f3b5c(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3B5C, &[gamesys, v])
    }
    fn fun_021f3d74(&mut self, gamesys: u32) {
        self.s(0x021F_3D74, &[gamesys]);
    }
    fn fun_021f3e24(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_3E24, &[gamesys, v])
    }
    fn fun_021f4134(&mut self, gamesys: u32, v: u32) {
        self.s(0x021F_4134, &[gamesys, v]);
    }
    fn fun_021f4184(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_4184, &[gamesys, v])
    }
    fn fun_021f41b0(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_41B0, &[gamesys, v])
    }
    fn fun_021f41dc(&mut self, gamesys: u32, v: u32) -> u32 {
        self.s(0x021F_41DC, &[gamesys, v])
    }
    fn fun_021f42a8(&mut self, gamesys: u32) {
        self.s(0x021F_42A8, &[gamesys]);
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
    let globals: &[u32] = match addr {
        0x021B_1A44 | 0x021B_1B08 | 0x021B_1BB8 => &[LOCK_FLAGS],
        _ => &[],
    };
    with_rom(c, globals, |h, v| {
        if COMMANDS.iter().any(|c| c.1 == addr) {
            return dispatch(h, addr, v[0], v[1]).unwrap() as u64;
        }
        match addr {
            // the spec never generates 0xF1 here (the original returns its caller's r6)
            0x021B_1668 => object_of(h, v[0], v[1], 0) as u64,
            0x021B_16DC => wait_movements(h, v[0], v[1]) as u64,
            0x021B_1A44 => wait_lock(h, v[0], v[1]) as u64,
            0x021B_1B08 => talk_lock(h, v[0], v[1]) as u64,
            0x021B_1BB8 => cmd_46_body(h, v[0], v[1]) as u64,
            0x021B_1BE8 => release_objects(h, v[0]) as u64,
            0x021B_1CFC => wait_117(h, v[0], v[1]) as u64,
            0x021B_1E74 => player_mmdl(h, v[0]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_obj`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_obj",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_obj.spec"),
            PORTED,
            RECORDED,
        ),
        &run,
    );
}

/// The ROM build's file `name` (`$BLACK_ROM_BUILD/<name>`), or `None` (skip) when it is missing and `REQUIRE_ROM` is not
/// set.
fn rom_file(name: &str) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/{name}")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/{name} is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C), and the ids are unique.
#[test]
fn commands_match_rom() {
    for w in COMMANDS.windows(2) {
        assert!(w[0].0 < w[1].0, "ids not sorted / unique at {}", w[1].0);
    }
    assert_eq!(COMMANDS.len(), 55);
    let Some(ovy) = rom_file("OVY_10.sbin") else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}

/// The embedded overlay-21 tables of commands 119 / 583 are the ROM's (`OVY_21.sbin`, loaded at 0x02187F00).
#[test]
fn tables_match_rom() {
    let Some(ovy) = rom_file("OVY_21.sbin") else {
        return;
    };
    let at = |a: u32, n: usize| {
        let o = (a - 0x0218_7F00) as usize;
        ovy[o..o + n].to_vec()
    };
    let dirs: Vec<u16> = at(DIRS_119_ADDR, 8)
        .chunks(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    assert_eq!(dirs, DIRS_119);
    for (d, &a) in RECORDS_119_ADDR.iter().enumerate() {
        assert_eq!(at(a, 8), RECORDS_119[d], "direction {d}");
    }
    assert_eq!(at(VEC_583_ADDR, 12), VEC_583);
}
