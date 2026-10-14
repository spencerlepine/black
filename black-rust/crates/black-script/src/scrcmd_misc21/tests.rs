use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CF7DC, "60c894eb3b0eb870"),
    (0x021CF844, "fb3005ee3e014c87"),
    (0x021CF8E4, "bcc7fe368df9a399"),
    (0x021CF940, "59208285d8afd8db"),
    (0x021CF9AC, "949d536566b96558"),
    (0x021CFA04, "a38d2b70f5f44d9f"),
    (0x021CFA1C, "97aeccb69611967c"),
    (0x021CFA58, "30b7ba19f4738f64"),
    (0x021CFA88, "131c2a074dd7f6d4"),
    (0x021CFAC0, "a83f7781c3b1e4e8"),
    (0x021CFB1C, "10bbf18b7d625047"),
    (0x021CFB68, "d1768dd22f31722b"),
    (0x021CFBA8, "bff41902c68340a6"),
    (0x021CFC14, "849101fc6e52db43"),
    (0x021CFC4C, "48d67f06545912cb"),
    (0x021CFC80, "8be75c45f6125a16"),
    (0x021CFCF4, "eb812d1bb178947a"),
    (0x021CFD08, "74f6bf14e11b7cfb"),
    (0x021CFD88, "2ed4e3a15afbdb38"),
    (0x021CFDBC, "6122af7965cb5ba1"),
    (0x021CFEA0, "89200fdfb492489a"),
    (0x021CFEF0, "dc61627d87112371"),
    (0x021CFF28, "b9b13c386b9961fa"),
    (0x021CFF78, "017d1e3a85854a2d"),
    (0x021CFFDC, "4a20c6f91de44c38"),
    (0x021D0014, "a5e0645131b00396"),
    (0x021D004C, "f078e9bb7c000d36"),
    (0x021D0084, "1710481d5dd1e428"),
    (0x021D00B8, "e7f1f059748a2581"),
    (0x021D0114, "1eb706ece983e75e"),
    (0x021D1670, "b84aad03075db956"),
    (0x021D16D8, "4b30ae4641e958ee"),
    (0x021D179C, "97917992db43c9f3"),
    (0x021D182C, "8a2add8857926283"),
    (0x021D1884, "278dccf250269ec5"),
    (0x021D1894, "45bd05391aca21a4"),
    (0x021D18E0, "7cc485c3134cf822"),
    (0x021D18F4, "ddf61a84e80df3bb"),
    (0x021D1984, "9280e7dec0439b85"),
    (0x021D1988, "3470fc4513a836f9"),
    (0x021D1A04, "7bc6bb3c489c12d2"),
    (0x021D1A54, "799d47d83e312a02"),
    (0x021D1A74, "f5b7c36fe583af41"),
    (0x021D1A9C, "67e033b7be2a115a"),
    (0x021D1AB0, "251c7e713a619d9e"),
    (0x021D1AEC, "6f2724fd06f901d3"),
    (0x021D1B2C, "b84c72463cb4552b"),
    (0x021D1B8C, "a1581b57d10339e8"),
    (0x021D1BD8, "579374c1329e2732"),
    (0x021D1C24, "a52cc913f320a85d"),
    (0x021D1F48, "9296a3dffc5c3305"),
    (0x021D1F50, "430e5be10f1672ce"),
    (0x021D1F64, "7ef4a1c79246fb6d"),
    (0x021D213C, "ff1ae0054d8be1fe"),
    (0x021D21C0, "d67b0ade3b5c7b41"),
    (0x021D21E0, "786c488fe2cf64c2"),
    (0x021D220C, "6ce86bc0892f664a"),
    (0x021D2278, "4157fb3e10b53e4c"),
    (0x021D2358, "0bc53b34714e3222"),
    (0x021D2418, "3573e8ce97402377"),
    (0x021D2440, "e780425263c17f5f"),
    (0x021D2468, "940d232aa08adee7"),
    (0x021D2524, "cb59e1d163dd4e5c"),
    (0x021D26D0, "726b949fefce8eb8"),
    (0x021D2710, "92bab45608cd5e9f"),
    (0x021D2750, "d40d96ca6fcebb64"),
    (0x021D297C, "7fe06c653924ac92"),
    (0x021D29A8, "ff74b5361b245b77"),
    (0x021D2ADC, "4906f05f68d06a2c"),
    (0x021D2B18, "10eeb491c16f38ed"),
    (0x021D2B54, "ce8f4c77e9c0492a"),
    (0x021D2CE8, "9d922d8e6a66a140"),
    (0x021D2D24, "a943047b998980f4"),
    (0x021D2D50, "7c26f8a3a0c0ae3c"),
    (0x021D2D7C, "97c8f1fb01e8e527"),
    (0x021D2EDC, "4a78b2bf7d719043"),
    (0x021D2F44, "a82f346225cb57d4"),
    (0x021D2F6C, "cb9b821f9299da95"),
    (0x021D3018, "25d90e372aec35d5"),
    (0x021D3068, "43243cc9f335631b"),
    (0x021D3090, "c06d548746047aa9"),
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
impl Misc21Hooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5cc, &mut [Arg::V(ctx)])
    }
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32 {
        self.stubs.call(0x0215a5ec, &mut [Arg::V(ctx)])
    }
    fn fun_02158768(&mut self, gamesys: u32, script: u32, x: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x02158768,
            &mut [Arg::V(gamesys), Arg::V(script), Arg::V(x), Arg::V(heap)],
        )
    }
    fn fun_021587b0(&mut self, event: u32, script: u32, x: u32, heap: u32) -> u32 {
        self.stubs.call(
            0x021587b0,
            &mut [Arg::V(event), Arg::V(script), Arg::V(x), Arg::V(heap)],
        )
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f58, &mut [Arg::V(sw)])
    }
    fn fun_02158f74(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x02158f74, &mut [Arg::V(sw)])
    }
    fn fun_02159058(&mut self, sw: u32, a: u32, b: u32, c: u32, d: u32) {
        self.stubs.call(
            0x02159058,
            &mut [Arg::V(sw), Arg::V(a), Arg::V(b), Arg::V(c), Arg::V(d)],
        );
    }
    fn fun_02159104(&mut self, sw: u32, w: u32) {
        self.stubs.call(0x02159104, &mut [Arg::V(sw), Arg::V(w)]);
    }
    fn fun_0215910c(&mut self, sw: u32) -> u32 {
        self.stubs.call(0x0215910c, &mut [Arg::V(sw)])
    }
    fn fun_02159144(&mut self, flags: u32) {
        self.stubs.call(0x02159144, &mut [Arg::V(flags)]);
    }
    fn fun_02159168(&mut self, flags: u32) {
        self.stubs.call(0x02159168, &mut [Arg::V(flags)]);
    }
    fn fun_020124e4(&mut self, event: u32) -> u32 {
        self.stubs.call(0x020124e4, &mut [Arg::V(event)])
    }
    fn fun_02012374(&mut self, event: u32, child: u32) {
        self.stubs
            .call(0x02012374, &mut [Arg::V(event), Arg::V(child)]);
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x0201210c, &mut [Arg::V(gamesys)])
    }
    fn fun_02012844(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012844, &mut [Arg::V(gamedata)])
    }
    fn fun_0201293c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x0201293c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012934, &mut [Arg::V(gamedata)])
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
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f2c, &mut [Arg::V(gamedata)])
    }
    fn fun_02012f38(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f38, &mut [Arg::V(gamedata)])
    }
    fn fun_02012f8c(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x02012f8c, &mut [Arg::V(gamedata)])
    }
    fn fun_02013c60(&mut self, zone: u32) -> u32 {
        self.stubs.call(0x02013c60, &mut [Arg::V(zone)])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.stubs.call(0x0201aa18, &mut [Arg::V(party)])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.stubs.call(0x0201ab28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32 {
        self.stubs
            .call(0x02017e1c, &mut [Arg::V(mon), Arg::V(id), Arg::V(x)])
    }
    fn fun_02017e88(&mut self, boxmon: u32, id: u32, x: u32) -> u32 {
        self.stubs
            .call(0x02017e88, &mut [Arg::V(boxmon), Arg::V(id), Arg::V(x)])
    }
    fn fun_020079bc(&mut self, bx: u32, b: u32, s: u32, id: u32, x: u32) -> u32 {
        self.stubs.call(
            0x020079bc,
            &mut [Arg::V(bx), Arg::V(b), Arg::V(s), Arg::V(id), Arg::V(x)],
        )
    }
    fn fun_02007b44(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02007b44, &mut [Arg::V(save)])
    }
    fn fun_0200bac8(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200bac8, &mut [Arg::V(save)])
    }
    fn fun_0200bbec(&mut self, block: u32, k: u32, i: u32) -> u32 {
        self.stubs
            .call(0x0200bbec, &mut [Arg::V(block), Arg::V(k), Arg::V(i)])
    }
    fn fun_0200be48(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200be48, &mut [Arg::V(save)])
    }
    fn fun_0200c0f0(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200c0f0, &mut [Arg::V(save)])
    }
    fn fun_0200c234(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200c234, &mut [Arg::V(x)])
    }
    fn fun_02008530(&mut self, status: u32) -> u32 {
        self.stubs.call(0x02008530, &mut [Arg::V(status)])
    }
    fn fun_02008550(&mut self, status: u32) -> u32 {
        self.stubs.call(0x02008550, &mut [Arg::V(status)])
    }
    fn fun_02008554(&mut self, rec: u32) -> u32 {
        self.stubs.call(0x02008554, &mut [Arg::V(rec)])
    }
    fn fun_0200856c(&mut self, status: u32) -> u32 {
        self.stubs.call(0x0200856c, &mut [Arg::V(status)])
    }
    fn fun_02008570(&mut self, rec: u32) -> u32 {
        self.stubs.call(0x02008570, &mut [Arg::V(rec)])
    }
    fn fun_02008854(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02008854, &mut [Arg::V(save)])
    }
    fn fun_02008990(&mut self, x: u32) -> u32 {
        self.stubs.call(0x02008990, &mut [Arg::V(x)])
    }
    fn fun_02008b08(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02008b08, &mut [Arg::V(save)])
    }
    fn fun_02008b94(&mut self, x: u32) -> u32 {
        self.stubs.call(0x02008b94, &mut [Arg::V(x)])
    }
    fn fun_02008b98(&mut self, x: u32) -> u32 {
        self.stubs.call(0x02008b98, &mut [Arg::V(x)])
    }
    fn fun_02008c58(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02008c58, &mut [Arg::V(save)])
    }
    fn fun_02008dac(&mut self, counters: u32, id: u32, n: u32) {
        self.stubs
            .call(0x02008dac, &mut [Arg::V(counters), Arg::V(id), Arg::V(n)]);
    }
    fn fun_02008dfc(&mut self, counters: u32, id: u32) -> u32 {
        self.stubs
            .call(0x02008dfc, &mut [Arg::V(counters), Arg::V(id)])
    }
    fn fun_020092d8(&mut self, z: u32) -> u32 {
        self.stubs.call(0x020092d8, &mut [Arg::V(z)])
    }
    fn fun_020092ec(&mut self, z: u32) -> u32 {
        self.stubs.call(0x020092ec, &mut [Arg::V(z)])
    }
    fn fun_02009390(&mut self, z: u32, i: u32) -> u32 {
        self.stubs.call(0x02009390, &mut [Arg::V(z), Arg::V(i)])
    }
    fn fun_020093d8(&mut self, z: u32) -> u32 {
        self.stubs.call(0x020093d8, &mut [Arg::V(z)])
    }
    fn fun_020093f0(&mut self, z: u32, a: u32) {
        self.stubs.call(0x020093f0, &mut [Arg::V(z), Arg::V(a)]);
    }
    fn fun_02009418(&mut self, z: u32) -> u32 {
        self.stubs.call(0x02009418, &mut [Arg::V(z)])
    }
    fn fun_02009428(&mut self, z: u32, x: u32) {
        self.stubs.call(0x02009428, &mut [Arg::V(z), Arg::V(x)]);
    }
    fn fun_02009470(&mut self, save: u32) -> u32 {
        self.stubs.call(0x02009470, &mut [Arg::V(save)])
    }
    fn fun_02009590(&mut self, s: u32, status: u32, v: u32) -> u32 {
        self.stubs
            .call(0x02009590, &mut [Arg::V(s), Arg::V(status), Arg::V(v)])
    }
    fn fun_020095c8(&mut self, s: u32) -> u32 {
        self.stubs.call(0x020095c8, &mut [Arg::V(s)])
    }
    fn fun_020095fc(&mut self, s: u32, i: u32) {
        self.stubs.call(0x020095fc, &mut [Arg::V(s), Arg::V(i)]);
    }
    fn fun_02009618(&mut self, s: u32, i: u32) {
        self.stubs.call(0x02009618, &mut [Arg::V(s), Arg::V(i)]);
    }
    fn fun_02009628(&mut self, s: u32) -> u32 {
        self.stubs.call(0x02009628, &mut [Arg::V(s)])
    }
    fn fun_02009634(&mut self, s: u32, i: u32, field: u32) -> u32 {
        self.stubs
            .call(0x02009634, &mut [Arg::V(s), Arg::V(i), Arg::V(field)])
    }
    fn fun_02009698(&mut self, s: u32, i: u32, field: u32, v: u32) {
        self.stubs.call(
            0x02009698,
            &mut [Arg::V(s), Arg::V(i), Arg::V(field), Arg::V(v)],
        );
    }
    fn fun_0200973c(&mut self, s: u32, i: u32) -> u32 {
        self.stubs.call(0x0200973c, &mut [Arg::V(s), Arg::V(i)])
    }
    fn fun_020249b0(&mut self, flags: u32, i: u32) -> u32 {
        self.stubs.call(0x020249b0, &mut [Arg::V(flags), Arg::V(i)])
    }
    fn fun_02025b64(&mut self, lang: u32) -> u32 {
        self.stubs.call(0x02025b64, &mut [Arg::V(lang)])
    }
    fn fun_02025c74(&mut self, t: u32) -> u32 {
        self.stubs.call(0x02025c74, &mut [Arg::V(t)])
    }
    fn fun_0201147c(&mut self, heap: u32, arc: u32, k: u32, x: u32, work: u32, size: u32) -> u32 {
        self.stubs.call(
            0x0201147c,
            &mut [
                Arg::V(heap),
                Arg::V(arc),
                Arg::V(k),
                Arg::V(x),
                Arg::V(work),
                Arg::V(size),
            ],
        )
    }
    fn fun_02011558(&mut self, anime: u32) {
        self.stubs.call(0x02011558, &mut [Arg::V(anime)]);
    }
    fn fun_02011590(&mut self, anime: u32, step: u32) -> u32 {
        self.stubs
            .call(0x02011590, &mut [Arg::V(anime), Arg::V(step)])
    }
    fn fun_02011610(&mut self, anime: u32, t: &mut [u8; 12]) {
        self.stubs
            .call(0x02011610, &mut [Arg::V(anime), Arg::Out(&mut t[..])]);
    }
    fn fun_02011638(&mut self, anime: u32, r: &mut [u8; 12]) {
        self.stubs
            .call(0x02011638, &mut [Arg::V(anime), Arg::Out(&mut r[..])]);
    }
    fn fun_0207c9d0(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]) {
        self.stubs.call(
            0x0207c9d0,
            &mut [Arg::In(a), Arg::In(b), Arg::Out(&mut out[..])],
        );
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
    fn fun_0201ee60(&mut self, ws: u32, i: u32, v: u32) {
        self.stubs
            .call(0x0201ee60, &mut [Arg::V(ws), Arg::V(i), Arg::V(v)]);
    }
    fn fun_0201f02c(&mut self, ws: u32, i: u32, status: u32) {
        self.stubs
            .call(0x0201f02c, &mut [Arg::V(ws), Arg::V(i), Arg::V(status)]);
    }
    fn fun_0201f048(&mut self, ws: u32, i: u32, lang: u32) {
        self.stubs
            .call(0x0201f048, &mut [Arg::V(ws), Arg::V(i), Arg::V(lang)]);
    }
    fn fun_0201f094(&mut self, ws: u32, i: u32, lang: u32, v: u32) {
        self.stubs.call(
            0x0201f094,
            &mut [Arg::V(ws), Arg::V(i), Arg::V(lang), Arg::V(v)],
        );
    }
    fn fun_0201f198(&mut self, ws: u32, i: u32, v: u32) -> u32 {
        self.stubs
            .call(0x0201f198, &mut [Arg::V(ws), Arg::V(i), Arg::V(v)])
    }
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32 {
        self.stubs.call(0x020457b0, &mut [Arg::V(n), Arg::V(heap)])
    }
    fn fun_02045808(&mut self, buf: u32) {
        self.stubs.call(0x02045808, &mut [Arg::V(buf)]);
    }
    fn fun_020452e8(&mut self, window: u32) -> u32 {
        self.stubs.call(0x020452e8, &mut [Arg::V(window)])
    }
    fn fun_020056ec(&mut self, n: u32) -> u32 {
        self.stubs.call(0x020056ec, &mut [Arg::V(n)])
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.stubs.call(0x020061e4, &mut [Arg::V(se)]);
    }
    fn fun_02006148(&mut self, se: u32) -> u32 {
        self.stubs.call(0x02006148, &mut [Arg::V(se)])
    }
    fn fun_02006224(&mut self, player: u32) -> u32 {
        self.stubs.call(0x02006224, &mut [Arg::V(player)])
    }
    fn fun_020362dc(&mut self) -> u32 {
        self.stubs.call(0x020362dc, &mut [])
    }
    fn fun_0203f4cc(&mut self, t: &mut [u8; 16]) {
        self.stubs.call(0x0203f4cc, &mut [Arg::Out(&mut t[..])]);
    }
    fn fun_02188c54(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c54, &mut [Arg::V(fm)])
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c58, &mut [Arg::V(fm)])
    }
    fn fun_02188c64(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c64, &mut [Arg::V(fm)])
    }
    fn fun_02188c7c(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c7c, &mut [Arg::V(fm)])
    }
    fn fun_02188c84(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c84, &mut [Arg::V(fm)])
    }
    fn fun_02188c9c(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188c9c, &mut [Arg::V(fm)])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188cb4, &mut [Arg::V(fm)])
    }
    fn fun_02188ccc(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188ccc, &mut [Arg::V(fm)])
    }
    fn fun_02188cfc(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188cfc, &mut [Arg::V(fm)])
    }
    fn fun_02188d24(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188d24, &mut [Arg::V(fm)])
    }
    fn fun_02188d38(&mut self, fm: u32) -> u32 {
        self.stubs.call(0x02188d38, &mut [Arg::V(fm)])
    }
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32 {
        self.stubs
            .call(0x021bb990, &mut [Arg::V(proc_id), Arg::V(x), Arg::V(table)])
    }
    fn fun_021bba1c(&mut self, x: u32) {
        self.stubs.call(0x021bba1c, &mut [Arg::V(x)]);
    }
    fn fun_021bba6c(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021bba6c, &mut [Arg::V(x)])
    }
    fn fun_0216d410(&mut self, obj: u32, n: u32) {
        self.stubs.call(0x0216d410, &mut [Arg::V(obj), Arg::V(n)]);
    }
    fn fun_0216d454(&mut self, obj: u32) -> u32 {
        self.stubs.call(0x0216d454, &mut [Arg::V(obj)])
    }
    fn fun_0216d568(&mut self, obj: u32, flag: u32) {
        self.stubs
            .call(0x0216d568, &mut [Arg::V(obj), Arg::V(flag)]);
    }
    fn fun_0216d570(&mut self, obj: u32, flag: u32) {
        self.stubs
            .call(0x0216d570, &mut [Arg::V(obj), Arg::V(flag)]);
    }
    fn fun_0216d5f8(&mut self, obj: u32) -> u32 {
        self.stubs.call(0x0216d5f8, &mut [Arg::V(obj)])
    }
    fn fun_0216d60c(&mut self, mmdl: u32, dir: u32) {
        self.stubs
            .call(0x0216d60c, &mut [Arg::V(mmdl), Arg::V(dir)]);
    }
    fn fun_0216d758(&mut self, obj: u32) {
        self.stubs.call(0x0216d758, &mut [Arg::V(obj)]);
    }
    fn fun_0216d88c(&mut self, obj: u32, pos: &mut [u8; 12]) {
        self.stubs
            .call(0x0216d88c, &mut [Arg::V(obj), Arg::Out(&mut pos[..])]);
    }
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]) {
        self.stubs
            .call(0x0216d89c, &mut [Arg::V(mmdl), Arg::In(pos)]);
    }
    fn fun_0216d9a0(&mut self, objsys: u32) {
        self.stubs.call(0x0216d9a0, &mut [Arg::V(objsys)]);
    }
    fn fun_0216d9e4(&mut self, objsys: u32) {
        self.stubs.call(0x0216d9e4, &mut [Arg::V(objsys)]);
    }
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32 {
        self.stubs
            .call(0x0216de94, &mut [Arg::V(objsys), Arg::V(id)])
    }
    fn fun_0216e014(&mut self, obj: u32, pos: &[u8; 12], dir: u32) {
        self.stubs
            .call(0x0216e014, &mut [Arg::V(obj), Arg::In(pos), Arg::V(dir)]);
    }
    fn fun_0216e2e8(&mut self, objsys: u32, code: u32, o: &mut [u8; 0x10]) {
        self.stubs.call(
            0x0216e2e8,
            &mut [Arg::V(objsys), Arg::V(code), Arg::Out(&mut o[..])],
        );
    }
    fn fun_02162740(&mut self, evdata: u32) -> u32 {
        self.stubs.call(0x02162740, &mut [Arg::V(evdata)])
    }
    fn fun_02162744(&mut self, evdata: u32) -> u32 {
        self.stubs.call(0x02162744, &mut [Arg::V(evdata)])
    }
    fn fun_02162890(&mut self, evdata: u32, zone: u32, pos: &[u8; 12]) -> u32 {
        self.stubs.call(
            0x02162890,
            &mut [Arg::V(evdata), Arg::V(zone), Arg::In(pos)],
        )
    }
    fn fun_021a0ed4(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021a0ed4, &mut [Arg::V(x)])
    }
    fn fun_021a0f30(&mut self, x: u32, mode: u32) {
        self.stubs.call(0x021a0f30, &mut [Arg::V(x), Arg::V(mode)]);
    }
    fn fun_021a0f94(&mut self, x: u32) -> u32 {
        self.stubs.call(0x021a0f94, &mut [Arg::V(x)])
    }
    fn fun_021a0fa8(&mut self, x: u32) {
        self.stubs.call(0x021a0fa8, &mut [Arg::V(x)]);
    }
    fn fun_021a01d4(&mut self, s: u32) -> u32 {
        self.stubs.call(0x021a01d4, &mut [Arg::V(s)])
    }
    fn fun_021a01fc(&mut self, s: u32, a: u32, b: u32, c: u32) {
        self.stubs.call(
            0x021a01fc,
            &mut [Arg::V(s), Arg::V(a), Arg::V(b), Arg::V(c)],
        );
    }
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]) {
        self.stubs
            .call(0x021a3038, &mut [Arg::V(player), Arg::Out(&mut pos[..])]);
    }
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        self.stubs.call(0x021a3074, &mut [Arg::V(player)])
    }
    fn fun_021a3098(&mut self, player: u32) -> u32 {
        self.stubs.call(0x021a3098, &mut [Arg::V(player)])
    }
    fn fun_021a30a4(&mut self, player: u32, x: u32) {
        self.stubs
            .call(0x021a30a4, &mut [Arg::V(player), Arg::V(x)]);
    }
    fn fun_021a35fc(&mut self, player: u32, x: &mut [u8; 2], y: &mut [u8; 2], z: &mut [u8; 2]) {
        self.stubs.call(
            0x021a35fc,
            &mut [
                Arg::V(player),
                Arg::Out(&mut x[..]),
                Arg::Out(&mut y[..]),
                Arg::Out(&mut z[..]),
            ],
        );
    }
    fn fun_021a36bc(&mut self, player: u32, p: &mut [u8; 8]) {
        self.stubs
            .call(0x021a36bc, &mut [Arg::V(player), Arg::Out(&mut p[..])]);
    }
    fn fun_021c6de0(&mut self, t: u32, mv: u32, after: u32) {
        self.stubs
            .call(0x021c6de0, &mut [Arg::V(t), Arg::V(mv), Arg::V(after)]);
    }
    fn fun_021c71d4(&mut self, fm: u32, a: u32, b: u32) -> u32 {
        self.stubs
            .call(0x021c71d4, &mut [Arg::V(fm), Arg::V(a), Arg::V(b)])
    }
    fn fun_021c7260(&mut self, fm: u32, a: u32, b: u32) -> u32 {
        self.stubs
            .call(0x021c7260, &mut [Arg::V(fm), Arg::V(a), Arg::V(b)])
    }
    fn fun_021c7858(&mut self, fm: u32, n: u32, vec: &[u8; 12]) -> u32 {
        self.stubs
            .call(0x021c7858, &mut [Arg::V(fm), Arg::V(n), Arg::In(vec)])
    }
    fn fun_021cd440(&mut self, fm: u32, zone: &mut [u8; 2]) {
        self.stubs
            .call(0x021cd440, &mut [Arg::V(fm), Arg::Out(&mut zone[..])]);
    }
    fn fun_02184f88(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x02184f88, &mut [Arg::V(gamesys)])
    }
    fn fun_02184fd4(&mut self, gamesys: u32, zone: u32) -> u32 {
        self.stubs
            .call(0x02184fd4, &mut [Arg::V(gamesys), Arg::V(zone)])
    }
    fn fun_021867c4(&mut self, x: u32, heap: u32) -> u32 {
        self.stubs.call(0x021867c4, &mut [Arg::V(x), Arg::V(heap)])
    }
    fn fun_02186804(&mut self, t: u32) {
        self.stubs.call(0x02186804, &mut [Arg::V(t)]);
    }
    fn fun_02186830(&mut self, t: u32, k: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x02186830, &mut [Arg::V(t), Arg::V(k), Arg::V(heap)])
    }
    fn fun_0218683c(&mut self, u: u32) {
        self.stubs.call(0x0218683c, &mut [Arg::V(u)]);
    }
    fn fun_02186844(&mut self, u: u32, i: u32) -> u32 {
        self.stubs.call(0x02186844, &mut [Arg::V(u), Arg::V(i)])
    }
    fn fun_0218684c(&mut self, u: u32, i: u32) -> u32 {
        self.stubs.call(0x0218684c, &mut [Arg::V(u), Arg::V(i)])
    }
    fn fun_0218ec30(&mut self, e: u32, pos: u32) {
        self.stubs.call(0x0218ec30, &mut [Arg::V(e), Arg::V(pos)]);
    }
    fn fun_0218ec54(&mut self, e: u32) {
        self.stubs.call(0x0218ec54, &mut [Arg::V(e)]);
    }
    fn fun_0218ec5c(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218ec5c, &mut [Arg::V(e)])
    }
    fn fun_0218ec90(&mut self, e: u32, pos: &[u8; 12]) {
        self.stubs.call(0x0218ec90, &mut [Arg::V(e), Arg::In(pos)]);
    }
    fn fun_0218ed04(&mut self, e: u32, off: &[u8; 12]) {
        self.stubs.call(0x0218ed04, &mut [Arg::V(e), Arg::In(off)]);
    }
    fn fun_0218f05c(&mut self, e: u32) {
        self.stubs.call(0x0218f05c, &mut [Arg::V(e)]);
    }
    fn fun_0218f0a8(&mut self, e: u32) {
        self.stubs.call(0x0218f0a8, &mut [Arg::V(e)]);
    }
    fn fun_0218f0e8(&mut self, e: u32) -> u32 {
        self.stubs.call(0x0218f0e8, &mut [Arg::V(e)])
    }
    fn fun_0216bf9c(&mut self, x: u32, v: u32, status: u32) -> u32 {
        self.stubs
            .call(0x0216bf9c, &mut [Arg::V(x), Arg::V(v), Arg::V(status)])
    }
    fn fun_021690d0(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x021690d0, &mut [Arg::V(gamedata)])
    }
    fn fun_021690e0(&mut self, gamedata: u32) -> u32 {
        self.stubs.call(0x021690e0, &mut [Arg::V(gamedata)])
    }
    fn fun_021d25a4(&mut self, w: u32) {
        self.stubs.call(0x021d25a4, &mut [Arg::V(w)]);
    }
    fn fun_021d25e8(&mut self, w: u32) {
        self.stubs.call(0x021d25e8, &mut [Arg::V(w)]);
    }
    fn fun_021d25f4(&mut self, w: u32, a: u32, b: u32, c: u32, d: u32, e: u32) {
        self.stubs.call(
            0x021d25f4,
            &mut [
                Arg::V(w),
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(d),
                Arg::V(e),
            ],
        );
    }
    fn fun_021d26ac(&mut self, w: u32) {
        self.stubs.call(0x021d26ac, &mut [Arg::V(w)]);
    }
    fn fun_021f37b8(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x021f37b8, &mut [Arg::V(gamesys)])
    }
    fn fun_021f3970(
        &mut self,
        gamesys: u32,
        pos: &[u8; 12],
        vec: &[u8; 12],
        a: u32,
        b: u32,
    ) -> u32 {
        self.stubs.call(
            0x021f3970,
            &mut [
                Arg::V(gamesys),
                Arg::In(pos),
                Arg::In(vec),
                Arg::V(a),
                Arg::V(b),
            ],
        )
    }
    fn fun_021f3a44(&mut self, gamesys: u32, vec: &[u8; 12]) {
        self.stubs
            .call(0x021f3a44, &mut [Arg::V(gamesys), Arg::In(vec)]);
    }
    fn fun_021f3ac0(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x021f3ac0, &mut [Arg::V(gamesys)])
    }
    fn fun_021f3adc(&mut self, gamesys: u32) -> u32 {
        self.stubs.call(0x021f3adc, &mut [Arg::V(gamesys)])
    }
    fn fun_021f3cfc(&mut self, gamesys: u32, pos: &[u8; 12], a: u32, b: u32) -> u32 {
        self.stubs.call(
            0x021f3cfc,
            &mut [Arg::V(gamesys), Arg::In(pos), Arg::V(a), Arg::V(b)],
        )
    }
    fn fun_0200e878(&mut self, r: u32) {
        self.s(0x0200_E878, &[r]);
    }
    fn fun_0201efa4(&mut self, ws: u32, i: u32, n: u32, digits: u32, a: u32, b: u32) {
        self.s(0x0201_EFA4, &[ws, i, n, digits, a, b]);
    }
    fn fun_02045994(&mut self, buf: u32, s: &[u8; 16], n: u32) {
        self.stubs
            .call(0x0204_5994, &mut [Arg::V(buf), Arg::In(s), Arg::V(n)]);
    }
    fn fun_02045994_at(&mut self, buf: u32, src: u32, n: u32) {
        self.s(0x0204_5994, &[buf, src, n]);
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
        0x021C_F844 => event_569(h, v[0], v[1], v[2]) as u64,
        0x021C_F8E4 => count_party(h, v[0], v[1], v[2]) as u64,
        0x021C_F940 => count_box(h, v[0], v[1], v[2], v[3]) as u64,
        0x021C_F9AC => count_daycare(h, v[0], v[1], v[2], v[3]) as u64,
        0x021C_FA04 => add_other(h, v[0], v[1], v[2]) as u64,
        0x021C_FA1C => ot_list_add(h, v[0], v[1]) as u64,
        0x021C_FAC0 => find_slot(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021C_FB1C => {
            take_slot(h, v[0], v[1], v[2]);
            0
        }
        0x021C_FB68 => next_slot(h, v[0], v[1]) as u64,
        0x021C_FBA8 => event_slot(h, v[0], v[1], v[2]) as u64,
        0x021C_FC14 => take_slot_event(h, v[0], v[1], v[2]) as u64,
        0x021C_FC4C => {
            fill_matches_top(h, v[0], v[1], v[2]);
            0
        }
        0x021C_FC80 => {
            fill_matches(h, v[0], v[1], v[2], v[3]);
            0
        }
        0x021C_FCF4 => match_index(h, v[0], v[1]) as u64,
        0x021C_FD08 => match_message(h, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021C_FD88 => message_id(v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021C_FDBC => {
            match_words(h, v[0], v[1], v[2]);
            0
        }
        0x021D_182C => obj_anim_create(h, v[0], v[1], v[2], v[3]) as u64,
        0x021D_1884 => {
            obj_anim_delete(h, v[0]);
            0
        }
        0x021D_1894 => {
            obj_anim_start(h, v[0], v[1]);
            0
        }
        0x021D_18E0 => obj_anim_done(h, v[0]) as u64,
        0x021D_18F4 => {
            obj_anim_proc(h, v[0], v[1], v[2]);
            0
        }
        0x021D_1984 => {
            obj_anim_nop();
            0
        }
        0x021D_1988 => {
            obj_anim_apply(h, v[0], v[1], v[2]);
            0
        }
        0x021D_1A9C => event_350(h, v[0], v[1], v[2]) as u64,
        0x021D_1AEC => event_412(h, v[0], v[1], v[2]) as u64,
        0x021D_1BD8 => init_proc(h, v[0], v[1]) as u64,
        0x021D_1C24 => show_mon_event(h, v[0], v[1], v[2]) as u64,
        0x021D_1F48 => flag_cb(h, v[0], v[1]) as u64,
        0x021D_1F50 => {
            pause_objs(h, v[0]);
            0
        }
        0x021D_1F64 => {
            resume_objs(h, v[0]);
            0
        }
        0x021D_21C0 => in_table_a(v[0]) as u64,
        0x021D_21E0 => in_table_b(v[0]) as u64,
        0x021D_220C => count_script(h, v[0], v[1]) as u64,
        0x021D_2278 => event_2278(h, v[0], v[1], v[2]) as u64,
        0x021D_2358 => event_2358(h, v[0], v[1], v[2]) as u64,
        0x021D_2418 => start_2278(h, v[0], v[1]) as u64,
        0x021D_2440 => start_2358(h, v[0], v[1], v[2]) as u64,
        0x021D_26D0 => event_551(h, v[0], v[1], v[2]) as u64,
        0x021D_2750 => grotto_check(h, v[0]) as u64,
        0x021D_297C => start_29a8(h, v[0]) as u64,
        0x021D_29A8 => event_29a8(h, v[0], v[1], v[2]) as u64,
        0x021D_2CE8 => adjust_a(h, v[0], v[1]) as u64,
        0x021D_2D24 => adjust_b(h, v[0], v[1], v[2]) as u64,
        0x021D_2D50 => shake_event(h, v[0], v[1]) as u64,
        0x021D_2D7C => event_shake(h, v[0], v[1]) as u64,
        a => dispatch(h, a, v[0], v[1]).expect("PORTED but not dispatched") as u64,
    })
}

/// The event / proc literals and the embedded ROM tables are the build's.
#[test]
fn tables_match_rom() {
    assert_eq!(crate::script_sys::INIT_PROCS[8], INIT_PROC);
    let Some(ovy) = overlay21() else {
        return;
    };
    let rd = |a: u32, n: usize| {
        let o = (a - 0x0218_7F00) as usize;
        let mut w = [0u8; 4];
        w[..n].copy_from_slice(&ovy[o..o + n]);
        u32::from_le_bytes(w)
    };
    for (lit, val) in [
        (0x021C_F840, EVENT_569),
        (0x021C_FC48, EVENT_SLOT),
        (0x021D_1AE8, EVENT_350),
        (0x021D_1B84, EVENT_412),
        (0x021D_1BD4, EVENT_412),
        (0x021D_1C88, EVENT_SHOW_MON),
        (0x021D_243C, EVENT_2278),
        (0x021D_2464, EVENT_2358),
        (0x021D_25A0, EVENT_551),
        (0x021D_29A4, EVENT_29A8),
        (0x021D_2D78, EVENT_SHAKE),
        (0x021D_1880, ANIM_PROC_TABLE),
        (0x021C_FB18, POSITIONS_ADDR),
        (0x021D_1980, ANIM_FLAGS_ADDR),
        (0x021D_1B88, TABLE_412_ADDR),
        (0x021D_21DC, TABLE_DD614_ADDR),
        (0x021D_226C, COUNT_SCRIPTS_ADDR),
        (0x021D_2520, LOTTERY_ADDR),
        (0x021D_2D20, ADJUST_A_ADDR),
        (0x021D_2D4C, ADJUST_B_ADDR),
        (0x021D_2CD8, REWARD_ROWS_ADDR),
        (0x021C_FA80, PROC_309.0),
        (0x021C_FA84, PROC_309.1),
        (0x021D_2274, SCRIPT_OVER_500),
        (0x021C_FC10, SCRIPT_SLOT),
        (0x021D_2354, SE_2278),
        (0x021D_2978, HEAP_GROTTO),
    ] {
        assert_eq!(rd(lit, 4), val, "{lit:#x}");
    }
    assert_eq!(rd(ANIM_PROC_TABLE + 0x10, 4), ANIM_PROC_STEP);
    assert_eq!(rd(ANIM_PROC_TABLE + 0x14, 4), ANIM_PROC_NOP);
    for (k, &(x, y, z, zone)) in POSITIONS.iter().enumerate() {
        let a = POSITIONS_ADDR + 8 * k as u32;
        assert_eq!(rd(a, 2) as u16 as i16, x);
        assert_eq!(rd(a + 2, 2) as u16 as i16, y);
        assert_eq!(rd(a + 4, 2) as u16 as i16, z);
        assert_eq!(rd(a + 6, 2) as u16, zone);
    }
    for (k, &x) in ANIM_FLAGS.iter().enumerate() {
        assert_eq!(rd(ANIM_FLAGS_ADDR + 4 * k as u32, 4), x);
    }
    for (k, &x) in TABLE_412.iter().enumerate() {
        assert_eq!(rd(TABLE_412_ADDR + 4 * k as u32, 4), x);
    }
    for (k, &(key, f)) in TABLE_DD614.iter().enumerate() {
        assert_eq!(rd(TABLE_DD614_ADDR + 8 * k as u32, 4), key);
        assert_eq!(rd(TABLE_DD614_ADDR + 8 * k as u32 + 4, 4), f);
        assert_eq!(TABLE_DD614_KEYS[k], key);
    }
    for (k, &(n, s)) in COUNT_SCRIPTS.iter().enumerate() {
        assert_eq!(rd(COUNT_SCRIPTS_ADDR + 4 * k as u32, 2), u32::from(n));
        assert_eq!(rd(COUNT_SCRIPTS_ADDR + 4 * k as u32 + 2, 2), u32::from(s));
    }
    for (k, &x) in LOTTERY.iter().enumerate() {
        assert_eq!(rd(LOTTERY_ADDR + 2 * k as u32, 2), u32::from(x));
    }
    for (k, &x) in ADJUST_A.iter().enumerate() {
        assert_eq!(rd(ADJUST_A_ADDR + 4 * k as u32, 4), x);
    }
    for (k, &x) in ADJUST_B.iter().enumerate() {
        assert_eq!(rd(ADJUST_B_ADDR + 4 * k as u32, 4), x);
    }
    for (r, row) in REWARD_ROWS.iter().enumerate() {
        for (k, &x) in row.iter().enumerate() {
            assert_eq!(rd(REWARD_ROWS_ADDR + 16 * r as u32 + 4 * k as u32, 4), x);
        }
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_misc21`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_misc21",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_misc21.spec"),
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
