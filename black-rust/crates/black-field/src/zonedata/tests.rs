use super::*;
use crate::hooks::{HeapHooks, Mem, MemEnv, ZoneHooks};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020139B0, "8ca7aed2becd8cdd"),
    (0x02013A48, "762dd0ed578c1430"),
    (0x02013A80, "97af024d044e9c63"),
    (0x02013A94, "1bef9ff122da5e7d"),
    (0x02013AD0, "98ba7e7fac0cb7ad"),
    (0x02013B10, "fd5c0bc678f7271c"),
    (0x02013B38, "325e7bb6baf02cef"),
    (0x02013B44, "7cba5b6748ab58d9"),
    (0x02013B50, "8923e217058bed8c"),
    (0x02013B5C, "8c015b0b35081d53"),
    (0x02013B68, "0f96f71d78781925"),
    (0x02013B74, "567e03dcc2ae9f83"),
    (0x02013B90, "3e444f76cb650c1a"),
    (0x02013BA8, "c266885b90f78b90"),
    (0x02013BBC, "0acc30074c42608f"),
    (0x02013BD0, "fda5788c30720ca6"),
    (0x02013BDC, "f205729835090015"),
    (0x02013C10, "90a2250f10fc45b7"),
    (0x02013C20, "c5c582b1c018ecce"),
    (0x02013C30, "9b0b4a59f083546e"),
    (0x02013C40, "4dc6f46a08034cf4"),
    (0x02013C50, "c454993fc0f34002"),
    (0x02013C60, "cd5d87041a807027"),
    (0x02013C70, "a063d1303d1e54d6"),
    (0x02013C80, "99994e12e6a13ffb"),
    (0x02013C9C, "35fea04e6dc02cd0"),
    (0x02013CC8, "dfae9d84432488d6"),
    (0x02013CE0, "5d98c139daa743d9"),
    (0x02013D00, "21372c516e3913fb"),
    (0x02013D20, "b9e007647bfb9e04"),
    (0x02013D34, "d18259dd8d2af77d"),
    (0x02013D5C, "206e9db2a624f678"),
    (0x02013D74, "ca7e7cd42351ee06"),
    (0x02013D9C, "763652558b01a3f5"),
    (0x02013DB8, "22411e991983500a"),
    (0x02013DD0, "426cc0f26652f0fb"),
    (0x02013DE0, "e40970f5d01f9403"),
    (0x02013DF0, "e63e074ae9b3f3fc"),
    (0x02013E04, "14e60ebb7181d4c7"),
    (0x02013E40, "b3401754e441f053"),
    (0x02013E4C, "97045a8dfbb9ae29"),
    (0x02013E6C, "cb2f993ba55441b4"),
    (0x02013E7C, "5939bc0859359046"),
    (0x02013E90, "810f2b1fc66ba77a"),
    (0x02013EC0, "51527360e26a312b"),
    (0x02013ECC, "4eaf487a6af3bdd9"),
    (0x02013ED8, "532ee8034fb6b799"),
    (0x02013EEC, "c16d3cd66f4e61f2"),
    (0x02013F2C, "88b16cffdd775e8c"),
    (0x02013F6C, "d5f8df4672d545b2"),
    (0x02013FA8, "a3c4b9f930d9064a"),
    (0x02013FD8, "769b4c374328f0de"),
    (0x02013FEC, "d4fd552256244657"),
    (0x02013FFC, "260c681d7915aa26"),
    (0x0201400C, "b05521a8583e3c8e"),
    (0x02014018, "4ae6a44fd5f1ec9a"),
    (0x02014024, "125e7955013fa389"),
    (0x02014034, "93a067154e5f18ae"),
    (0x02014044, "6e6a747b9d2db586"),
    (0x0201405C, "c88fcd3bfd66d0e2"),
    (0x02014068, "6cd30f8f717f7fe7"),
    (0x0201407C, "fd55f0f5cb5fbffd"),
    (0x020140B4, "1e1df1cef737c6b6"),
    (0x020140BC, "297132bf350f5e58"),
    (0x020140C0, "62db697482c5942d"),
    (0x020140CC, "ab5aac6a5d42d4ef"),
    (0x020140D8, "c32a61f0cde20999"),
    (0x020140DC, "965f101f0117d060"),
    (0x020140E0, "9e4797084fb4ff13"),
    (0x020140E4, "eb582c7bf640b6dd"),
    (0x020140E8, "370e3a26f1c3b785"),
    (0x020140F8, "afb10d8c53271994"),
    (0x02014110, "3630f2deb6cdbbcf"),
    (0x02014134, "d24deec79e517aca"),
    (0x0201416C, "4394fc1c00cbf520"),
    (0x02014184, "3e54d0fda9e33a5d"),
    (0x020141A8, "d95f3f8ea5d90868"),
    (0x020141DC, "40a34b85ada68360"),
    (0x020141E0, "b93a2d5010b986e4"),
    (0x020141F0, "cddf42a1673f35cd"),
    (0x02014204, "79f34d76b6795101"),
    (0x02014264, "b6f7748c7c421cbb"),
    (0x02014274, "fe17892bc9f6ab67"),
    (0x020142A4, "ebc760a2be2676f9"),
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

impl ZoneHooks for Rom<'_> {
    fn fun_020490f4(&mut self, n: u32, heap: u32) -> u32 {
        self.stubs.call(0x0204_90f4, &mut [Arg::V(n), Arg::V(heap)])
    }
    fn fun_02049238(&mut self, handle: u32) {
        self.stubs.call(0x0204_9238, &mut [Arg::V(handle)]);
    }
    fn fun_0201441c(&mut self, v: u32) -> u32 {
        self.stubs.call(0x0201_441c, &mut [Arg::V(v)])
    }
    // the spec records r0-r2 (the size in r3 is the constant 10)
    fn fun_02020e08(&mut self, a: u32, b: u32, c: u32, _size: u32) -> u32 {
        self.stubs
            .call(0x0202_0e08, &mut [Arg::V(a), Arg::V(b), Arg::V(c)])
    }
    fn fun_0204a298(&mut self, a: u32, b: u32, c: u32, heap: u32, size: &mut u32) -> u32 {
        let mut out = [0u8; 4];
        let r = self.stubs.call(
            0x0204_A298,
            &mut [
                Arg::V(a),
                Arg::V(b),
                Arg::V(c),
                Arg::V(heap),
                Arg::Out(&mut out),
            ],
        );
        *size = u32::from_le_bytes(out);
        r
    }
    fn fun_0204944c(&mut self, handle: u32, a: u32, offset: u32, size: u32, dst: u32) {
        let mut out = [0u8; 0x30];
        self.stubs.call(
            0x0204_944C,
            &mut [
                Arg::V(handle),
                Arg::V(a),
                Arg::V(offset),
                Arg::V(size),
                Arg::Out(&mut out),
            ],
        );
        self.mem.write(dst, &out);
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
        0x0201_39B0 => with_rom(c, &[0x02146264], |h, v| {
            cache_new(h, v[0]);
            0
        }),
        0x0201_3A48 => with_rom(c, &[0x02146264], |h, _v| {
            cache_free(h);
            0
        }),
        0x0201_3A80 => with_rom(c, &[0x02146264], |h, v| {
            set_remap_flag(h, v[0], v[1] as u8);
            0
        }),
        0x0201_3A94 => with_rom(c, &[0x02146264], |h, v| remap(h, v[0]) as u64),
        0x0201_3AD0 => with_rom(c, &[0x02146264], |h, v| match zone_record(h, v[0]) {
            0 => 0,
            r => r.wrapping_sub(0x02E2_0000) as u64,
        }),
        0x0201_3B10 => with_rom(c, &[0x02146264], |h, v| {
            zone_in_list(h, v[0], v[1], v[2] as i32) as u64
        }),
        0x0201_3BDC => with_rom(c, &[0x02146264], |h, v| zone_u16_0c_k(h, v[0], v[1]) as u64),
        0x0201_3B74 => with_rom(c, &[0x02146264], |h, v| {
            zone_pos_fixed(h, v[0], v[1]);
            0
        }),
        0x0201_3B90 => with_rom(c, &[0x02146264], |h, v| {
            zone_pos(h, v[0], v[1]);
            0
        }),
        0x0201_3E4C => with_rom(c, &[0x02146264], |h, v| zone_1a_low10(h, v[0]) as u64),
        0x0201_3E7C => with_rom(c, &[0x02146264], |h, v| zone_u16_04_is_zero(h, v[0]) as u64),
        0x0201_3E90 => with_rom(c, &[0x02146264], |h, v| {
            zone_same_area(h, v[0], v[1]) as u64
        }),
        0x0201_4068 => with_rom(c, &[0x02146264], |h, v| zone_u16_18_is_d6(h, v[0]) as u64),
        0x0201_3C70 => with_rom(c, &[0x02146264], |h, v| zone_in_e010(h, v[0]) as u64),
        0x0201_3DD0 => with_rom(c, &[0x02146264], |h, v| zone_in_dfc0(h, v[0]) as u64),
        0x0201_3DE0 => with_rom(c, &[0x02146264], |h, v| zone_in_dfec(h, v[0]) as u64),
        0x0201_3FEC => with_rom(c, &[0x02146264], |h, v| zone_in_dfdc(h, v[0]) as u64),
        0x0201_3FFC => with_rom(c, &[0x02146264], |h, v| zone_in_dfb8(h, v[0]) as u64),
        0x0201_4024 => with_rom(c, &[0x02146264], |h, v| zone_in_dfd2(h, v[0]) as u64),
        0x0201_3C9C => with_rom(c, &[0x02146264], |h, v| zone_table_e026(h, v[0]) as u64),
        0x0201_3C80 => with_rom(c, &[0x02146264], |h, v| zone_has_e026(h, v[0]) as u64),
        0x0201_3FA8 => with_rom(c, &[0x02146264], |h, v| zone_table_dfc8(h, v[0]) as u64),
        0x0201_3CC8 => with_rom(c, &[0x02146264], |h, v| zone_is_422(h, v[0]) as u64),
        0x0201_3CE0 => with_rom(c, &[0x02146264], |h, v| zone_is_150_151(h, v[0]) as u64),
        0x0201_3D00 => with_rom(c, &[0x02146264], |h, v| zone_is_75_76(h, v[0]) as u64),
        0x0201_3D20 => with_rom(c, &[0x02146264], |h, v| zone_is_34(h, v[0]) as u64),
        0x0201_3D34 => with_rom(c, &[0x02146264], |h, v| zone_is_blocked(h, v[0]) as u64),
        0x0201_3D5C => with_rom(c, &[0x02146264], |h, v| zone_is_279(h, v[0]) as u64),
        0x0201_3D9C => with_rom(c, &[0x02146264], |h, v| zone_in_280_288(h, v[0]) as u64),
        0x0201_3D74 => with_rom(c, &[0x02146264], |h, v| zone_in_280_316(h, v[0]) as u64),
        0x0201_3DF0 => with_rom(c, &[0x02146264], |h, v| zone_is_4e(h, v[0]) as u64),
        0x0201_3E04 => with_rom(c, &[0x02146264], |h, v| zone_is_plain(h, v[0]) as u64),
        0x0201_3EEC => with_rom(c, &[0x02146264], |h, v| zone_table_a(h, v[0]) as u64),
        0x0201_3F2C => with_rom(c, &[0x02146264], |h, v| zone_table_b(h, v[0]) as u64),
        0x0201_3F6C => with_rom(c, &[0x02146264], |h, v| zone_table_c(h, v[0]) as u64),
        0x0201_407C => with_rom(c, &[], |h, v| load_area_data(h, v[0], v[1], v[2]) as u64),
        0x0201_40B4 => with_rom(c, &[], |h, v| {
            small_free(h, v[0]);
            0
        }),
        0x0201_40F8 => with_rom(c, &[], |h, v| {
            loc_clear(h, v[0]);
            0
        }),
        0x0201_4110 => with_rom(c, &[], |h, v| {
            loc_set_fixed(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            0
        }),
        0x0201_4134 => with_rom(c, &[], |h, v| {
            loc_set_tiles(h, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            0
        }),
        0x0201_416C => with_rom(c, &[], |h, v| {
            loc_set_ids(h, v[0], v[1], v[2], v[3]);
            0
        }),
        0x0201_4184 => with_rom(c, &[], |h, v| {
            loc_set_kind1(h, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }),
        0x0201_41A8 => with_rom(c, &[], |h, v| {
            loc_set_kind1_tiles(h, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }),
        0x0201_41E0 => with_rom(c, &[], |h, v| {
            loc_get_pos(h, v[0], v[1]);
            0
        }),
        0x0201_41F0 => with_rom(c, &[], |h, v| {
            loc_set_pos(h, v[0], v[1]);
            0
        }),
        0x0201_4204 => with_rom(c, &[0x02146264], |h, v| {
            loc_set_zone(h, v[0], v[1]);
            0
        }),
        0x0201_4264 => with_rom(c, &[0x02146264], |h, v| {
            loc_set_default_zone(h, v[0]);
            0
        }),
        0x0201_4274 => with_rom(c, &[], |_h, v| dir_index(v[0]) as u64),
        0x0201_42A4 => with_rom(c, &[], |_h, v| dir_code(v[0]) as u64),
        0x0201_3B38 => with_rom(c, &[0x02146264], |h, v| zone_u16_02(h, v[0]) as u64),
        0x0201_3B44 => with_rom(c, &[0x02146264], |h, v| zone_u16_04(h, v[0]) as u64),
        0x0201_3B5C => with_rom(c, &[0x02146264], |h, v| zone_u16_06(h, v[0]) as u64),
        0x0201_3B68 => with_rom(c, &[0x02146264], |h, v| zone_u16_0a(h, v[0]) as u64),
        0x0201_3BD0 => with_rom(c, &[0x02146264], |h, v| zone_u16_20(h, v[0]) as u64),
        0x0201_3E40 => with_rom(c, &[0x02146264], |h, v| zone_u16_18(h, v[0]) as u64),
        0x0201_3EC0 => with_rom(c, &[0x02146264], |h, v| zone_u16_14(h, v[0]) as u64),
        0x0201_3ECC => with_rom(c, &[0x02146264], |h, v| zone_u16_16(h, v[0]) as u64),
        0x0201_3B50 => with_rom(c, &[0x02146264], |h, v| zone_u8_00(h, v[0]) as u64),
        0x0201_405C => with_rom(c, &[0x02146264], |h, v| zone_u8_01(h, v[0]) as u64),
        0x0201_3BA8 => with_rom(c, &[0x02146264], |h, v| zone_1c_bits_6_8(h, v[0]) as u64),
        0x0201_3BBC => with_rom(c, &[0x02146264], |h, v| zone_1c_bits_9_15(h, v[0]) as u64),
        0x0201_3C10 => with_rom(c, &[0x02146264], |h, v| zone_1c_bits_0_5(h, v[0]) as u64),
        0x0201_3C20 => with_rom(c, &[0x02146264], |h, v| zone_1e_bit10(h, v[0]) as u64),
        0x0201_3C30 => with_rom(c, &[0x02146264], |h, v| zone_1e_bit14(h, v[0]) as u64),
        0x0201_3C40 => with_rom(c, &[0x02146264], |h, v| zone_1e_bit13(h, v[0]) as u64),
        0x0201_3C50 => with_rom(c, &[0x02146264], |h, v| zone_1e_bit12(h, v[0]) as u64),
        0x0201_3C60 => with_rom(c, &[0x02146264], |h, v| zone_1e_bit15(h, v[0]) as u64),
        0x0201_3ED8 => with_rom(c, &[0x02146264], |h, v| zone_1e_bits_5_9(h, v[0]) as u64),
        0x0201_3FD8 => with_rom(c, &[0x02146264], |h, v| zone_1e_bits_0_4(h, v[0]) as u64),
        0x0201_3E6C => with_rom(c, &[0x02146264], |h, v| zone_1a_bits_10_15(h, v[0]) as u64),
        0x0201_40BC => with_rom(c, &[], |h, v| small_u16_02(h, v[0]) as u64),
        0x0201_40D8 => with_rom(c, &[], |h, v| small_u8_06(h, v[0]) as u64),
        0x0201_40DC => with_rom(c, &[], |h, v| small_u8_07(h, v[0]) as u64),
        0x0201_40E0 => with_rom(c, &[], |h, v| small_u8_08(h, v[0]) as u64),
        0x0201_40E4 => with_rom(c, &[], |h, v| small_u8_09(h, v[0]) as u64),
        0x0201_40C0 => with_rom(c, &[], |h, v| small_u8_04(h, v[0]) as u64),
        0x0201_40CC => with_rom(c, &[], |h, v| small_u8_05(h, v[0]) as u64),
        0x0201_41DC => with_rom(c, &[], |h, v| loc_get_0c(h, v[0]) as u64),
        0x0201_4044 => with_rom(c, &[], |_h, v| is_zone_374_or_378(v[0]) as u64),
        0x0201_400C => with_rom(c, &[], |_h, v| is_zone_f6(v[0]) as u64),
        0x0201_4018 => with_rom(c, &[], |_h, v| is_zone_f7(v[0]) as u64),
        0x0201_4034 => with_rom(c, &[], |_h, v| is_zone_4c_69(v[0]) as u64),
        0x0201_40E8 => with_rom(c, &[], |_h, v| is_zone_2_d1(v[0]) as u64),
        0x0201_3DB8 => with_rom(c, &[], |_h, v| is_zone_289_316(v[0]) as u64),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field zonedata`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "zonedata",
            include_str!("../../../../tools/rom-diff/specs/black-field/zonedata.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Memory only, for the functions that need no callee.
struct Env {
    mem: Mem,
}

impl MemEnv for Env {
    fn mem(&mut self) -> &mut Mem {
        &mut self.mem
    }
}

/// A zone cache at `0x02300000` with the given remap enable bytes.
fn env_with_cache(flags: [u8; 2]) -> Env {
    let mut mem = Mem::new(0x0230_0000, vec![0u8; 0x100]);
    mem.add_window(ZONE_CACHE_PTR, vec![0, 0, 0, 0]);
    mem.w32(ZONE_CACHE_PTR, 0x0230_0000);
    mem.w8(0x0230_0048, flags[0]);
    mem.w8(0x0230_0049, flags[1]);
    Env { mem }
}

#[test]
fn remap_follows_the_enable_bytes() {
    let mut e = env_with_cache([0, 0]);
    assert_eq!([0, 0x126, 5].map(|i| remap(&mut e, i)), [0, 0x126, 5]);
    let mut e = env_with_cache([1, 0]);
    assert_eq!([0, 0x126, 5].map(|i| remap(&mut e, i)), [0x1A8, 0x126, 5]);
    let mut e = env_with_cache([0, 1]);
    assert_eq!([0, 0x126, 5].map(|i| remap(&mut e, i)), [0, 0x127, 5]);
    set_remap_flag(&mut e, 0, 1);
    assert_eq!(remap(&mut e, 0), 0x1A8);
}

#[test]
fn id_predicates_use_the_remapped_id() {
    let mut e = env_with_cache([1, 0]);
    // id 0 becomes 0x1A8, which is in table 0x0209DFC0 but not 0
    assert!(zone_in_dfc0(&mut e, 0));
    assert!(zone_in_dfc0(&mut e, 0x1A8));
    assert!(!zone_in_dfc0(&mut e, 5));
    assert!(zone_is_422(&mut e, 0x1A6));
    assert!(zone_is_150_151(&mut e, 150) && zone_is_150_151(&mut e, 151));
    assert!(!zone_is_150_151(&mut e, 152));
    assert!(zone_is_75_76(&mut e, 76) && !zone_is_75_76(&mut e, 77));
    assert!(zone_in_280_316(&mut e, 280) && zone_in_280_316(&mut e, 316));
    assert!(!zone_in_280_316(&mut e, 317) && !zone_in_280_316(&mut e, 279));
    assert!(zone_is_blocked(&mut e, 0x34));
    assert_eq!(zone_table_e026(&mut e, 0x1C), 0);
    assert_eq!(zone_table_e026(&mut e, 0xF9), 0xB);
    assert_eq!(zone_table_e026(&mut e, 0x2), 0xFFFF_FFFF);
    assert_eq!(zone_table_dfc8(&mut e, 0x3E), 0);
    assert_eq!(zone_table_dfc8(&mut e, 0x146), 3);
    assert_eq!(zone_table_dfc8(&mut e, 0x2), 0xFFFF);
}

#[test]
fn plain_predicates() {
    assert!(is_zone_2_d1(2) && is_zone_2_d1(0xD1));
    assert!(!is_zone_2_d1(1) && !is_zone_2_d1(0xD2));
    assert!(is_zone_289_316(289) && is_zone_289_316(316) && !is_zone_289_316(317));
    assert!(is_zone_4c_69(0x4C) && is_zone_4c_69(0x69) && !is_zone_4c_69(0x6A));
    assert!(is_zone_374_or_378(374) && is_zone_374_or_378(378) && !is_zone_374_or_378(376));
}

#[test]
fn direction_codes_round_trip() {
    assert_eq!([0, 1, 2, 3, 4, 5].map(dir_index), [1, 0, 1, 2, 3, 1]);
    assert_eq!([0, 1, 2, 3, 4].map(dir_code), [1, 2, 3, 4, 2]);
    for code in 1..=4 {
        assert_eq!(dir_code(dir_index(code)), code);
    }
}

#[test]
fn location_records() {
    let mut e = Env {
        mem: Mem::new(0x0230_0000, vec![0xAA; 0x40]),
    };
    loc_clear(&mut e, 0x0230_0000);
    assert!((0..0x1C).all(|i| e.mem.r8(0x0230_0000 + i) == 0));
    loc_set_kind1(&mut e, 0x0230_0000, 7, 2, 0x11, 0x22, 0x33);
    assert_eq!(
        (
            e.mem.r32(0x0230_0000),
            e.mem.r16(0x0230_0004),
            e.mem.r16(0x0230_0006)
        ),
        (1, 7, 0xFFFF)
    );
    assert_eq!(loc_get_0c(&mut e, 0x0230_0000), 0);
    let mut out = [0u32; 3];
    loc_get_pos(&mut e, 0x0230_0000, 0x0230_0000 + 0x1C);
    for (k, o) in out.iter_mut().enumerate() {
        *o = e.mem.r32(0x0230_0000 + 0x1C + 4 * k as u32);
    }
    assert_eq!(out, [0x11, 0x22, 0x33]);
}
