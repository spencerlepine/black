use super::*;
use crate::hooks::{GameDataHooks, HeapHooks, Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02011F24, "35c8d2a79674e525"),
    (0x02011F28, "fd78b4ef9438f800"),
    (0x0201205C, "01386a274170c8d3"),
    (0x0201209C, "6b252e8667c791b7"),
    (0x020120A4, "5a6fe946d2f072e4"),
    (0x020120B4, "da50702494654c0e"),
    (0x020120C0, "cb4c0a105582aafe"),
    (0x020120D8, "6d376803b0d3eb5e"),
    (0x020120F0, "cc578368b7f9748a"),
    (0x020120F4, "edc0d1eb777e31df"),
    (0x020120F8, "379c050c39e98c5a"),
    (0x02012104, "6843563cda7c6972"),
    (0x02012108, "b673f5ac6b9b52c4"),
    (0x0201210C, "68837066544a80e7"),
    (0x02012110, "f3caff3e2bb3b739"),
    (0x02012114, "d8368fd4cdea8153"),
    (0x02012124, "6de9b7ed1d979ec0"),
    (0x0201212C, "9d9b30b0d163e829"),
    (0x02012134, "de0e5535f18e5795"),
    (0x02012138, "fae8534de5f376ec"),
    (0x0201213C, "ec14b117a3d3aef2"),
    (0x02012144, "d3362c6529166d5d"),
    (0x0201214C, "2de72ce98b720937"),
    (0x02012158, "5ee32229496b97be"),
    (0x02012164, "ec81f188fa4376ae"),
    (0x020121CC, "7c86c4d5dce4b13b"),
    (0x02012204, "bb01da9c89aa057e"),
    (0x02012210, "4ba8f9c147b9127a"),
    (0x02012220, "01e9ec51dd3c86a0"),
    (0x02012248, "fda83eb8f40fc3cf"),
    (0x02012258, "a5812ccea78884cd"),
    (0x02012264, "f1bd30e77e90fce0"),
    (0x020122A4, "5ab4985931752c0e"),
    (0x020122B4, "6162e7fdd02411bd"),
    (0x02012314, "3259cf1d5233b21b"),
    (0x02012374, "6511feba0d6b1cb4"),
    (0x020123C4, "c2c08b40f866f024"),
    (0x020124E4, "33ce4480118f25a5"),
    (0x020124E8, "ae3cd8342944fbda"),
    (0x020124EC, "654949c6656f1175"),
    (0x020124F0, "e1d2e40c32b1b664"),
    (0x02012798, "dc0ab716b30125d5"),
    (0x020127A4, "bcee001249fbabf2"),
    (0x020127AC, "e7c97de8af4a10a9"),
    (0x020127B8, "b3420cac3d606975"),
    (0x020127C4, "6fe8a59c8edf533d"),
    (0x020127D0, "5bb7d78d472e5931"),
    (0x020127E0, "6da8fcc28be384b7"),
    (0x020127F0, "482536990e254e01"),
    (0x02012800, "218a34708910fb1d"),
    (0x0201281C, "02ef92e22b4ef9ae"),
    (0x0201283C, "a0f5f19b224738de"),
    (0x02012844, "83019eba7eb24250"),
    (0x0201284C, "d848fee79681f67f"),
    (0x02012854, "a1b6fe3b745cdefe"),
    (0x02012870, "d7ee2cea0583da2a"),
    (0x02012878, "d711dac777142a9c"),
    (0x02012894, "d6c848fdd8c4945a"),
    (0x0201289C, "cc2a89b09d9ff46e"),
    (0x020128B8, "0d2747712a66d032"),
    (0x020128C0, "74c70c9bd4827583"),
    (0x020128DC, "a29b88ee980d9afd"),
    (0x020128E4, "31bf077105494acd"),
    (0x02012900, "014a770c2e83afeb"),
    (0x02012914, "5f8603d2c9bb4562"),
    (0x02012924, "c912e069807b0bc3"),
    (0x0201292C, "66786aa4ad1b249c"),
    (0x02012934, "d2d496444e3bb2cd"),
    (0x0201293C, "c94cf5aff5c74f6d"),
    (0x02012944, "4a1f1a12131929fe"),
    (0x02012950, "b5df8fba188e0f09"),
    (0x0201295C, "d661341e5fae4816"),
    (0x02012964, "7ebc96f0e56ec389"),
    (0x0201296C, "3d98bb36a5ef124f"),
    (0x02012974, "0d88654e157e5241"),
    (0x0201297C, "51b6c93f8b7184bb"),
    (0x02012984, "6900e0bed6c8a624"),
    (0x0201298C, "c1435c070bd5452b"),
    (0x02012994, "4af9f858b9ee77ca"),
    (0x020129A0, "32ad5f23fcf6cdf1"),
    (0x020129AC, "37fad9932b928b8a"),
    (0x020129B8, "2ffe860e197ecc0e"),
    (0x020129C4, "9b1d6897a89c826b"),
    (0x020129D0, "9f0227e57fa6df46"),
    (0x020129DC, "a2745e8973d62103"),
    (0x020129F0, "caa97572d65b5488"),
    (0x020129FC, "04563899527c0809"),
    (0x02012A24, "577101082a681dd5"),
    (0x02012A30, "f56ed99c291b6a04"),
    (0x02012A38, "9c919f312a3cee72"),
    (0x02012A44, "ebd3db1e26e1ba2d"),
    (0x02012A4C, "f594decb78057b9c"),
    (0x02012A54, "4d2d32db73f915bf"),
    (0x02012A60, "09e4a1ad046b3541"),
    (0x02012A90, "17b1d23b3e8fcdb2"),
    (0x02012AA0, "b42d7b16f7907a36"),
    (0x02012AA4, "494826522734507e"),
    (0x02012AB8, "632b71e90851088a"),
    (0x02012ABC, "fcf34ca99390787d"),
    (0x02012AC0, "48ac6fd410b008a2"),
    (0x02012AC4, "094ef825cb7435e5"),
    (0x02012AC8, "b923651969293154"),
    (0x02012ACC, "cd172bbcfdc29ff9"),
    (0x02012AD0, "3b565d3e8207879a"),
    (0x02012AD4, "8456a8ecd90f18e0"),
    (0x02012AE4, "9b7e8d2a610cab38"),
    (0x02012B28, "0e79ff3aa9ebf532"),
    (0x02012B2C, "72afddf547e6939b"),
    (0x02012B30, "bb27543735b2901c"),
    (0x02012B34, "49d488e60b93852f"),
    (0x02012B38, "68963c2717b15b39"),
    (0x02012B40, "5769617da50cb1fd"),
    (0x02012B4C, "e8387d874490cdcf"),
    (0x02012B58, "f21f677cec4a9c06"),
    (0x02012B64, "ccf34b523f7b6f64"),
    (0x02012B70, "2d220311fa2c6a36"),
    (0x02012B78, "164554d21dda5109"),
    (0x02012B80, "a104bb8433cbd95b"),
    (0x02012B8C, "23d87d2a7b1b1d0c"),
    (0x02012B98, "9fb8e9c9495dade6"),
    (0x02012BA4, "a6b768ab23217997"),
    (0x02012BB0, "a00360778b4287df"),
    (0x02012BC8, "42f61ace9e5d9b12"),
    (0x02012BE0, "2996b3d2b5fe22d6"),
    (0x02012BF8, "efa28d28855574fb"),
    (0x02012C0C, "9596e702fa503500"),
    (0x02012E40, "53e0b53436ce75da"),
    (0x02012EBC, "777b77dcdf02b665"),
    (0x02012EC0, "1af8bb214c59da04"),
    (0x02012EC4, "f8c5c2213c76abe5"),
    (0x02012ED0, "174b2a4b3c676927"),
    (0x02012ED8, "88cd93999121f910"),
    (0x02012EE4, "c03490b51d5e9908"),
    (0x02012EF0, "6294ad99cde642be"),
    (0x02012EFC, "b5ffd25d8e190de3"),
    (0x02012F08, "4e7a1af6a043e9da"),
    (0x02012F14, "51ca93ee241aa164"),
    (0x02012F20, "2c4043efa1412dd9"),
    (0x02012F2C, "c66a7314ec9b3616"),
    (0x02012F38, "11ca1e05c1cb22b8"),
    (0x02012F44, "66d901c40fde037d"),
    (0x02012F50, "0738cc47353ce557"),
    (0x02012F5C, "2895c8f67f25aabf"),
    (0x02012F68, "51632fac8b2d10f7"),
    (0x02012F74, "a8efb477f019256d"),
    (0x02012F80, "93d9833b74425f25"),
    (0x02012F8C, "a150f70f03733aab"),
    (0x02012F98, "e842154d7e557ad1"),
    (0x02012FA4, "0321ff178b84b0c9"),
    (0x02012FB0, "bf3a1d4d3d165345"),
    (0x02012FBC, "0e1b6cb3f83701d4"),
    (0x02012FCC, "e4c67ff44584467c"),
    (0x02012FD8, "7796c602a22fccb2"),
    (0x02012FE4, "111f674e5ef7cb6f"),
    (0x02012FF0, "9fe0cb3d57f8c84b"),
    (0x02012FFC, "1d8c2c5a2840ca38"),
    (0x02013020, "f801edde5f85bc13"),
    (0x02013050, "5d24abc1098a5807"),
    (0x02013080, "9f977d5875a4773f"),
];
// fndiff-recorded-end

/// The unported callees of the spec as stubs, with every buffer, region and global of the case in
/// one memory window set (at the addresses the emulator uses).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    mem: Mem,
}

impl MemEnv for Rom<'_> {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

impl HeapHooks for Rom<'_> {
    fn heap_alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32 {
        self.stubs
            .call(0x0203_0734, &mut [heap, size, flag, file, line].map(Arg::V))
    }
    fn heap_free(&mut self, block: u32) {
        self.stubs.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
}

impl GameDataHooks for Rom<'_> {
    fn fun_02031324(&mut self, heap: u32) -> u32 {
        self.stubs.call(0x0203_1324, &mut [Arg::V(heap)])
    }
    fn fun_020124f4(&mut self, heap: u32) -> u32 {
        self.stubs.call(0x0201_24f4, &mut [Arg::V(heap)])
    }
    fn fun_020261e4(&mut self, heap: u32, _gd: u32) -> u32 {
        // the spec's stub logs r0 only (`args=v`)
        self.stubs.call(0x0202_61e4, &mut [Arg::V(heap)])
    }
    fn fun_0202b4bc(&mut self, gd: u32, heap: u32) -> u32 {
        self.stubs
            .call(0x0202_b4bc, &mut [Arg::V(gd), Arg::V(heap)])
    }
    fn fun_02035138(&mut self, proc_: u32, arg: u32) {
        self.stubs
            .call(0x0203_5138, &mut [Arg::V(proc_), Arg::V(arg)]);
    }
    fn fun_02035064(&mut self, proc_: u32, arg: u32) {
        self.stubs
            .call(0x0203_5064, &mut [Arg::V(proc_), Arg::V(arg)]);
    }
    fn fun_020139b0(&mut self, heap: u32) {
        self.stubs.call(0x0201_39b0, &mut [Arg::V(heap)]);
    }
    fn fun_02031348(&mut self, sched: u32, a: u32, b: u32, c: u32) -> u32 {
        self.stubs.call(
            0x0203_1348,
            &mut [Arg::V(sched), Arg::V(a), Arg::V(b), Arg::V(c)],
        )
    }
    fn fun_02008448(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200_8448, &mut [Arg::V(x)])
    }
    fn fun_02008450(&mut self, x: u32, flag: u32) {
        self.stubs.call(0x0200_8450, &mut [Arg::V(x), Arg::V(flag)]);
    }
    fn fun_02013d34(&mut self, id: u32) -> u32 {
        self.stubs.call(0x0201_3d34, &mut [Arg::V(id)])
    }
    fn fun_0200fb24(&mut self) -> u32 {
        self.stubs.call(0x0200_fb24, &mut [])
    }
    fn fun_02158318(&mut self, sys: u32) -> u32 {
        self.stubs.call(0x0215_8318, &mut [Arg::V(sys)])
    }
    fn fun_02035390(&mut self) -> u32 {
        self.stubs.call(0x0203_5390, &mut [])
    }
    fn fun_020353b0(&mut self) -> u32 {
        self.stubs.call(0x0203_53b0, &mut [])
    }
    fn fun_0200921c(&mut self, gd: u32) {
        self.stubs.call(0x0200_921c, &mut [Arg::V(gd)]);
    }
    fn fun_020090e0(&mut self) -> u32 {
        self.stubs.call(0x0200_90e0, &mut [])
    }
    fn fun_0203cd98(&mut self) -> u32 {
        self.stubs.call(0x0203_cd98, &mut [])
    }
    fn fun_02026454(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0202_6454, &mut [Arg::V(x)])
    }
    fn fun_020263d8(&mut self, x: u32, a: u32, sys: u32) {
        self.stubs
            .call(0x0202_63d8, &mut [Arg::V(x), Arg::V(a), Arg::V(sys)]);
    }
    fn fun_02188be8(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0218_8be8, &mut [Arg::V(x)])
    }
    fn fun_0203da4c(&mut self) -> u32 {
        self.stubs.call(0x0203_da4c, &mut [])
    }
    fn fun_0200a864(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_a864, &mut [Arg::V(save)])
    }
    fn fun_0200a890(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200_a890, &mut [Arg::V(x)])
    }
    fn fun_0200a898(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200_a898, &mut [Arg::V(x)])
    }
    fn fun_02161d64(&mut self, record: u32) {
        self.stubs.call(0x0216_1d64, &mut [Arg::V(record)]);
    }
    fn fun_02008850(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200_8850, &mut [Arg::V(x)])
    }
    fn fun_0200c40c(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_c40c, &mut [Arg::V(save)])
    }
    fn fun_0200c418(&mut self, x: u32, a: u32, b: u32) {
        self.stubs
            .call(0x0200_c418, &mut [Arg::V(x), Arg::V(a), Arg::V(b)]);
    }
    fn fun_0200c404(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_c404, &mut [Arg::V(save)])
    }
    fn fun_0200c44c(&mut self, x: u32, a: u32) -> u32 {
        self.stubs.call(0x0200_c44c, &mut [Arg::V(x), Arg::V(a)])
    }
    fn fun_020071cc(&mut self, save: u32, block: u32) -> u32 {
        self.stubs
            .call(0x0200_71cc, &mut [Arg::V(save), Arg::V(block)])
    }
    fn fun_0200c0f0(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_c0f0, &mut [Arg::V(save)])
    }
    fn fun_02009ec8(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_9ec8, &mut [Arg::V(save)])
    }
    fn fun_020089c0(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_89c0, &mut [Arg::V(save)])
    }
    fn fun_02009448(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_9448, &mut [Arg::V(save)])
    }
    fn fun_02007b44(&mut self, save: u32) -> u32 {
        self.stubs.call(0x0200_7b44, &mut [Arg::V(save)])
    }
    // the spec records r0 (r1 passes through)
    fn fun_020085d4(&mut self, x: u32, _b: u32) -> u32 {
        self.stubs.call(0x0200_85d4, &mut [Arg::V(x)])
    }
    fn fun_0200873c(&mut self, handle: u32) -> u32 {
        self.stubs.call(0x0200_873c, &mut [Arg::V(handle)])
    }
    fn fun_02008374(&mut self, x: u32) -> u32 {
        self.stubs.call(0x0200_8374, &mut [Arg::V(x)])
    }
}

/// Run `f` on the case's memory: buffers at their argument addresses, regions at
/// `0x02F10000 + k * 0x10000`, the case's globals at `globals`.
fn with_rom(c: &mut Case, globals: &[u32], f: impl FnOnce(&mut Rom, &[u32]) -> u64) -> u64 {
    let vals = c.vals.clone();
    let mut mem = Mem::default();
    let mut keep = Vec::new();
    for (i, b) in c.bufs.iter_mut().enumerate() {
        if !b.is_empty() {
            mem.add_window(vals[i], std::mem::take(b));
            keep.push((0, i));
        }
    }
    for (k, r) in c.regions.iter_mut().enumerate() {
        mem.add_window(0x02F1_0000 + 0x1_0000 * k as u32, std::mem::take(r));
        keep.push((1, k));
    }
    for (k, g) in c.globals.iter_mut().enumerate() {
        mem.add_window(globals[k], std::mem::take(g));
        keep.push((2, k));
    }
    let mut rom = Rom {
        stubs: &mut c.env,
        mem,
    };
    let r = f(&mut rom, &vals);
    let wins = std::mem::take(&mut rom.mem.extra);
    for ((kind, i), (_, bytes)) in keep.into_iter().zip(wins) {
        match kind {
            0 => c.bufs[i] = bytes,
            1 => c.regions[i] = bytes,
            _ => c.globals[i] = bytes,
        }
    }
    r
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_1F24 => with_rom(c, &[], |_h, _v| sys_size() as u64),
        0x0201_1F28 => with_rom(c, &[], |h, v| {
            sys_init(h, v[0], v[1], v[2]);
            0
        }),
        0x0201_209C => with_rom(c, &[], |h, v| {
            sys_set_callbacks(h, v[0], v[1], v[2]);
            0
        }),
        0x0201_20A4 => with_rom(c, &[], |h, v| {
            sys_cache_event_active(h, v[0]);
            0
        }),
        0x0201_20B4 => with_rom(c, &[], |h, v| {
            sys_call_02031348(h, v[0], v[1], v[2], v[3]) as u64
        }),
        0x0201_20C0 => with_rom(c, &[], |h, v| sys_call_021d359c(h, v[0]) as u64),
        0x0201_20D8 => with_rom(c, &[], |h, v| sys_event_active(h, v[0]) as u64),
        0x0201_23C4 => with_rom(c, &[], |h, v| event_queued(h, v[0]) as u64),
        0x0201_20F8 => with_rom(c, &[], |h, v| {
            sys_gamedata_block(h, v[0]).wrapping_sub(v[1]) as u64
        }),
        0x0201_2114 => with_rom(c, &[], |h, v| sys_has_20(h, v[0]) as u64),
        0x0201_214C => with_rom(c, &[], |h, v| sys_flag_198(h, v[0]) as u64),
        0x0201_2158 => with_rom(c, &[], |h, v| {
            sys_set_flag_198(h, v[0], v[1]);
            0
        }),
        0x0201_2164 => with_rom(c, &[], |h, v| sys_field_free(h, v[0]) as u64),
        0x0201_21CC => with_rom(c, &[], |h, v| sys_try_start(h, v[0]) as u64),
        0x0201_205C => with_rom(c, &[], |h, v| {
            sys_poll(h, v[0]);
            0
        }),
        0x0201_2204 => with_rom(c, &[], |h, v| sys_get_33_bit0(h, v[0]) as u64),
        0x0201_2210 => with_rom(c, &[], |h, v| {
            sys_clear_33_bit0(h, v[0]);
            0
        }),
        0x0201_2220 => with_rom(c, &[], |h, v| {
            sys_suspend(h, v[0]);
            0
        }),
        0x0201_2248 => with_rom(c, &[], |h, v| {
            sys_register_a(h, v[0]);
            0
        }),
        0x0201_2258 => with_rom(c, &[], |h, _v| {
            sys_unregister_a(h);
            0
        }),
        0x0201_2264 => with_rom(c, &[], |h, v| sys_busy(h, v[0]) as u64),
        0x0201_22A4 => with_rom(c, &[], |h, v| {
            sys_register_b(h, v[0]);
            0
        }),
        0x0201_22B4 => with_rom(c, &[], |h, _v| {
            sys_unregister_b(h);
            0
        }),
        0x0201_2314 => with_rom(c, &[], |h, v| {
            ev_free(h, v[0]);
            0
        }),
        0x0201_2374 => with_rom(c, &[], |h, v| {
            ev_publish(h, v[0], v[1]);
            0
        }),
        0x0201_2798 => with_rom(c, &[], |_h, v| {
            gd_player(v[0], v[1]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2950 => with_rom(c, &[], |_h, v| {
            gd_player_24(v[0], v[1]).wrapping_sub(v[0]) as u64
        }),
        0x0201_27AC => with_rom(c, &[], |_h, v| {
            gd_record_10(v[0], v[1]).wrapping_sub(v[0]) as u64
        }),
        0x0201_27B8 => with_rom(c, &[], |_h, v| {
            gd_record_10_3(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_27D0 => with_rom(c, &[], |_h, v| {
            gd_record_1e8(v[0], v[1]).wrapping_sub(v[0]) as u64
        }),
        0x0201_27C4 => with_rom(c, &[], |_h, v| {
            gd_record_1e8_1(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_27E0 => with_rom(c, &[], |h, v| gd_save_chain_a(h, v[0]) as u64),
        0x0201_27F0 => with_rom(c, &[], |h, v| gd_save_chain_b(h, v[0]) as u64),
        0x0201_2800 => with_rom(c, &[], |h, v| {
            gd_set_record_1e8_0(h, v[0], v[1]);
            0
        }),
        0x0201_281C => with_rom(c, &[], |h, v| {
            gd_clear_record_1e8_0(h, v[0]);
            0
        }),
        0x0201_28E4 => with_rom(c, &[], |h, v| {
            gd_copy_in_020128e4(h, v[0], v[1]);
            0
        }),
        0x0201_2900 => with_rom(c, &[], |h, v| {
            gd_set_halfword_15c(h, v[0], v[1] as u16);
            0
        }),
        0x0201_2914 => with_rom(c, &[], |h, v| gd_get_halfword_15c(h, v[0]) as u64),
        0x0201_2944 => with_rom(c, &[], |_h, v| {
            gd_addr_02012944(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_29DC => with_rom(c, &[], |h, v| {
            gd_set_020129dc(h, v[0], v[1] as u8);
            0
        }),
        0x0201_29F0 => with_rom(c, &[], |h, v| {
            gd_clear_1c8(h, v[0]);
            0
        }),
        0x0201_29FC => with_rom(c, &[], |h, v| gd_next_1c8(h, v[0]) as u64),
        0x0201_2A60 => with_rom(c, &[], |h, v| {
            pl_init(h, v[0]);
            0
        }),
        0x0201_2A90 => with_rom(c, &[], |h, v| {
            pl_copy_in_04(h, v[0], v[1]);
            0
        }),
        0x0201_2AA4 => with_rom(c, &[], |h, v| {
            pl_copy_in_10(h, v[0], v[1]);
            0
        }),
        0x0201_2AD4 => with_rom(c, &[], |h, v| {
            pl_set_18_from_table(h, v[0], v[1]);
            0
        }),
        0x0201_2AE4 => with_rom(c, &[], |h, v| pl_direction(h, v[0]) as u64),
        0x0201_2B98 => with_rom(c, &[], |h, v| gd_get_byte_1d0(h, v[0], v[1]) as u64),
        0x0201_2BA4 => with_rom(c, &[], |h, v| {
            gd_set_byte_1d0(h, v[0], v[1], v[2] as u8);
            0
        }),
        0x0201_2BB0 => with_rom(c, &[], |h, v| gd_flag_198(h, v[0]) as u64),
        0x0201_2BC8 => with_rom(c, &[], |h, v| {
            gd_set_flag_198(h, v[0], v[1]);
            0
        }),
        0x0201_2BE0 => with_rom(c, &[], |h, v| {
            gd_call_0200c418(h, v[0], v[1], v[2]);
            0
        }),
        0x0201_2BF8 => with_rom(c, &[], |h, v| gd_call_0200c44c(h, v[0], v[1]) as u64),
        0x0201_2FBC => with_rom(c, &[], |h, v| gd_call_020085d4(h, v[0], 0) as u64),
        0x0201_2FF0 => with_rom(c, &[0x020A7298], |h, v| {
            global_handle_set(h, v[0]);
            0
        }),
        0x0201_3080 => with_rom(c, &[], |_h, v| table_0209dfb0(v[0]) as u64),
        0x0201_2FFC => with_rom(c, &[0x020A7298], |h, _v| handle_table_direct(h) as u64),
        0x0201_3020 => with_rom(c, &[0x020A7298], |h, _v| handle_table_plus_one(h) as u64),
        0x0201_3050 => with_rom(c, &[0x020A7298], |h, _v| handle_table_rotated(h) as u64),
        0x0201_20F0 => with_rom(c, &[], |h, v| sys_get_0c(h, v[0]) as u64),
        0x0201_20F4 => with_rom(c, &[], |h, v| sys_get_gamedata(h, v[0]) as u64),
        0x0201_2104 => with_rom(c, &[], |h, v| sys_get_event(h, v[0]) as u64),
        0x0201_210C => with_rom(c, &[], |h, v| sys_get_20(h, v[0]) as u64),
        0x0201_2134 => with_rom(c, &[], |h, v| sys_get_2c(h, v[0]) as u64),
        0x0201_2138 => with_rom(c, &[], |h, v| sys_get_24(h, v[0]) as u64),
        0x0201_212C => with_rom(c, &[], |h, v| sys_get_30(h, v[0]) as u64),
        0x0201_2144 => with_rom(c, &[], |h, v| sys_get_32(h, v[0]) as u64),
        0x0201_2108 => with_rom(c, &[], |h, v| {
            sys_set_event(h, v[0], v[1]);
            0
        }),
        0x0201_2110 => with_rom(c, &[], |h, v| {
            sys_set_20(h, v[0], v[1]);
            0
        }),
        0x0201_2124 => with_rom(c, &[], |h, v| {
            sys_set_30(h, v[0], v[1] as u8);
            0
        }),
        0x0201_213C => with_rom(c, &[], |h, v| {
            sys_set_32(h, v[0], v[1] as u8);
            0
        }),
        0x0201_24E4 => with_rom(c, &[], |h, v| ev_owner(h, v[0]) as u64),
        0x0201_24E8 => with_rom(c, &[], |h, v| ev_data(h, v[0]) as u64),
        0x0201_24F0 => with_rom(c, &[], |h, v| ev_func(h, v[0]) as u64),
        0x0201_24EC => with_rom(c, &[], |_h, v| {
            ev_state_addr(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_283C => with_rom(c, &[], |h, v| gd_get_0201283c(h, v[0]) as u64),
        0x0201_2844 => with_rom(c, &[], |h, v| gd_get_02012844(h, v[0]) as u64),
        0x0201_284C => with_rom(c, &[], |h, v| gd_get_0201284c(h, v[0]) as u64),
        0x0201_2870 => with_rom(c, &[], |h, v| gd_get_02012870(h, v[0]) as u64),
        0x0201_2894 => with_rom(c, &[], |h, v| gd_get_02012894(h, v[0]) as u64),
        0x0201_28B8 => with_rom(c, &[], |h, v| gd_get_020128b8(h, v[0]) as u64),
        0x0201_2924 => with_rom(c, &[], |h, v| gd_get_02012924(h, v[0]) as u64),
        0x0201_292C => with_rom(c, &[], |h, v| gd_get_0201292c(h, v[0]) as u64),
        0x0201_2934 => with_rom(c, &[], |h, v| gd_get_02012934(h, v[0]) as u64),
        0x0201_293C => with_rom(c, &[], |h, v| gd_get_0201293c(h, v[0]) as u64),
        0x0201_295C => with_rom(c, &[], |h, v| gd_get_0201295c(h, v[0]) as u64),
        0x0201_2964 => with_rom(c, &[], |h, v| gd_get_02012964(h, v[0]) as u64),
        0x0201_296C => with_rom(c, &[], |h, v| event_work_ptr(h, v[0]) as u64),
        0x0201_2974 => with_rom(c, &[], |h, v| gd_get_02012974(h, v[0]) as u64),
        0x0201_297C => with_rom(c, &[], |h, v| gd_get_0201297c(h, v[0]) as u64),
        0x0201_2A30 => with_rom(c, &[], |h, v| gd_get_02012a30(h, v[0]) as u64),
        0x0201_2A38 => with_rom(c, &[], |h, v| gd_get_02012a38(h, v[0]) as u64),
        0x0201_2A44 => with_rom(c, &[], |h, v| gd_get_02012a44(h, v[0]) as u64),
        0x0201_2A54 => with_rom(c, &[], |h, v| gd_get_02012a54(h, v[0]) as u64),
        0x0201_2B38 => with_rom(c, &[], |h, v| gd_get_02012b38(h, v[0]) as u64),
        0x0201_2EBC => with_rom(c, &[], |h, v| gd_get_02012ebc(h, v[0]) as u64),
        0x0201_2EC0 => with_rom(c, &[], |h, v| gd_get_02012ec0(h, v[0]) as u64),
        0x0201_2ED8 => with_rom(c, &[], |h, v| gd_get_02012ed8(h, v[0]) as u64),
        0x0201_2F20 => with_rom(c, &[], |h, v| gd_get_02012f20(h, v[0]) as u64),
        0x0201_2F80 => with_rom(c, &[], |h, v| gd_get_02012f80(h, v[0]) as u64),
        0x0201_2FCC => with_rom(c, &[], |h, v| gd_get_02012fcc(h, v[0]) as u64),
        0x0201_2A4C => with_rom(c, &[], |h, v| {
            gd_set_02012a4c(h, v[0], v[1]);
            0
        }),
        0x0201_2EE4 => with_rom(c, &[], |h, v| {
            gd_set_02012ee4(h, v[0], v[1]);
            0
        }),
        0x0201_2F74 => with_rom(c, &[], |h, v| {
            gd_set_02012f74(h, v[0], v[1]);
            0
        }),
        0x0201_2984 => with_rom(c, &[], |h, v| gd_get_02012984(h, v[0]) as u64),
        0x0201_298C => with_rom(c, &[], |h, v| {
            gd_set_0201298c(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2994 => with_rom(c, &[], |h, v| gd_get_02012994(h, v[0]) as u64),
        0x0201_29A0 => with_rom(c, &[], |h, v| {
            gd_set_020129a0(h, v[0], v[1] as u8);
            0
        }),
        0x0201_29AC => with_rom(c, &[], |h, v| gd_get_020129ac(h, v[0]) as u64),
        0x0201_29B8 => with_rom(c, &[], |h, v| {
            gd_set_020129b8(h, v[0], v[1] as u8);
            0
        }),
        0x0201_29C4 => with_rom(c, &[], |h, v| gd_get_020129c4(h, v[0]) as u64),
        0x0201_29D0 => with_rom(c, &[], |h, v| {
            gd_set_020129d0(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2B40 => with_rom(c, &[], |h, v| gd_get_02012b40(h, v[0]) as u64),
        0x0201_2B4C => with_rom(c, &[], |h, v| {
            gd_set_02012b4c(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2B58 => with_rom(c, &[], |h, v| gd_get_02012b58(h, v[0]) as u64),
        0x0201_2B64 => with_rom(c, &[], |h, v| {
            gd_set_02012b64(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2B70 => with_rom(c, &[], |h, v| gd_get_02012b70(h, v[0]) as u64),
        0x0201_2B78 => with_rom(c, &[], |h, v| {
            gd_set_02012b78(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2B80 => with_rom(c, &[], |h, v| gd_get_02012b80(h, v[0]) as u64),
        0x0201_2B8C => with_rom(c, &[], |h, v| {
            gd_set_02012b8c(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2FB0 => with_rom(c, &[], |h, v| gd_get_02012fb0(h, v[0]) as u64),
        0x0201_2FA4 => with_rom(c, &[], |h, v| {
            gd_set_02012fa4(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2A24 => with_rom(c, &[], |h, v| gd_get_02012a24(h, v[0]) as u64),
        0x0201_2E40 => with_rom(c, &[], |h, v| gd_get_02012e40(h, v[0]) as u64),
        0x0201_27A4 => with_rom(c, &[], |_h, v| {
            gd_addr_020127a4(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_28DC => with_rom(c, &[], |_h, v| {
            gd_addr_020128dc(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2C0C => with_rom(c, &[], |_h, v| {
            gd_addr_02012c0c(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2EC4 => with_rom(c, &[], |_h, v| {
            gd_addr_02012ec4(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2ED0 => with_rom(c, &[], |_h, v| {
            gd_addr_02012ed0(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2F38 => with_rom(c, &[], |_h, v| {
            gd_addr_02012f38(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2F50 => with_rom(c, &[], |_h, v| {
            gd_addr_02012f50(v[0]).wrapping_sub(v[0]) as u64
        }),
        0x0201_2AC0 => with_rom(c, &[], |h, v| pl_get_1b(h, v[0]) as u64),
        0x0201_2AC8 => with_rom(c, &[], |h, v| pl_get_00(h, v[0]) as u64),
        0x0201_2AD0 => with_rom(c, &[], |h, v| pl_get_18(h, v[0]) as u64),
        0x0201_2B28 => with_rom(c, &[], |h, v| pl_get_40(h, v[0]) as u64),
        0x0201_2B34 => with_rom(c, &[], |h, v| pl_get_1c(h, v[0]) as u64),
        0x0201_2ABC => with_rom(c, &[], |h, v| {
            pl_set_1b(h, v[0], v[1] as u8);
            0
        }),
        0x0201_2AC4 => with_rom(c, &[], |h, v| {
            pl_set_00(h, v[0], v[1] as u16);
            0
        }),
        0x0201_2ACC => with_rom(c, &[], |h, v| {
            pl_set_18(h, v[0], v[1] as u16);
            0
        }),
        0x0201_2B2C => with_rom(c, &[], |h, v| {
            pl_set_40(h, v[0], v[1]);
            0
        }),
        0x0201_2B30 => with_rom(c, &[], |h, v| {
            pl_set_1c(h, v[0], v[1] as u16);
            0
        }),
        0x0201_2AA0 => with_rom(c, &[], |_h, v| pl_addr_04(v[0]).wrapping_sub(v[0]) as u64),
        0x0201_2AB8 => with_rom(c, &[], |_h, v| pl_addr_10(v[0]).wrapping_sub(v[0]) as u64),
        0x0201_2EF0 => with_rom(c, &[], |h, v| gd_save_block_02012ef0(h, v[0]) as u64),
        0x0201_2EFC => with_rom(c, &[], |h, v| gd_save_block_02012efc(h, v[0]) as u64),
        0x0201_2F2C => with_rom(c, &[], |h, v| gd_save_block_02012f2c(h, v[0]) as u64),
        0x0201_2F44 => with_rom(c, &[], |h, v| gd_save_block_02012f44(h, v[0]) as u64),
        0x0201_2F68 => with_rom(c, &[], |h, v| gd_save_block_02012f68(h, v[0]) as u64),
        0x0201_2FD8 => with_rom(c, &[], |h, v| gd_save_block_02012fd8(h, v[0]) as u64),
        0x0201_2FE4 => with_rom(c, &[], |h, v| save_block_2d(h, v[0]) as u64),
        0x0201_2F08 => with_rom(c, &[], |h, v| gd_save_call_02012f08(h, v[0]) as u64),
        0x0201_2F14 => with_rom(c, &[], |h, v| gd_save_call_02012f14(h, v[0]) as u64),
        0x0201_2F5C => with_rom(c, &[], |h, v| gd_save_call_02012f5c(h, v[0]) as u64),
        0x0201_2F8C => with_rom(c, &[], |h, v| gd_save_call_02012f8c(h, v[0]) as u64),
        0x0201_2F98 => with_rom(c, &[], |h, v| gd_save_call_02012f98(h, v[0]) as u64),
        0x0201_2854 => with_rom(c, &[], |h, v| {
            gd_copy_in_02012854(h, v[0], v[1]);
            0
        }),
        0x0201_2878 => with_rom(c, &[], |h, v| {
            gd_copy_in_02012878(h, v[0], v[1]);
            0
        }),
        0x0201_289C => with_rom(c, &[], |h, v| {
            gd_copy_in_0201289c(h, v[0], v[1]);
            0
        }),
        0x0201_28C0 => with_rom(c, &[], |h, v| {
            gd_copy_in_020128c0(h, v[0], v[1]);
            0
        }),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field game_data`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "game_data",
            include_str!("../../../../tools/rom-diff/specs/black-field/game_data.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

fn mem_with(base: u32, len: usize) -> Mem {
    Mem::new(base, vec![0u8; len])
}

#[test]
fn event_work_pointer_is_the_word_at_0x1ac() {
    let mut m = mem_with(0x0230_0000, GAMEDATA_SIZE);
    m.w32(0x0230_0000 + 0x1AC, 0x0230_1234);
    m.w8(0x0230_0000 + 0x1B0, 0xFF);
    let mut e = Env { mem: m };
    assert_eq!(event_work_ptr(&mut e, 0x0230_0000), 0x0230_1234);
    assert_eq!(EVENT_WORK_PTR_OFS, 0x1AC);
}

/// Memory only, for the accessors that need no callee.
struct Env {
    mem: Mem,
}

impl MemEnv for Env {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

#[test]
fn accessors_use_their_offsets() {
    let base = 0x0230_0000;
    let mut e = Env {
        mem: mem_with(base, GAMEDATA_SIZE),
    };
    gd_set_02012a4c(&mut e, base, 0x1122_3344);
    assert_eq!(e.r32(base + 0x1BC), 0x1122_3344);
    assert_eq!(gd_get_02012a44(&mut e, base), 0x1122_3344);
    gd_set_0201298c(&mut e, base, 0x5A);
    assert_eq!(e.r8(base + 0x1C4), 0x5A);
    assert_eq!(gd_player(base, 2), base + 4 + 2 * 0x44);
    assert_eq!(gd_player_24(base, 2), base + 0x24 + 2 * 0x44);
    assert_eq!(gd_record_10_3(base), base + 0x1D4 + 3 * 0x10);
    assert_eq!(gd_record_1e8_1(base), base + 0x214 + 0x1E8);
    assert_eq!(gd_addr_02012944(base), base + 0x134);
}

#[test]
fn counter_runs_only_while_enabled() {
    let base = 0x0230_0000;
    let mut e = Env {
        mem: mem_with(base, GAMEDATA_SIZE),
    };
    assert_eq!(gd_next_1c8(&mut e, base), 0);
    gd_set_020129dc(&mut e, base, 1);
    assert_eq!(gd_next_1c8(&mut e, base), 0);
    assert_eq!(gd_next_1c8(&mut e, base), 1);
    assert_eq!(e.r8(base + 0x1C8), 2);
    // setting the enable byte to a nonzero value restarts the counter
    gd_set_020129dc(&mut e, base, 5);
    assert_eq!(e.r8(base + 0x1C8), 0);
    // 0 disables it and leaves the counter alone
    e.w8(base + 0x1C8, 9);
    gd_set_020129dc(&mut e, base, 0);
    assert_eq!(e.r8(base + 0x1C8), 9);
    assert_eq!(gd_next_1c8(&mut e, base), 0);
}

#[test]
fn player_direction_classes() {
    let base = 0x0230_0000;
    let mut e = Env {
        mem: mem_with(base, 0x48),
    };
    for (angle, want) in [
        (0u16, 0),
        (0x2000, 0),
        (0x2001, 2),
        (0x5FFF, 2),
        (0x6000, 1),
        (0xA000, 1),
        (0xA001, 3),
        (0xDFFF, 3),
        (0xE000, 0),
        (0xFFFF, 0),
    ] {
        pl_set_18(&mut e, base, angle);
        assert_eq!(pl_direction(&mut e, base), want, "{angle:#x}");
    }
    pl_set_18_from_table(&mut e, base, 1);
    assert_eq!(pl_get_18(&mut e, base), 0x8000);
    pl_init(&mut e, base);
    assert_eq!(pl_get_1c(&mut e, base), 0xFFFF);
    assert_eq!(pl_get_18(&mut e, base), 0);
}

#[test]
fn handle_table_is_signed_and_clamped() {
    assert_eq!(
        [0, 1, 2, 3, 4, 5, 99].map(table_0209dfb0),
        [3, 1, 0xFE, 0xFC, 6, 6, 6]
    );
}
