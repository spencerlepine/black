use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215A514, "d24658bc8fc63c5b"),
    (0x0215A55C, "2be9227f8f151e84"),
    (0x0215A5A4, "498fea7f97b03819"),
    (0x0215A5FC, "8a680f32278f480a"),
    (0x0215A60C, "5605f7f8882f9abf"),
    (0x0215A610, "e3f89d78f690106b"),
    (0x0215A614, "c2635b79ffe7a89a"),
    (0x0215A61C, "546ecc17c2c08568"),
    (0x0215A624, "2842dfa9ee6728ae"),
    (0x0215A630, "feab0ea9acc31c97"),
    (0x0215A63C, "b3eb137767f19538"),
    (0x0215A644, "fff6ff147712bedb"),
    (0x0215A64C, "e72bb7eea362b449"),
    (0x0215A654, "1a8d48cccaa09ee3"),
    (0x0215A660, "71a22de6b4071cdb"),
    (0x0215A66C, "d7581f8fb99e6a08"),
    (0x0215A678, "fed8eb27b3e3ab05"),
    (0x0215A684, "bc792aa69eae0e7a"),
    (0x0215A690, "3c2c1aeef87b4c15"),
    (0x0215A6AC, "f1a3eb5b44fb0cea"),
    (0x0215A6B8, "f72b3e74a4ad8612"),
    (0x0215A6C4, "76ff390d67321e5c"),
    (0x0215A6D0, "f4d54036121d497d"),
    (0x0215A6DC, "00e3710fbe56c081"),
    (0x0215A6FC, "621fed393e9e8e56"),
    (0x0215A738, "dc2e670d42b773db"),
    (0x0215A760, "d1c39441d908bd7f"),
    (0x0215A77C, "466f85316e7a9b4a"),
    (0x0215A7F0, "46071c77c0f91143"),
    (0x0215A864, "5d6a4349c8151b75"),
    (0x0215A920, "ac5f053897c32f25"),
    (0x0215A954, "1ea414154bf03a86"),
    (0x0215A990, "412fdf94e23f28a2"),
    (0x0215A9C0, "f8af1f7614c3a374"),
    (0x0215AA24, "8a96fb0ca7ba0bf4"),
    (0x0215AA34, "2fbe787f186c0d6f"),
    (0x0215AA48, "7cbad10ac2c8baff"),
    (0x0215AA4C, "dee7eb252e06ea74"),
    (0x0215AA50, "002e5d11df4f4d55"),
    (0x0215AA60, "56d0203542a64e4d"),
    (0x0215AA90, "5bb7969cd50e1426"),
    (0x0215AABC, "aa40f277a4fe39c8"),
    (0x0215AADC, "2e4257a3aca5462a"),
    (0x0215AAF4, "5475dd143167e8bd"),
    (0x0215AB18, "0cd1648165aec244"),
    (0x0215AB3C, "c3e6686279b8157e"),
    (0x0215AB60, "67440ef39316d9ac"),
    (0x0215AB98, "24b4a8b640ae3cfa"),
    (0x0215ABD0, "bc96ae063cb5f4ca"),
    (0x0215ABEC, "2726b4b7f83d2544"),
    (0x0215AC1C, "0f85126b68e79805"),
    (0x0215AC3C, "d7c29e73e5edd1b9"),
    (0x0215AC68, "228b430f20dd385a"),
    (0x0215ACA8, "1070e026df195789"),
    (0x0215ACC8, "6e2d06baff0f55fa"),
    (0x0215AD38, "07719c04f24a8512"),
    (0x0215AD50, "ea13e790e4919b5d"),
    (0x0215AD7C, "4d2720fd43b6ad24"),
    (0x0215ADCC, "653e844910a6ca48"),
    (0x0215AE34, "e2308825fdb1814d"),
    (0x0215AEB8, "11ff0db67c36d801"),
    (0x0215AEE8, "92887ab537769c0c"),
    (0x0215AF40, "8ddc2041ca5e687a"),
    (0x0215AF98, "a621c52b1785bd3b"),
    (0x0215AFE0, "75d78e46893c3949"),
    (0x0215AFF4, "1d743e971134961d"),
    (0x0215B014, "ef75619d1513b74a"),
    (0x0215B028, "bcdef46125585d09"),
    (0x0215B068, "859ec07a0e2155f6"),
    (0x0215B084, "9a26b57553ff779f"),
    (0x0215B0B4, "9bf4e146a795ccb1"),
    (0x0215B104, "31fedb0daed7ed46"),
    (0x0215B138, "82b872ce9fc97e3e"),
    (0x0215B158, "c0113b66036b7847"),
    (0x0215B190, "351bd0770a0e1215"),
    (0x0215B214, "a3624e20a8a61e33"),
    (0x0215B26C, "90bf07c4129c6c88"),
    (0x0215B2C4, "4ebdf820d45a7368"),
    (0x0215B328, "ef510aab429bff2b"),
    (0x0215B350, "e26337ad61377732"),
    (0x0215B378, "c2bb9ef56fab010d"),
    (0x0215B3AC, "20ebd0d4cbe4f7b9"),
    (0x0215B3DC, "892fa610df81abca"),
    (0x0215B408, "b292d29e604ab21d"),
    (0x0215B438, "01fb0ae37d3357d2"),
    (0x0215B464, "bb3e4d16b9a2dc0b"),
    (0x0215B488, "6cfb2533007bcc6b"),
    (0x0215B4AC, "78a9671616e771cf"),
    (0x0215B4DC, "3d0f2f023a3f59ea"),
    (0x0215B500, "01283eb364354281"),
    (0x0215B558, "8042d058681d7750"),
    (0x0215B5A8, "f072a42c1649861b"),
    (0x0215B5BC, "a81d5a1b3e090a09"),
    (0x0215B660, "ed88ee8176975c3c"),
    (0x0215B6C0, "f617089b3f273c34"),
    (0x0215B704, "579c8dbe5aad16c3"),
    (0x0215B74C, "5e1dc243698d453d"),
    (0x0215B7E8, "b41368fe74a2bb4f"),
    (0x0215B824, "af367696b5deafa7"),
    (0x0215B848, "95db46475521f45c"),
    (0x0215B888, "feea6ec565df5d72"),
    (0x0215B904, "6fe839088062cd65"),
    (0x0215B994, "24dc1ac3e56d02ed"),
    (0x0215BA18, "c3accaf5d2e571b8"),
    (0x0215BAB0, "56c59f4c7504dbfd"),
    (0x0215BAF4, "12a87be29876c3d6"),
    (0x0215BB50, "f1bd2f179576f21b"),
    (0x0215BBD0, "ddcc671a3b4b6206"),
    (0x0215BCA0, "3cb19b8e4d2bb87b"),
    (0x0215BCE8, "bbd72fbdeddb587e"),
    (0x0215BD2C, "4cb86b454a2c45bc"),
    (0x0215BE24, "0bcc3cf19fb94fa8"),
    (0x0215BF30, "99a7ad03b8e0640c"),
    (0x0215BF7C, "e05687e6a0d68a02"),
    (0x0215BFD4, "597a2e636a393cb8"),
    (0x0215C01C, "ba3e070057fcb7cf"),
    (0x0215C064, "5d139c5310532ced"),
    (0x0215C0D0, "92b28150ab9fa6d8"),
    (0x0215C138, "3d04cadc8e07c869"),
    (0x0215C180, "bb9b09328b017e3a"),
    (0x0215C200, "5463f38007d30d4f"),
    (0x0215C2D0, "6bf201097dda3cca"),
    (0x0215C40C, "a44495c6a1cd146d"),
    (0x0215C4C8, "62b0f76b55651a08"),
    (0x0215C510, "dc325bebd2e4a7ab"),
    (0x0215C558, "dd8394ef1ffa2329"),
    (0x0215C584, "b4d7e7ca21a57d5a"),
    (0x0215C5B0, "90330844d86a0444"),
    (0x0215C5EC, "f138c4088f70adfb"),
    (0x0215C610, "45edbbb687b0005b"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer (and the listed globals) of the case in one memory window set (at the
/// emulator's addresses).
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

impl WorkHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5CC, &[ctx])
    }
    fn fun_0215a5d0(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5D0, &[ctx])
    }
    fn fun_0215898c(&mut self, sw: u32, arg: u32) {
        self.s(0x0215_898C, &[sw, arg]);
    }
    fn fun_02158f54(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F54, &[sw])
    }
    fn fun_02158f58(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F58, &[sw])
    }
    fn fun_02158f6c(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F6C, &[sw])
    }
    fn fun_0215909c(&mut self, sw: u32) -> u32 {
        self.s(0x0215_909C, &[sw])
    }
    fn fun_021590d4(&mut self, sw: u32, buf: u32) {
        self.s(0x0215_90D4, &[sw, buf]);
    }
    fn fun_02159198(&mut self, bit: u32) -> u32 {
        self.s(0x0215_9198, &[bit])
    }
    fn fun_02159b94(&mut self, x: u32, matrix: u32, zone: u32, heap: u32) {
        self.s(0x0215_9B94, &[x, matrix, zone, heap]);
    }
    fn fun_02159cc4(&mut self, x: u32, gamesys: u32, heap: u32) {
        self.s(0x0215_9CC4, &[x, gamesys, heap]);
    }
    fn fun_02159f28(&mut self, gamedata: u32, a: u32, b: u32) {
        self.s(0x0215_9F28, &[gamedata, a, b]);
    }
    fn fun_02159f54(&mut self, gamedata: u32, v: u32) -> u32 {
        self.s(0x0215_9F54, &[gamedata, v])
    }
    fn fun_02159f84(&mut self, v: u32) -> u32 {
        self.s(0x0215_9F84, &[v])
    }
    fn fun_0215a4d8(&mut self, ew: u32, v: u32) -> u32 {
        self.s(0x0215_A4D8, &[ew, v])
    }
    fn fun_0215a4ec(&mut self, ew: u32, v: u32) {
        self.s(0x0215_A4EC, &[ew, v]);
    }
    fn fun_0215a500(&mut self, ew: u32, v: u32) {
        self.s(0x0215_A500, &[ew, v]);
    }
    fn fun_0215f6e8(&mut self, gamesys: u32) -> u32 {
        self.s(0x0215_F6E8, &[gamesys])
    }
    fn fun_02162708(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32) {
        self.s(0x0216_2708, &[x, a, b, c, d]);
    }
    fn fun_02162748(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32, e: u32) {
        self.s(0x0216_2748, &[x, a, b, c, d, e]);
    }
    fn fun_02162a74(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32) {
        self.s(0x0216_2A74, &[x, a, b, c, d]);
    }
    fn fun_021690b4(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_90B4, &[gamedata])
    }
    fn fun_021690d0(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_90D0, &[gamedata])
    }
    fn fun_021690e0(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_90E0, &[gamedata])
    }
    fn fun_021690f0(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_90F0, &[gamedata])
    }
    fn fun_02169100(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_9100, &[gamedata])
    }
    fn fun_02169110(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_9110, &[gamedata])
    }
    fn fun_0216efa8(&mut self, x: u32, v: u32) -> u32 {
        self.s(0x0216_EFA8, &[x, v])
    }
    fn fun_0216f830(&mut self, gamedata: u32) -> u32 {
        self.s(0x0216_F830, &[gamedata])
    }
    fn fun_0216f948(&mut self, mon: u32, heap: u32) -> u32 {
        self.s(0x0216_F948, &[mon, heap])
    }
    fn fun_0216faa0(&mut self, x: u32) -> u32 {
        self.s(0x0216_FAA0, &[x])
    }
    fn fun_021909bc(&mut self, handle: u32) {
        self.s(0x0219_09BC, &[handle]);
    }
    fn fun_021909e4(&mut self, handle: u32) -> u32 {
        self.s(0x0219_09E4, &[handle])
    }
    fn fun_02190a20(&mut self, n: u32, heap: u32) -> u32 {
        self.s(0x0219_0A20, &[n, heap])
    }
    fn fun_02190a64(&mut self, list: u32, text: u32, param: u32, heap: u32) {
        self.s(0x0219_0A64, &[list, text, param, heap]);
    }
    fn fun_02190ac4(&mut self, list: u32) -> u32 {
        self.s(0x0219_0AC4, &[list])
    }
    fn fun_02190acc(&mut self, fm: u32, list: u32, a: u32, b: u32) -> u32 {
        self.s(0x0219_0ACC, &[fm, list, a, b])
    }
    fn fun_02190aec(&mut self, rows: u32, a: u32, b: u32, scroll: u32) -> u32 {
        self.s(0x0219_0AEC, &[rows, a, b, scroll])
    }
    fn fun_02191164(&mut self, window: u32) {
        self.s(0x0219_1164, &[window]);
    }
    fn fun_02191184(&mut self, window: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_1184, &[window, x, y, text]);
    }
    fn fun_02197da0(&mut self, obj: u32) -> u32 {
        self.s(0x0219_7DA0, &[obj])
    }
    fn fun_02197dac(&mut self, obj: u32) {
        self.s(0x0219_7DAC, &[obj]);
    }
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.s(0x0203_0734, &[heap, size, flag, file, line])
    }
    fn fun_020307b0(&mut self, block: u32) {
        self.s(0x0203_07B0, &[block]);
    }
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32) {
        self.s(0x0201_F250, &[wordset, dst, src]);
    }
    fn fun_0201f91c(&mut self, list: u32) -> u32 {
        self.s(0x0201_F91C, &[list])
    }
    fn fun_02020d0c(&mut self, menu: u32, a: u32) -> u32 {
        self.s(0x0202_0D0C, &[menu, a])
    }
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32 {
        self.s(0x0204_57B0, &[len, heap])
    }
    fn fun_02045808(&mut self, strbuf: u32) {
        self.s(0x0204_5808, &[strbuf]);
    }
    fn fun_02045874(&mut self, strbuf: u32, heap: u32) -> u32 {
        self.s(0x0204_5874, &[strbuf, heap])
    }
    fn fun_0204589c(&mut self, a: u32, b: u32) -> u32 {
        self.s(0x0204_589C, &[a, b])
    }
    fn fun_02045b38(&mut self, a: u32, ty: u32, msg: u32, heap: u32) -> u32 {
        self.s(0x0204_5B38, &[a, ty, msg, heap])
    }
    fn fun_02045c04(&mut self, msgdata: u32) {
        self.s(0x0204_5C04, &[msgdata]);
    }
    fn fun_02045cac(&mut self, msgdata: u32, id: u32, strbuf: u32) {
        self.s(0x0204_5CAC, &[msgdata, id, strbuf]);
    }
    fn fun_02045ec0(&mut self, msgdata: u32, id: u32) -> u32 {
        self.s(0x0204_5EC0, &[msgdata, id])
    }
    fn fun_020056ec(&mut self, v: u32) -> u32 {
        self.s(0x0200_56EC, &[v])
    }
    fn fun_020071e4(&mut self, x: u32) -> u32 {
        self.s(0x0200_71E4, &[x])
    }
    fn fun_020071ec(&mut self, x: u32) -> u32 {
        self.s(0x0200_71EC, &[x])
    }
    fn fun_0200725c(&mut self, x: u32, a: u32, b: u32) -> u32 {
        self.s(0x0200_725C, &[x, a, b])
    }
    fn fun_020072a4(&mut self, x: u32, a: u32) {
        self.s(0x0200_72A4, &[x, a]);
    }
    fn fun_020073d8(&mut self, x: u32, a: u32, b: u32) -> u32 {
        self.s(0x0200_73D8, &[x, a, b])
    }
    fn fun_020074d8(&mut self, boxes: u32, mon: u32) {
        self.s(0x0200_74D8, &[boxes, mon]);
    }
    fn fun_02007984(&mut self, boxes: u32) -> u32 {
        self.s(0x0200_7984, &[boxes])
    }
    fn fun_020079a0(&mut self, boxes: u32) -> u32 {
        self.s(0x0200_79A0, &[boxes])
    }
    fn fun_02007a68(&mut self, boxes: u32, v: u32) {
        self.s(0x0200_7A68, &[boxes, v]);
    }
    fn fun_020083e4(&mut self, x: u32) -> u32 {
        self.s(0x0200_83E4, &[x])
    }
    fn fun_020083ec(&mut self, x: u32, v: u32) {
        self.s(0x0200_83EC, &[x, v]);
    }
    fn fun_02008428(&mut self, x: u32) -> u32 {
        self.s(0x0200_8428, &[x])
    }
    fn fun_02008500(&mut self, player: u32, strbuf: u32) {
        self.s(0x0200_8500, &[player, strbuf]);
    }
    fn fun_02008510(&mut self, player: u32, heap: u32) -> u32 {
        self.s(0x0200_8510, &[player, heap])
    }
    fn fun_02008530(&mut self, player: u32) -> u32 {
        self.s(0x0200_8530, &[player])
    }
    fn fun_02008550(&mut self, player: u32) -> u32 {
        self.s(0x0200_8550, &[player])
    }
    fn fun_02008554(&mut self, x: u32) -> u32 {
        self.s(0x0200_8554, &[x])
    }
    fn fun_0200873c(&mut self, x: u32) -> u32 {
        self.s(0x0200_873C, &[x])
    }
    fn fun_020089c0(&mut self, x: u32) -> u32 {
        self.s(0x0200_89C0, &[x])
    }
    fn fun_020089f4(&mut self, x: u32) -> u32 {
        self.s(0x0200_89F4, &[x])
    }
    fn fun_02008a00(&mut self, x: u32) {
        self.s(0x0200_8A00, &[x]);
    }
    fn fun_02008a7c(&mut self, x: u32, v: u32, t0: u32, t1: u32, t2: u32) {
        self.s(0x0200_8A7C, &[x, v, t0, t1, t2]);
    }
    fn fun_0200c234(&mut self, x: u32, v: u32) -> u32 {
        self.s(0x0200_C234, &[x, v])
    }
    fn fun_0200c248(&mut self, x: u32, v: u32) {
        self.s(0x0200_C248, &[x, v]);
    }
    fn fun_0200c258(&mut self, x: u32) -> u32 {
        self.s(0x0200_C258, &[x])
    }
    fn fun_0200c274(&mut self, x: u32) -> u32 {
        self.s(0x0200_C274, &[x])
    }
    fn fun_0200c278(&mut self, x: u32, v: u32) {
        self.s(0x0200_C278, &[x, v]);
    }
    fn fun_0200c29c(&mut self, x: u32, v: u32) {
        self.s(0x0200_C29C, &[x, v]);
    }
    fn fun_0200c2f0(&mut self, x: u32, v: u32, a: u32) {
        self.s(0x0200_C2F0, &[x, v, a]);
    }
    fn fun_0200c308(&mut self, x: u32, v: u32) -> u32 {
        self.s(0x0200_C308, &[x, v])
    }
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0200_CA1C, &[gamedata])
    }
    fn fun_0200ca28(&mut self, dex: u32) {
        self.s(0x0200_CA28, &[dex]);
    }
    fn fun_0200ca38(&mut self, dex: u32) -> u32 {
        self.s(0x0200_CA38, &[dex])
    }
    fn fun_0200ca40(&mut self, dex: u32) {
        self.s(0x0200_CA40, &[dex]);
    }
    fn fun_0200cde0(&mut self, dex: u32, mon: u32) {
        self.s(0x0200_CDE0, &[dex, mon]);
    }
    fn fun_0200ced4(&mut self, dex: u32, v: u32) -> u32 {
        self.s(0x0200_CED4, &[dex, v])
    }
    fn fun_0200cf28(&mut self, dex: u32) -> u32 {
        self.s(0x0200_CF28, &[dex])
    }
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32) {
        self.s(0x0200_CFA0, &[dex, mon]);
    }
    fn fun_0200d064(&mut self, dex: u32, v: u32) -> u32 {
        self.s(0x0200_D064, &[dex, v])
    }
    fn fun_0200d0cc(&mut self, dex: u32) -> u32 {
        self.s(0x0200_D0CC, &[dex])
    }
    fn fun_0200ecb4(&mut self, x: u32) -> u32 {
        self.s(0x0200_ECB4, &[x])
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
    fn fun_02012900(&mut self, gamedata: u32, v: u32) {
        self.s(0x0201_2900, &[gamedata, v]);
    }
    fn fun_02012934(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2934, &[gamedata])
    }
    fn fun_0201293c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_293C, &[gamedata])
    }
    fn fun_02012944(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2944, &[gamedata])
    }
    fn fun_02012964(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2964, &[gamedata])
    }
    fn fun_02012974(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2974, &[gamedata])
    }
    fn fun_02012984(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2984, &[gamedata])
    }
    fn fun_02012a44(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2A44, &[gamedata])
    }
    fn fun_02012aa0(&mut self, loc: u32) -> u32 {
        self.s(0x0201_2AA0, &[loc])
    }
    fn fun_02012ac8(&mut self, loc: u32) -> u32 {
        self.s(0x0201_2AC8, &[loc])
    }
    fn fun_02012ae4(&mut self, loc: u32) -> u32 {
        self.s(0x0201_2AE4, &[loc])
    }
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2EBC, &[gamedata])
    }
    fn fun_02012f08(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F08, &[gamedata])
    }
    fn fun_02012f5c(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F5C, &[gamedata])
    }
    fn fun_02013b44(&mut self, zone: u32) -> u32 {
        self.s(0x0201_3B44, &[zone])
    }
    fn fun_02013e4c(&mut self, zone: u32) -> u32 {
        self.s(0x0201_3E4C, &[zone])
    }
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32 {
        self.s(0x0201_44DC, &[gamesys, fm])
    }
    fn fun_020145cc(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_45CC, &[gamesys])
    }
    fn fun_020173f0(&mut self, species: u32, level: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0201_73F0, &[species, level, a, b, heap])
    }
    fn fun_02017430(
        &mut self,
        species: u32,
        level: u32,
        ot: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        heap: u32,
    ) -> u32 {
        self.s(0x0201_7430, &[species, level, ot, a, b, c, d, heap])
    }
    fn fun_02017988(&mut self, mon: u32, v: u32) {
        self.s(0x0201_7988, &[mon, v]);
    }
    fn fun_02017a20(&mut self, mon: u32, a: u32, b: u32) {
        self.s(0x0201_7A20, &[mon, a, b]);
    }
    fn fun_02017d30(&mut self, mon: u32) -> u32 {
        self.s(0x0201_7D30, &[mon])
    }
    fn fun_02017d70(&mut self, mon: u32, k: u32) {
        self.s(0x0201_7D70, &[mon, k]);
    }
    fn fun_02017e1c(&mut self, mon: u32, param: u32, out: u32) -> u32 {
        self.s(0x0201_7E1C, &[mon, param, out])
    }
    fn fun_02017e40(&mut self, mon: u32, param: u32, v: u32) {
        self.s(0x0201_7E40, &[mon, param, v]);
    }
    fn fun_020180f4(&mut self, mon: u32, mv: u32) {
        self.s(0x0201_80F4, &[mon, mv]);
    }
    fn fun_020181f4(&mut self, mon: u32, mv: u32, slot: u32) {
        self.s(0x0201_81F4, &[mon, mv, slot]);
    }
    fn fun_020185c4(&mut self, mon: u32) {
        self.s(0x0201_85C4, &[mon]);
    }
    fn fun_020185f4(&mut self, mon: u32) -> u32 {
        self.s(0x0201_85F4, &[mon])
    }
    fn fun_020186c8(&mut self, ot: u32, species: u32, form: u32, a: u32, b: u32, c: u32) -> u32 {
        self.s(0x0201_86C8, &[ot, species, form, a, b, c])
    }
    fn fun_0201a98c(&mut self, party: u32, mon: u32) {
        self.s(0x0201_A98C, &[party, mon]);
    }
    fn fun_0201aa14(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA14, &[party])
    }
    fn fun_0201aa18(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA18, &[party])
    }
    fn fun_0201aa1c(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA1C, &[party])
    }
    fn fun_0201aa50(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA50, &[party])
    }
    fn fun_0201aa88(&mut self, party: u32) -> u32 {
        self.s(0x0201_AA88, &[party])
    }
    fn fun_0201aad4(&mut self, party: u32) -> u32 {
        self.s(0x0201_AAD4, &[party])
    }
    fn fun_0201ab08(&mut self, party: u32) -> u32 {
        self.s(0x0201_AB08, &[party])
    }
    fn fun_0201ab18(&mut self, party: u32) -> u32 {
        self.s(0x0201_AB18, &[party])
    }
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32 {
        self.s(0x0201_AB28, &[party, i])
    }
    fn fun_0201ade4(&mut self, species: u32, form: u32, heap: u32) -> u32 {
        self.s(0x0201_ADE4, &[species, form, heap])
    }
    fn fun_0201ae14(&mut self, pd: u32) {
        self.s(0x0201_AE14, &[pd]);
    }
    fn fun_0201ae1c(&mut self, pd: u32, field: u32) -> u32 {
        self.s(0x0201_AE1C, &[pd, field])
    }
    fn fun_0201ba34(&mut self, party: u32) {
        self.s(0x0201_BA34, &[party]);
    }
    fn fun_02020fe4(&mut self, item: u32, field: u32, heap: u32) -> u32 {
        self.s(0x0202_0FE4, &[item, field, heap])
    }
    fn fun_0202142c(&mut self, item: u32) -> u32 {
        self.s(0x0202_142C, &[item])
    }
    fn fun_02025c88(&mut self, x: u32) -> u32 {
        self.s(0x0202_5C88, &[x])
    }
    fn fun_0202f190(&mut self, mon: u32, a: u32, player: u32, b: u32, heap: u32) {
        self.s(0x0202_F190, &[mon, a, player, b, heap]);
    }
    fn fun_020362dc(&mut self) -> u32 {
        self.s(0x0203_62DC, &[])
    }
    #[allow(clippy::too_many_arguments)]
    fn fun_02190988(
        &mut self,
        fm: u32,
        header: &[u8; 0x20],
        list: u32,
        cb: u32,
        ctx: u32,
        a: u32,
        cursor: u32,
        flag: u32,
    ) -> u32 {
        let mut args = [
            Arg::V(fm),
            Arg::In(header),
            Arg::V(list),
            Arg::V(cb),
            Arg::V(ctx),
            Arg::V(a),
            Arg::V(cursor),
            Arg::V(flag),
        ];
        self.stubs.call(0x0219_0988, &mut args)
    }
    fn fun_02190b08(&mut self, header: &mut [u8; 0x20], n: u32, x: u32, y: u32, w: u32, h: u32) {
        let mut args = [
            Arg::In(header),
            Arg::V(n),
            Arg::V(x),
            Arg::V(y),
            Arg::V(w),
            Arg::V(h),
        ];
        self.stubs.call(0x0219_0B08, &mut args);
    }
    fn fun_020202ec(&mut self, menu: u32, idx: &mut [u8; 2]) {
        self.stubs
            .call(0x0202_02EC, &mut [Arg::V(menu), Arg::Out(idx)]);
    }
    fn fun_0200723c(&mut self, x: u32, a: &mut [u8; 4], b: &mut [u8; 4]) {
        self.stubs
            .call(0x0200_723C, &mut [Arg::V(x), Arg::Out(a), Arg::Out(b)]);
    }
    fn fun_0201289c(&mut self, gamedata: u32, rec: &[u8; 0x1C]) {
        self.stubs
            .call(0x0201_289C, &mut [Arg::V(gamedata), Arg::In(rec)]);
    }
    fn fun_02014184(&mut self, rec: &mut [u8; 0x1C], zone: u32, dir: u32, x: u32, y: u32, z: u32) {
        let mut args = [
            Arg::Out(rec),
            Arg::V(zone),
            Arg::V(dir),
            Arg::V(x),
            Arg::V(y),
            Arg::V(z),
        ];
        self.stubs.call(0x0201_4184, &mut args);
    }
    fn fun_0203f4e4(&mut self, t: &mut [u8; 12]) {
        self.stubs.call(0x0203_F4E4, &mut [Arg::Out(t)]);
    }
    fn fun_02087d28(&mut self, t: &mut [u8; 8]) {
        self.stubs.call(0x0208_7D28, &mut [Arg::Out(t)]);
    }
}

/// Run `f` on the case's memory: every non-empty argument buffer at its address and the globals at `globals` (spec
/// order); the bytes are put back afterwards.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals: Vec<u32> = (0..9).map(|i| c.word(i)).collect();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(c.vals[i], std::mem::take(b));
            keep.push(i);
        }
    }
    for (k, &g) in globals.iter().enumerate() {
        mem.add_window(g, std::mem::take(&mut c.globals[k]));
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let mut wins = std::mem::take(&mut rom.mem.extra).into_iter();
    for (i, (_, bytes)) in keep.into_iter().zip(wins.by_ref()) {
        c.bufs[i] = bytes;
    }
    for (k, (_, bytes)) in wins.enumerate() {
        c.globals[k] = bytes;
    }
    r
}

/// `ofs:argK` results: the offset of a returned address into argument `k` (0 stays 0).
fn ofs(r: u32, base: u32) -> u64 {
    if r == 0 {
        0
    } else {
        r.wrapping_sub(base) as u64
    }
}

/// The `N` bytes at `addr`.
fn bytes<const N: usize>(h: &mut Rom, addr: u32) -> [u8; N] {
    let mut b = [0u8; N];
    for (i, x) in b.iter_mut().enumerate() {
        *x = h.r8(addr + i as u32);
    }
    b
}

fn run(addr: u32, c: &mut Case) -> u64 {
    let globals: &[u32] = if addr == 0x0215_C2D0 {
        &[EGG_NAME_MSGDATA]
    } else {
        &[]
    };
    with_rom(c, globals, |h, v| match addr {
        0x0215_A514 => ofs(work_new(h, v[0], v[1]), v[2]),
        0x0215_A55C => {
            let init = bytes::<16>(h, v[0]);
            ofs(ctx_new(h, &init, v[1]), v[2])
        }
        0x0215_A5A4 => {
            ctx_free(h, v[0]);
            0
        }
        0x0215_A5FC => field_block_0(h, v[0]) as u64,
        0x0215_A60C => ctx_msgdata(h, v[0]) as u64,
        0x0215_A610 => ctx_msg_bank(h, v[0]) as u64,
        0x0215_A614 => {
            set_window(h, v[0], v[1]);
            0
        }
        0x0215_A61C => window(h, v[0]) as u64,
        0x0215_A624 => {
            set_work_100(h, v[0], v[1]);
            0
        }
        0x0215_A630 => work_100(h, v[0]) as u64,
        0x0215_A63C => ofs(work_d4_addr(h, v[0]), v[0]),
        0x0215_A644 => {
            set_work_10(h, v[0], v[1]);
            0
        }
        0x0215_A64C => work_10(h, v[0]) as u64,
        0x0215_A654 => work_104(h, v[0]) as u64,
        0x0215_A660 => {
            set_work_104(h, v[0], v[1]);
            0
        }
        0x0215_A66C => work_105(h, v[0]) as u64,
        0x0215_A678 => {
            set_work_105(h, v[0], v[1]);
            0
        }
        0x0215_A684 => {
            set_countdown(h, v[0], v[1]);
            0
        }
        0x0215_A690 => countdown(h, v[0]) as u64,
        0x0215_A6AC => work_108(h, v[0]) as u64,
        0x0215_A6B8 => {
            set_work_108(h, v[0], v[1]);
            0
        }
        0x0215_A6C4 => work_10c(h, v[0]) as u64,
        0x0215_A6D0 => {
            set_work_10c(h, v[0], v[1]);
            0
        }
        0x0215_A6DC => {
            add_slot_object(h, v[0], v[1]);
            0
        }
        0x0215_A6FC => slots_wait(h, v[0]) as u64,
        0x0215_A738 => {
            slots_clear(h, v[0]);
            0
        }
        0x0215_A760 => {
            message_bank_open(h, v[0], v[1], v[2]);
            0
        }
        0x0215_A77C => {
            menu_begin(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8]);
            0
        }
        0x0215_A7F0 => {
            menu_add(h, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        0x0215_A864 => {
            menu_open(h, v[0]);
            0
        }
        0x0215_A920 => {
            menu_close(h, v[0]);
            0
        }
        0x0215_A954 => menu_poll(h, v[0]) as u64,
        0x0215_A990 => menu_poll_x(h, v[0]) as u64,
        0x0215_A9C0 => {
            menu_cursor_cb(h, v[0]);
            0
        }
        0x0215_AA24 => {
            save_vars(h, v[0]);
            0
        }
        0x0215_AA34 => {
            restore_vars(h, v[0]);
            0
        }
        0x0215_AA48 => {
            set_wait_slot(h, v[0], v[1]);
            0
        }
        0x0215_AA4C => wait_slot(h, v[0]) as u64,
        0x0215_B4AC => party_mon_to(h, v[0], v[1], v[2]) as u64,
        0x0215_B4DC => party_mon_param(h, v[0], v[1], v[2]) as u64,
        0x0215_B500 => mon_knows_move(h, v[0], v[1]) as u64,
        0x0215_C40C => {
            let rec = bytes::<0x24>(h, v[1]);
            build_mon(h, v[0], &rec) as u64
        }
        0x0215_C4C8 => {
            let rec = bytes::<0x24>(h, v[1]);
            give_box(h, v[0], &rec) as u64
        }
        0x0215_C510 => {
            let rec = bytes::<0x24>(h, v[1]);
            give_party(h, v[0], &rec) as u64
        }
        a => dispatch(h, a, v[0], v[1])
            .unwrap_or_else(|| unreachable!("{a:#010x} is PORTED but not wired into the test"))
            as u64,
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-script scrcmd_work`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "scrcmd_work",
            include_str!("../../../../tools/rom-diff/specs/black-script/scrcmd_work.spec"),
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

/// The embedded command-562 table is the ROM's.
#[test]
fn table_562_matches_rom() {
    let Some(ovy) = overlay10() else {
        return;
    };
    let o = (TABLE_562_ADDR - 0x0215_5160) as usize;
    let rom: Vec<u16> = ovy[o..o + 16]
        .chunks(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    assert_eq!(rom, TABLE_562);
}

/// The other embedded overlay-10 tables are the ROM's.
#[test]
fn tables_match_rom() {
    let Some(ovy) = overlay10() else {
        return;
    };
    let at = |a: u32, n: usize| {
        let o = (a - 0x0215_5160) as usize;
        ovy[o..o + n].to_vec()
    };
    let u16s = |b: Vec<u8>| -> Vec<u16> {
        b.chunks(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect()
    };
    assert_eq!(at(MENU_HEADER_ADDR, 0x20), MENU_HEADER);
    assert_eq!(u16s(at(TABLE_563_ADDR, 16)), TABLE_563);
    assert_eq!(u16s(at(TABLE_281_ADDR, 4)), TABLE_281);
    assert_eq!(at(PARAM_WHITELIST_ADDR, 12), PARAM_WHITELIST);
    let iv = u16s(at(IV_TABLE_ADDR, 24));
    for (i, &(p, m)) in IV_TABLE.iter().enumerate() {
        assert_eq!((iv[2 * i], iv[2 * i + 1]), (p, m));
    }
}

/// [`VERSION`] is the byte at main 0x0209DF08 (`$BLACK_ROM_BUILD/main.sbin`, loaded at 0x02004000).
#[test]
fn version_matches_rom() {
    let Some(main) = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/main.sbin")).ok())
    else {
        return;
    };
    assert_eq!(main[(VERSION_ADDR - 0x0200_4000) as usize], VERSION);
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
