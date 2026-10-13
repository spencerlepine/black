use super::*;
use crate::hooks::{Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B0084, "44bfafb1cb50c8be"),
    (0x021B010C, "a0e28048e4f30448"),
    (0x021B0130, "a431f20a1938bdd6"),
    (0x021B015C, "1c7d1f979c1e28a3"),
    (0x021B01B8, "d20634086abd729b"),
    (0x021B01DC, "9021d6e43ad28641"),
    (0x021B024C, "992ff8646499cbc7"),
    (0x021B02C8, "d53a43e67c41bb0b"),
    (0x021B0344, "c778d1108b6558af"),
    (0x021B0394, "6e62c439bf093925"),
    (0x021B03BC, "4f3913b2b707fe75"),
    (0x021B03E4, "2d987ccd14ce5ca7"),
    (0x021B0480, "4ff798c83efd316c"),
    (0x021B04C0, "003bcd53aeb3c3b8"),
    (0x021B04D8, "29c890608f54bd2f"),
    (0x021B0530, "409a44297938e9ee"),
    (0x021B05AC, "2b5d92a207d5925a"),
    (0x021B0624, "5d9de383f1083f85"),
    (0x021B0630, "e2ae91bf4f7a47dd"),
    (0x021B063C, "72f45da896af459e"),
    (0x021B0730, "6bb5aac45013df32"),
    (0x021B07E0, "3f6008ba1fb30022"),
    (0x021B0808, "96a843542e6c7104"),
    (0x021B090C, "eecd9af9ea932435"),
    (0x021B0960, "ed5a84a96aec5c31"),
    (0x021B09D8, "afd9a0f4d6ddcb04"),
    (0x021B09EC, "badfd511a420926a"),
    (0x021B0A40, "2a3cfb6b2bac89f3"),
    (0x021B0A60, "b623f7a894c7e387"),
    (0x021B0A88, "939edf87048cd19f"),
    (0x021B0AB0, "be93cdf0c8eee13f"),
    (0x021B0ADC, "ae9987237a16f19e"),
    (0x021B0B08, "e89c37826ed57446"),
    (0x021B0BAC, "005133b9d47220de"),
    (0x021B0CEC, "2764da2a352e1ca2"),
    (0x021B0D14, "8a06319541a71799"),
    (0x021B0D74, "83bf2b5728cd6c0b"),
    (0x021B0DF0, "dd3645a383662ca8"),
    (0x021B0E74, "a9a3f61f4f839ecc"),
    (0x021B0EDC, "8f3b24b4bbf46878"),
    (0x021B0EE8, "28f2d081f4695f17"),
    (0x021B0EF4, "236cef5c95888869"),
    (0x021B0FF0, "19fcaededd93065f"),
    (0x021B1008, "6ef46d5da4b063ff"),
    (0x021B10A4, "0c2fa1a912b29318"),
    (0x021B10D0, "609481951b157233"),
    (0x021B10DC, "88e9215878c5645f"),
    (0x021B10E8, "20f4241110708b11"),
    (0x021B10F4, "b154d6d9e8d13036"),
    (0x021B1100, "e423e8a5997110c7"),
    (0x021B11D8, "9db8c79dffd4df4b"),
    (0x021B120C, "a1c56c01e79c9c5c"),
    (0x021B1234, "c05d29f865055a8e"),
    (0x021B1240, "ad7b326af44a100f"),
    (0x021B1268, "7c98e3d9af2062b6"),
    (0x021B12B0, "1aaf5bcfd9d2282b"),
    (0x021B1334, "7d9695cd1824b3e2"),
    (0x021B13E0, "428fc0e667d0ac62"),
    (0x021B13EC, "2b08d9a06ba76588"),
    (0x021B13F8, "313b90924a910e52"),
    (0x021B1424, "f3aac64a58027a39"),
    (0x021B143C, "169472f89ecf83b8"),
    (0x021B14DC, "bd15a9d680573c6d"),
    (0x021B14E8, "86a715694c376a01"),
    (0x021B14F4, "62f124194938011c"),
    (0x021B1550, "d64c69725ee2986d"),
    (0x021B15B8, "36aea09a2fe85272"),
    (0x021B15C8, "3119f277153e12a0"),
    (0x021B1604, "f2ead4e0ab054119"),
    (0x021B162C, "5a3de7bce500dc44"),
];
// fndiff-recorded-end

/// The spec's stubs, with every argument buffer of the case in one memory window set (at the emulator's
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
impl TalkHooks for Rom<'_> {
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5CC, &[ctx])
    }
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A5EC, &[ctx])
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
    fn fun_0215a63c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A63C, &[ctx])
    }
    fn fun_0215a654(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A654, &[ctx])
    }
    fn fun_0215a660(&mut self, ctx: u32, v: u32) {
        self.s(0x0215_A660, &[ctx, v]);
    }
    fn fun_0215a66c(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A66C, &[ctx])
    }
    fn fun_0215a678(&mut self, ctx: u32, v: u32) {
        self.s(0x0215_A678, &[ctx, v]);
    }
    fn fun_0215a77c(
        &mut self,
        ctx: u32,
        x: u32,
        y: u32,
        cursor: u32,
        flag6: u32,
        right_align: u32,
        var: u32,
        wordset: u32,
        msgdata: u32,
    ) {
        self.s(
            0x0215_A77C,
            &[ctx, x, y, cursor, flag6, right_align, var, wordset, msgdata],
        );
    }
    fn fun_0215a7f0(&mut self, ctx: u32, msg: u32, help: u32, param: u32, s1: u32, s2: u32) {
        self.s(0x0215_A7F0, &[ctx, msg, help, param, s1, s2]);
    }
    fn fun_0215a864(&mut self, ctx: u32) {
        self.s(0x0215_A864, &[ctx]);
    }
    fn fun_0215a954(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A954, &[ctx])
    }
    fn fun_0215a990(&mut self, ctx: u32) -> u32 {
        self.s(0x0215_A990, &[ctx])
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
    fn fun_02158f80(&mut self, sw: u32, v: u32) {
        self.s(0x0215_8F80, &[sw, v]);
    }
    fn fun_02158f84(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F84, &[sw])
    }
    fn fun_02158f90(&mut self, sw: u32) -> u32 {
        self.s(0x0215_8F90, &[sw])
    }
    fn fun_02158fe0(&mut self, sw: u32, gamedata: u32, var: u32) -> u32 {
        self.s(0x0215_8FE0, &[sw, gamedata, var])
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
    fn fun_020056bc(&mut self, x: u32) -> u32 {
        self.s(0x0200_56BC, &[x])
    }
    fn fun_020061e4(&mut self, se: u32) {
        self.s(0x0200_61E4, &[se]);
    }
    fn fun_02008550(&mut self, x: u32) -> u32 {
        self.s(0x0200_8550, &[x])
    }
    fn fun_0200c274(&mut self, x: u32) -> u32 {
        self.s(0x0200_C274, &[x])
    }
    fn fun_0201210c(&mut self, gamesys: u32) -> u32 {
        self.s(0x0201_210C, &[gamesys])
    }
    fn fun_020127a4(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_27A4, &[gamedata])
    }
    fn fun_02012f08(&mut self, gamedata: u32) -> u32 {
        self.s(0x0201_2F08, &[gamedata])
    }
    fn fun_0201d7ec(&mut self, a: u32, b: u32, c: u32, d: u32, heap: u32) -> u32 {
        self.s(0x0201_D7EC, &[a, b, c, d, heap])
    }
    fn fun_0201d83c(&mut self, x: u32) {
        self.s(0x0201_D83C, &[x]);
    }
    fn fun_0201efa4(&mut self, wordset: u32, slot: u32, value: u32, digits: u32, a: u32, b: u32) {
        self.s(0x0201_EFA4, &[wordset, slot, value, digits, a, b]);
    }
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32) {
        self.s(0x0201_F250, &[wordset, dst, src]);
    }
    fn fun_0202a310(&mut self, a: u32, b: u32, strbuf: u32, heap: u32) {
        self.s(0x0202_A310, &[a, b, strbuf, heap]);
    }
    fn fun_0202ee30(&mut self, res: u32, window: u32, a: u32, b: u32, heap: u32) -> u32 {
        self.s(0x0202_EE30, &[res, window, a, b, heap])
    }
    fn fun_0202f038(&mut self, obj: u32) {
        self.s(0x0202_F038, &[obj]);
    }
    fn fun_020362dc(&mut self) -> u32 {
        self.s(0x0203_62DC, &[])
    }
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32 {
        self.s(0x0204_57B0, &[len, heap])
    }
    fn fun_02045808(&mut self, strbuf: u32) {
        self.s(0x0204_5808, &[strbuf]);
    }
    fn fun_02045b38(&mut self, a: u32, ty: u32, msg: u32, heap: u32) -> u32 {
        self.s(0x0204_5B38, &[a, ty, msg, heap])
    }
    fn fun_02045c04(&mut self, msgdata: u32) {
        self.s(0x0204_5C04, &[msgdata]);
    }
    fn fun_02045cac(&mut self, msgdata: u32, msg: u32, strbuf: u32) {
        self.s(0x0204_5CAC, &[msgdata, msg, strbuf]);
    }
    fn fun_02048ad0(&mut self, camera: u32) {
        self.s(0x0204_8AD0, &[camera]);
    }
    fn fun_02048ae8(&mut self, camera: u32, out: &mut [u8; 12]) {
        self.stubs
            .call(0x0204_8AE8, &mut [Arg::V(camera), Arg::Out(&mut out[..])]);
    }
    fn fun_02048b0c(&mut self, camera: u32, out: &mut [u8; 12]) {
        self.stubs
            .call(0x0204_8B0C, &mut [Arg::V(camera), Arg::Out(&mut out[..])]);
    }
    fn fun_02048b30(&mut self, camera: u32, out: &mut [u8; 12]) {
        self.stubs
            .call(0x0204_8B30, &mut [Arg::V(camera), Arg::Out(&mut out[..])]);
    }
    fn fun_0206df3c(&mut self, pos: &[u8; 12], y: &mut [u8; 4], x: &mut [u8; 4]) {
        self.stubs.call(
            0x0206_DF3C,
            &mut [
                Arg::In(&pos[..]),
                Arg::Out(&mut y[..]),
                Arg::Out(&mut x[..]),
            ],
        );
    }
    fn fun_0207b0a4(&mut self, m: &[u8; 0x30], out: &mut [u8; 0x30]) {
        self.stubs
            .call(0x0207_B0A4, &mut [Arg::In(&m[..]), Arg::Out(&mut out[..])]);
    }
    fn fun_0207b51c(&mut self, v: &[u8; 12], m: &[u8; 0x30], out: &mut [u8; 12]) {
        self.stubs.call(
            0x0207_B51C,
            &mut [Arg::In(&v[..]), Arg::In(&m[..]), Arg::Out(&mut out[..])],
        );
    }
    fn fun_0207bca0(&mut self, a: &[u8; 12], b: &[u8; 12], c: &[u8; 12], out: &mut [u8; 0x30]) {
        self.stubs.call(
            0x0207_BCA0,
            &mut [
                Arg::In(&a[..]),
                Arg::In(&b[..]),
                Arg::In(&c[..]),
                Arg::Out(&mut out[..]),
            ],
        );
    }
    fn fun_0207ca00(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]) {
        self.stubs.call(
            0x0207_CA00,
            &mut [Arg::In(&a[..]), Arg::In(&b[..]), Arg::Out(&mut out[..])],
        );
    }
    fn fun_0207cae8(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]) {
        self.stubs.call(
            0x0207_CAE8,
            &mut [Arg::In(&a[..]), Arg::In(&b[..]), Arg::Out(&mut out[..])],
        );
    }
    fn fun_0207cca4(&mut self, a: &[u8; 12], out: &mut [u8; 12]) {
        self.stubs
            .call(0x0207_CCA4, &mut [Arg::In(&a[..]), Arg::Out(&mut out[..])]);
    }
    fn fun_02082b60(&mut self, a: &[u8; 0x30], out: &mut [u8; 0x30]) {
        self.stubs
            .call(0x0208_2B60, &mut [Arg::In(&a[..]), Arg::Out(&mut out[..])]);
    }
    fn fun_0216d588(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D588, &[mmdl])
    }
    fn fun_0216d5f8(&mut self, mmdl: u32) -> u32 {
        self.s(0x0216_D5F8, &[mmdl])
    }
    fn fun_0216d88c(&mut self, mmdl: u32, out: &mut [u8; 12]) {
        self.stubs
            .call(0x0216_D88C, &mut [Arg::V(mmdl), Arg::Out(&mut out[..])]);
    }
    fn fun_0216de94(&mut self, mmdls: u32, obj: u32) -> u32 {
        self.s(0x0216_DE94, &[mmdls, obj])
    }
    fn fun_02188c54(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C54, &[fm])
    }
    fn fun_02188c58(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C58, &[fm])
    }
    fn fun_02188c84(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8C84, &[fm])
    }
    fn fun_02188cb4(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8CB4, &[fm])
    }
    fn fun_02188d48(&mut self, fm: u32) -> u32 {
        self.s(0x0218_8D48, &[fm])
    }
    fn fun_02188d4c(&mut self, fm: u32, board: u32) {
        self.s(0x0218_8D4C, &[fm, board]);
    }
    fn fun_0218ec2c(&mut self, camera: u32) -> u32 {
        self.s(0x0218_EC2C, &[camera])
    }
    fn fun_021904dc(&mut self, msgbg: u32, on: u32) {
        self.s(0x0219_04DC, &[msgbg, on]);
    }
    fn fun_02190708(&mut self, msgbg: u32, msgdata: u32, x: u32, y: u32, w: u32, h: u32) -> u32 {
        self.s(0x0219_0708, &[msgbg, msgdata, x, y, w, h])
    }
    fn fun_02190728(&mut self, board: u32) {
        self.s(0x0219_0728, &[board]);
    }
    fn fun_02190758(&mut self, board: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_0758, &[board, x, y, text]);
    }
    fn fun_02190788(&mut self, board: u32) {
        self.s(0x0219_0788, &[board]);
    }
    fn fun_021909bc(&mut self, menu: u32) {
        self.s(0x0219_09BC, &[menu]);
    }
    fn fun_02190be4(&mut self, msgbg: u32, arg: u32) -> u32 {
        self.s(0x0219_0BE4, &[msgbg, arg])
    }
    fn fun_02190c78(&mut self, menu: u32) -> u32 {
        self.s(0x0219_0C78, &[menu])
    }
    fn fun_02190fa0(&mut self, msgbg: u32, msgdata: u32, style: u32) -> u32 {
        self.s(0x0219_0FA0, &[msgbg, msgdata, style])
    }
    fn fun_02191008(&mut self, window: u32) {
        self.s(0x0219_1008, &[window]);
    }
    fn fun_02191084(&mut self, window: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_1084, &[window, x, y, text]);
    }
    fn fun_021910c0(&mut self, window: u32) -> u32 {
        self.s(0x0219_10C0, &[window])
    }
    fn fun_02191134(&mut self, window: u32) {
        self.s(0x0219_1134, &[window]);
    }
    fn fun_02191164(&mut self, window: u32) {
        self.s(0x0219_1164, &[window]);
    }
    fn fun_02191184(&mut self, window: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_1184, &[window, x, y, text]);
    }
    fn fun_021911b4(&mut self, window: u32) -> u32 {
        self.s(0x0219_11B4, &[window])
    }
    fn fun_0219129c(&mut self, msgbg: u32, tail: u32, pos: u32, text: u32, a: u32, b: u32) -> u32 {
        self.s(0x0219_129C, &[msgbg, tail, pos, text, a, b])
    }
    fn fun_021912d8(&mut self, window: u32) {
        self.s(0x0219_12D8, &[window]);
    }
    fn fun_021912f8(&mut self, window: u32) -> u32 {
        self.s(0x0219_12F8, &[window])
    }
    fn fun_0219131c(&mut self, window: u32) {
        self.s(0x0219_131C, &[window]);
    }
    fn fun_02191348(&mut self, window: u32, text: u32) {
        self.s(0x0219_1348, &[window, text]);
    }
    fn fun_02191388(&mut self, window: u32) -> u32 {
        self.s(0x0219_1388, &[window])
    }
    fn fun_021914cc(&mut self, window: u32) {
        self.s(0x0219_14CC, &[window]);
    }
    fn fun_02191558(
        &mut self,
        msgbg: u32,
        a: u32,
        kind: u32,
        b: u32,
        style: u32,
        c: u32,
        d: u32,
    ) -> u32 {
        self.s(0x0219_1558, &[msgbg, a, kind, b, style, c, d])
    }
    fn fun_021915b4(&mut self, window: u32) {
        self.s(0x0219_15B4, &[window]);
    }
    fn fun_021915ec(&mut self, window: u32) {
        self.s(0x0219_15EC, &[window]);
    }
    fn fun_021916a8(&mut self, window: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_16A8, &[window, x, y, text]);
    }
    fn fun_021916e0(&mut self, window: u32) -> u32 {
        self.s(0x0219_16E0, &[window])
    }
    fn fun_02191794(&mut self, window: u32) {
        self.s(0x0219_1794, &[window]);
    }
    fn fun_02191830(&mut self, msgbg: u32, text: u32, w: &mut u8, h: &mut u8) {
        self.stubs.call(
            0x0219_1830,
            &mut [
                Arg::V(msgbg),
                Arg::V(text),
                Arg::Out(std::slice::from_mut(w)),
                Arg::Out(std::slice::from_mut(h)),
            ],
        );
    }
    fn fun_021918dc(&mut self, msgbg: u32, text: u32, a: u32, x: u32, y: u32, w: u32, h: u32) {
        self.s(0x0219_18DC, &[msgbg, text, a, x, y, w, h]);
    }
    fn fun_0219196c(&mut self, msgbg: u32, v: u32) {
        self.s(0x0219_196C, &[msgbg, v]);
    }
    fn fun_02191978(&mut self, msgbg: u32) {
        self.s(0x0219_1978, &[msgbg]);
    }
    fn fun_021919ac(&mut self, msgbg: u32) -> u32 {
        self.s(0x0219_19AC, &[msgbg])
    }
    fn fun_02191a04(&mut self, msgbg: u32, a: u32) -> u32 {
        self.s(0x0219_1A04, &[msgbg, a])
    }
    fn fun_02191a10(&mut self, msgbg: u32, a: u32, x: u32) -> u32 {
        self.s(0x0219_1A10, &[msgbg, a, x])
    }
    fn fun_02191b14(&mut self, window: u32) {
        self.s(0x0219_1B14, &[window]);
    }
    fn fun_02191be8(&mut self, window: u32, text: u32) -> u32 {
        self.s(0x0219_1BE8, &[window, text])
    }
    fn fun_02192570(&mut self, msgbg: u32, a: u32, x: u32, y: u32, w: u32, h: u32) -> u32 {
        self.s(0x0219_2570, &[msgbg, a, x, y, w, h])
    }
    fn fun_02192628(&mut self, window: u32) {
        self.s(0x0219_2628, &[window]);
    }
    fn fun_0219269c(&mut self, window: u32, x: u32, y: u32, text: u32) {
        self.s(0x0219_269C, &[window, x, y, text]);
    }
    fn fun_021926d8(&mut self, window: u32) -> u32 {
        self.s(0x0219_26D8, &[window])
    }
    fn fun_0219270c(&mut self, msgbg: u32, text: u32, a: u32) -> u32 {
        self.s(0x0219_270C, &[msgbg, text, a])
    }
    fn fun_0219272c(&mut self, msgbg: u32, text: u32, a: u32) -> u32 {
        self.s(0x0219_272C, &[msgbg, text, a])
    }
    fn fun_021a3074(&mut self, player: u32) -> u32 {
        self.s(0x021A_3074, &[player])
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
        0x021B_015C => menu_event(h, v[0], v[1], v[2]) as u64,
        0x021B_010C => yes_no_event_new(h, v[0], v[1], v[2]) as u64,
        0x021B_01B8 => menu_event_new(h, v[0], v[1], v[2]) as u64,
        0x021B_03E4 => {
            talk_open(h, v[0], v[1]);
            0
        }
        0x021B_0480 => {
            talk_close(h, v[0]);
            0
        }
        0x021B_0808 => {
            balloon_project(h, v[0], v[1], v[2]);
            0
        }
        0x021B_090C => {
            balloon_to_view(h, v[0], v[1]);
            0
        }
        0x021B_0960 => {
            balloon_place(h, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021B_09D8 => {
            balloon_place_veneer(h, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021B_09EC => {
            balloon_tail(h, v[0], v[1], v[2]);
            0
        }
        0x021B_0A40 => {
            let mut pos = [0u8; 12];
            for i in 0..3 {
                let w = h.r32(v[1] + 4 * i as u32);
                pos[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
            }
            balloon_side(h, v[0], &pos) as u64
        }
        0x021B_0A88 => {
            balloon_close(h, v[0], v[1]);
            0
        }
        0x021B_0AB0 => {
            balloon_delete(h, v[0]);
            0
        }
        0x021B_0ADC => {
            balloon_create(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021B_0B08 => {
            balloon_follow(h, v[0], v[1]);
            0
        }
        0x021B_0BAC => balloon_print(h, v[0], v[1], v[2], v[3], v[4], v[5]) as u64,
        0x021B_1008 => system_message(h, v[0], v[1], v[2]) as u64,
        0x021B_10A4 => {
            system_close(h, v[0]);
            0
        }
        0x021B_120C => {
            info_close(h, v[0]);
            0
        }
        0x021B_1268 => {
            window_67_close(h, v[0]);
            0
        }
        0x021B_13F8 => {
            window_69_close(h, v[0]);
            0
        }
        0x021B_162C => message_text(h, v[0], v[1], v[2]) as u64,
        0x021B_0130 => cmd_71(h, v[0], v[1]) as u64,
        0x021B_01DC => cmd_173(h, v[0], v[1]) as u64,
        0x021B_024C => cmd_174(h, v[0], v[1]) as u64,
        0x021B_02C8 => cmd_178(h, v[0], v[1]) as u64,
        0x021B_0344 => cmd_175(h, v[0], v[1]) as u64,
        0x021B_0394 => cmd_176(h, v[0], v[1]) as u64,
        0x021B_03BC => cmd_177(h, v[0], v[1]) as u64,
        0x021B_04C0 => wait_talk(h, v[0], v[1]) as u64,
        0x021B_04D8 => cmd_52(h, v[0], v[1]) as u64,
        0x021B_0530 => cmd_53(h, v[0], v[1]) as u64,
        0x021B_05AC => cmd_55(h, v[0], v[1]) as u64,
        0x021B_0624 => cmd_54(h, v[0], v[1]) as u64,
        0x021B_063C => cmd_64(h, v[0], v[1]) as u64,
        0x021B_0730 => cmd_66(h, v[0], v[1]) as u64,
        0x021B_07E0 => cmd_65(h, v[0], v[1]) as u64,
        0x021B_0A60 => wait_balloon_close(h, v[0], v[1]) as u64,
        0x021B_0CEC => wait_balloon(h, v[0], v[1]) as u64,
        0x021B_0D14 => cmd_60(h, v[0], v[1]) as u64,
        0x021B_0D74 => cmd_61(h, v[0], v[1]) as u64,
        0x021B_0DF0 => cmd_72(h, v[0], v[1]) as u64,
        0x021B_0E74 => cmd_73(h, v[0], v[1]) as u64,
        0x021B_0EDC => cmd_62(h, v[0], v[1]) as u64,
        0x021B_0EF4 => cmd_135(h, v[0], v[1]) as u64,
        0x021B_0FF0 => wait_system(h, v[0], v[1]) as u64,
        0x021B_10D0 => cmd_56(h, v[0], v[1]) as u64,
        0x021B_10DC => cmd_74(h, v[0], v[1]) as u64,
        0x021B_10E8 => cmd_57(h, v[0], v[1]) as u64,
        0x021B_1100 => cmd_58(h, v[0], v[1]) as u64,
        0x021B_11D8 => cmd_59(h, v[0], v[1]) as u64,
        0x021B_1240 => wait_window_67(h, v[0], v[1]) as u64,
        0x021B_12B0 => cmd_67(h, v[0], v[1]) as u64,
        0x021B_1334 => cmd_579(h, v[0], v[1]) as u64,
        0x021B_13E0 => cmd_68(h, v[0], v[1]) as u64,
        0x021B_1424 => wait_window_69(h, v[0], v[1]) as u64,
        0x021B_143C => cmd_69(h, v[0], v[1]) as u64,
        0x021B_14DC => cmd_70(h, v[0], v[1]) as u64,
        0x021B_14F4 => cmd_63(h, v[0], v[1]) as u64,
        0x021B_1550 => wait_button(h, v[0], v[1]) as u64,
        0x021B_15B8 => cmd_75(h, v[0], v[1]) as u64,
        0x021B_15C8 => cmd_51(h, v[0], v[1]) as u64,
        0x021B_0630 => init_talk(h, v[0], v[1]) as u64,
        0x021B_0EE8 => init_balloon(h, v[0], v[1]) as u64,
        0x021B_10F4 => init_system(h, v[0], v[1]) as u64,
        0x021B_1234 => init_info(h, v[0], v[1]) as u64,
        0x021B_13EC => init_window_67(h, v[0], v[1]) as u64,
        0x021B_14E8 => init_window_69(h, v[0], v[1]) as u64,
        0x021B_1604 => init_msgbg_flag(h, v[0], v[1]) as u64,
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

/// `$BLACK_ROM_BUILD/OVY_<n>.sbin`, or `None` (skip) when it is missing and `REQUIRE_ROM` is not set.
fn overlay(n: u32) -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/OVY_{n}.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/OVY_{n}.sbin is missing");
    }
    got
}

/// The command ids of [`COMMANDS`] point at these handlers in the ROM command table (0x0217062C, overlay 10), every handler
/// is in [`PORTED`], and the init procs [`call_init_proc`] routes are the ROM's table entries.
#[test]
fn commands_match_rom() {
    for &(_, addr) in COMMANDS {
        assert!(PORTED.contains(&addr), "{addr:#x}");
    }
    for slot in [1, 2, 3, 4, 5, 6, 9] {
        let f = crate::script_sys::INIT_PROCS[slot];
        assert!(PORTED.contains(&(f & !1)), "slot {slot}");
    }
    let Some(ovy) = overlay(10) else {
        return;
    };
    for &(id, addr) in COMMANDS {
        let o = (crate::script_sys::CMD_TABLE - 0x0215_5160) as usize + 4 * id as usize;
        let w = u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap());
        assert_eq!(w, addr | 1, "command {id}");
    }
}

/// The embedded [`UP`] / [`OFFSETS`] vectors are the overlay-21 data at 0x021D9390 / 0x021D939C (overlay 21 is loaded at
/// 0x02187F00).
#[test]
fn tables_match_rom() {
    let Some(ovy) = overlay(21) else {
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0218_7F00) as usize;
        u32::from_le_bytes(ovy[o..o + 4].try_into().unwrap())
    };
    for (i, &w) in UP.iter().enumerate() {
        assert_eq!(word(UP_ADDR + 4 * i as u32), w);
    }
    for (c, col) in OFFSETS.iter().enumerate() {
        for (r, rec) in col.iter().enumerate() {
            for (i, &w) in rec.iter().enumerate() {
                let a = OFFSETS_ADDR + 0x30 * c as u32 + 0xC * r as u32 + 4 * i as u32;
                assert_eq!(word(a), w, "col {c} row {r} word {i}");
            }
        }
    }
}
