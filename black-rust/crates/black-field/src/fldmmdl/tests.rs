use super::*;
use crate::hooks::Mem;
use crate::testenv::{ofs_arg, ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216CC58, "b6d05a4c5dcc58c1"),
    (0x0216CCB0, "436dd3f6139a7423"),
    (0x0216CCCC, "c1431a06a32e59aa"),
    (0x0216CCDC, "5943626c4f1c6b87"),
    (0x0216CD30, "88b1686089474e39"),
    (0x0216CD50, "a5ab1abc8f1b7421"),
    (0x0216CD64, "24559600fee26015"),
    (0x0216CD78, "1412e6126c84739f"),
    (0x0216CD88, "2309d63d3c4eacc5"),
    (0x0216CDB8, "7f9564bad9cfe117"),
    (0x0216CDC4, "5823d21e2ae162c4"),
    (0x0216CE60, "70801da384aa2c38"),
    (0x0216CE6C, "7ba814927c148b8c"),
    (0x0216CEAC, "5e1cca7aaaeaa6c8"),
    (0x0216CEE4, "99112337f4c86763"),
    (0x0216CF2C, "97ac03726488e359"),
    (0x0216CF6C, "9ad2b0b8e2c06702"),
    (0x0216CFC4, "c3d47f73108857c2"),
    (0x0216D02C, "e9968dc2e6d191c0"),
    (0x0216D040, "d1334645caf8aa0d"),
    (0x0216D0E8, "88885963c0f8ef9a"),
    (0x0216D0F8, "2b8d60e046286ee1"),
    (0x0216D1C8, "994ea46a29dc200e"),
    (0x0216D208, "208005578d9b3c03"),
    (0x0216D230, "44fb7fbe4d3230f3"),
    (0x0216D244, "31df3dfc14546363"),
    (0x0216D268, "96ff43b43b0d94f9"),
    (0x0216D28C, "10ee008fb92fdbf1"),
    (0x0216D2AC, "c7d26f4bb22a38d8"),
    (0x0216D2F8, "a47f7b73330f52a6"),
    (0x0216D38C, "73f4f523d0b2cfda"),
    (0x0216D3A8, "ef6a8fc9345d1d2d"),
    (0x0216D3B4, "40a03ee9a38c4215"),
    (0x0216D3CC, "a7f19af2f1ea856c"),
    (0x0216D410, "6efb31348a4bb64d"),
    (0x0216D438, "4656c3d2d12a2db3"),
    (0x0216D454, "a2dec32ec44133da"),
    (0x0216D480, "ca16c567843a78b1"),
    (0x0216D4B4, "4df59aba015891bd"),
    (0x0216D4D4, "ddd69118750ef198"),
    (0x0216D4DC, "da4d129d02152ee0"),
    (0x0216D4E4, "f7a06e4f7dd2b2f8"),
    (0x0216D4F0, "b52340c0d3950151"),
    (0x0216D4F4, "c2d32fae1dad3357"),
    (0x0216D4F8, "25bf03a787758d7e"),
    (0x0216D4FC, "d83eaac344d276aa"),
    (0x0216D508, "547d375119ab1e6c"),
    (0x0216D514, "68cdb83268e5a776"),
    (0x0216D518, "d80983da2e21c154"),
    (0x0216D51C, "2493044ac4536809"),
    (0x0216D520, "cc53609b4094a8bd"),
    (0x0216D524, "8f0d75ffe9ab0cb1"),
    (0x0216D528, "28ddf9d74d12ae9f"),
    (0x0216D52C, "9a7fe7cee28dd4a8"),
    (0x0216D530, "ba21d10b54ca90cc"),
    (0x0216D534, "c93da13d152a33d0"),
    (0x0216D538, "9b1a803279eac028"),
    (0x0216D548, "c34ab745de44a674"),
    (0x0216D550, "46868cc358464794"),
    (0x0216D55C, "079615e140f95902"),
    (0x0216D564, "c355a8ec3c4dd516"),
    (0x0216D568, "a4023ff72adc1ab1"),
    (0x0216D570, "0bb18532c971c32f"),
    (0x0216D57C, "8d33bdb63e3a7c78"),
    (0x0216D584, "3a51660c3cad6f0b"),
    (0x0216D588, "36cb46cf849c388d"),
    (0x0216D58C, "cb68f06d3d99cc4b"),
    (0x0216D590, "54059f339319aae4"),
    (0x0216D594, "83d23d88d4f14dbe"),
    (0x0216D598, "03fb868e391bdfae"),
    (0x0216D59C, "cbb2f0691c3d47c0"),
    (0x0216D5A0, "3fcf5e39d0b5bbc3"),
    (0x0216D5A4, "e034b5a912d08022"),
    (0x0216D5A8, "b3d9cc994c13a8e0"),
    (0x0216D5AC, "5cac5d7023525a85"),
    (0x0216D5B0, "7f98c12ca9917afc"),
    (0x0216D5B4, "19b000e0a00d9a9c"),
    (0x0216D5B8, "bfbb854ef4e80932"),
    (0x0216D5BC, "757a85744a779431"),
    (0x0216D5D4, "f9059e3c48e3fcd9"),
    (0x0216D5D8, "f064332bc065b6ff"),
    (0x0216D5E0, "e889de3fa00b7626"),
    (0x0216D5F8, "283f3ed95c1be88c"),
    (0x0216D5FC, "8b47a9f02797a7a4"),
    (0x0216D600, "5c0695a42e25b02b"),
    (0x0216D608, "3438c48f86fc6fa9"),
    (0x0216D60C, "b2b654f47d69c759"),
    (0x0216D620, "58920c432da36943"),
    (0x0216D63C, "8f497bcd1e82a247"),
    (0x0216D65C, "d85239fee8445916"),
    (0x0216D660, "01f6810e4e653361"),
    (0x0216D668, "d161fc870c05d382"),
    (0x0216D66C, "a870a90a7eae183e"),
    (0x0216D674, "7ca0343736870e0c"),
    (0x0216D678, "2a74673b425236e0"),
    (0x0216D67C, "4250f23b5d2a364a"),
    (0x0216D684, "4a7d1b320a330525"),
    (0x0216D69C, "2bda2f6400d2c613"),
    (0x0216D6A0, "4e924e48fea89d8a"),
    (0x0216D6B8, "c7e401f812be7716"),
    (0x0216D6BC, "86bcf8f89f74de8b"),
    (0x0216D6D4, "d1e2496221e38e9a"),
    (0x0216D6D8, "dcc7271ae534589a"),
    (0x0216D6F0, "25e1c181333d2e3b"),
    (0x0216D6F4, "cf2f6186e61e7495"),
    (0x0216D708, "d29cef64f64b5590"),
    (0x0216D71C, "ef347ced404ef3da"),
    (0x0216D730, "e948815ca04a693e"),
    (0x0216D744, "d42802836ff78bb9"),
    (0x0216D758, "d10b26ce49d9b8db"),
    (0x0216D76C, "d25d42009a0c2874"),
    (0x0216D780, "162c159bf37df135"),
    (0x0216D794, "24552758fc628a5c"),
    (0x0216D7A8, "c224fb025d806475"),
    (0x0216D7C0, "9cc16a92787eb9ab"),
    (0x0216D7C4, "e81c6ede3d45b07d"),
    (0x0216D7C8, "bd281f2f3548333f"),
    (0x0216D7CC, "367636709cf74d77"),
    (0x0216D7D4, "e1451c5d1a99c00f"),
    (0x0216D7D8, "386d80db8e06713b"),
    (0x0216D7DC, "bbd6a56d9f4c9dc0"),
    (0x0216D7E0, "1ccd29ec6eca8972"),
    (0x0216D7E4, "0d8fb8d48d319f79"),
    (0x0216D7E8, "71f83aa12e5ddf39"),
    (0x0216D7F0, "82aacc45b1f702f6"),
    (0x0216D7F4, "69539fe8025d1ea6"),
    (0x0216D7F8, "c0c3e3f6007fbbe9"),
    (0x0216D800, "c9bff415794b1521"),
    (0x0216D804, "d05435ed4fa9e11f"),
    (0x0216D80C, "bc11bc0285fb183e"),
    (0x0216D810, "ea02d4a3d47772c6"),
    (0x0216D818, "20e7bdfe406b5b22"),
    (0x0216D81C, "ef86174d9c7b18eb"),
    (0x0216D824, "08f67bcd721b9af8"),
    (0x0216D828, "a7602df11b3a69f0"),
    (0x0216D83C, "f2b51f40fb8f886c"),
    (0x0216D844, "f24d0b3c2603c7df"),
    (0x0216D848, "6fea197874a20a78"),
    (0x0216D854, "479ef0d43320adb8"),
    (0x0216D85C, "4c06767fb73b46ae"),
    (0x0216D860, "b611c875cdf49b36"),
    (0x0216D86C, "f70e5ac0ec306b12"),
    (0x0216D874, "7337ad16a39afaa5"),
    (0x0216D87C, "a7a413528ee4e913"),
    (0x0216D888, "c8a1773fbcef9c13"),
    (0x0216D88C, "3b17068e49b2493c"),
    (0x0216D89C, "045b04f024687bd4"),
    (0x0216D8AC, "f28b0b3a1db8af48"),
    (0x0216D8B0, "05b2defa280478a1"),
    (0x0216D8C0, "03686136c76cef26"),
    (0x0216D8D0, "f5723a329985fa01"),
    (0x0216D8E0, "42b56a89442d0f86"),
    (0x0216D8F0, "b49c396017327b29"),
    (0x0216D900, "bdd7a8dc6cf81b7b"),
    (0x0216D910, "f10e210da8e9fc26"),
    (0x0216D91C, "8c3bb7c2c128454a"),
    (0x0216D924, "d6d8a324bdd95bd8"),
    (0x0216D92C, "6f645087bc192a51"),
    (0x0216D948, "356ebdb67da6c786"),
    (0x0216D95C, "a89cd2d1c9eaccb2"),
    (0x0216D974, "bfe6707b19259ba7"),
    (0x0216D98C, "23fbc768b67bdf0a"),
    (0x0216D9A0, "90e88b690af98438"),
    (0x0216D9E4, "bf59244ba33b6759"),
    (0x0216DA20, "58f2d4cc1aac55a3"),
    (0x0216DA30, "43f9a63b38ab53ad"),
    (0x0216DA44, "5feb10eb998646c4"),
    (0x0216DA50, "2689694f620bf34b"),
    (0x0216DA5C, "234029401ad035ee"),
    (0x0216DA70, "8ba1b2a906f56946"),
    (0x0216DA7C, "e2c595930194adfb"),
    (0x0216DA88, "33628c6bb728beaa"),
    (0x0216DA94, "e176004531ebbd2c"),
    (0x0216DAA0, "b7fa9b17ad7445fc"),
    (0x0216DAB4, "4069dde7fd62b072"),
    (0x0216DAC8, "a8d068ce89e99b24"),
    (0x0216DAE0, "9ca5009175477e6d"),
    (0x0216DAF8, "dad1b7f22805a06f"),
    (0x0216DB04, "be7287640adadc0d"),
    (0x0216DB10, "c25534811960352e"),
    (0x0216DB24, "6469f2091119b3fa"),
    (0x0216DB4C, "23a2cd1d4c824031"),
    (0x0216DB68, "aa03818998c2d330"),
    (0x0216DB80, "74bd7e08ed71db79"),
    (0x0216DB98, "962a451acbaeccb3"),
    (0x0216DBB4, "46843c040b9cfb20"),
    (0x0216DBCC, "621535cd6bb8a996"),
    (0x0216DBE8, "0541fc9e45ede953"),
    (0x0216DC00, "904401993608d9b5"),
    (0x0216DC18, "a469dde066c0f972"),
    (0x0216DC34, "4448b493f1c20469"),
    (0x0216DC4C, "0f577895f1d8229a"),
    (0x0216DC64, "b594effc4ef0d349"),
    (0x0216DC80, "e204244ab8fea998"),
    (0x0216DC98, "2abacd0c83b4da3c"),
    (0x0216DCB4, "7060e4f09c1d20cf"),
    (0x0216DCCC, "8598421a96a46d04"),
    (0x0216DCE8, "16aeda983a75be49"),
    (0x0216DD04, "5dcdbd3d387e8109"),
    (0x0216DD1C, "d7636c31b57a843c"),
    (0x0216DD60, "5486c3b3b2d3d057"),
    (0x0216DDC0, "90e27a3907f8a68a"),
    (0x0216DDDC, "c9af0d1427a2e89c"),
    (0x0216DE54, "44fcb252bf407d00"),
    (0x0216DE94, "8a3d27dffcb9b0b0"),
    (0x0216DEE0, "211b812487ab850e"),
    (0x0216DEEC, "f10b0851fbba4f51"),
    (0x0216DF1C, "07206d663598d136"),
    (0x0216DF64, "ecfee2fa41caac66"),
    (0x0216DFAC, "9496d1c865700d02"),
    (0x0216DFC4, "9567c6ebab476bd1"),
    (0x0216E014, "e8ef166d41ca0d94"),
    (0x0216E084, "c62dbd19004a83fa"),
    (0x0216E0AC, "16b2b711828bb667"),
    (0x0216E0E8, "39a0ac82b7b4c05a"),
    (0x0216E114, "3c67f3764dfac331"),
    (0x0216E160, "7bc580712b3f1b16"),
    (0x0216E174, "729b3a07c941ab98"),
    (0x0216E180, "f409dcbe327ef1f7"),
    (0x0216E190, "9a478fa686d65860"),
    (0x0216E1A4, "39e9947ee0279507"),
    (0x0216E1C4, "fff837294633cde7"),
    (0x0216E204, "752aeb1ef7cacb7d"),
    (0x0216E280, "5d449066eeb2bd46"),
    (0x0216E290, "3f1c31a4cf9f7d28"),
    (0x0216E29C, "fda03e3d622ccd23"),
    (0x0216E2AC, "4d00ef25a88f624b"),
    (0x0216E2E0, "ff06441e170dcbe7"),
    (0x0216E2E8, "45168eb424f8ddce"),
    (0x0216E39C, "8ef3d02976951a11"),
    (0x0216E3C4, "6445a896c868ae95"),
    (0x0216E3C8, "a77eec40c31e3164"),
    (0x0216E3CC, "1a7b11e494e20795"),
    (0x0216E3D8, "f80b44aec38046d7"),
    (0x0216E400, "9f91fabe0d1a297b"),
    (0x0216E40C, "f7cf3965fc784c2e"),
    (0x0216E418, "61f2347893853577"),
    (0x0216E434, "e566122aa882e1f8"),
    (0x0216E464, "50305b358173e622"),
    (0x0216E468, "41af703d4d501ec1"),
    (0x0216E46C, "95421a98d30f9a1d"),
    (0x0216E470, "f1595baf530c69d5"),
    (0x0216E474, "2bcc996061eadbc5"),
    (0x0216E47C, "d241dfb877911d58"),
    (0x0216E490, "e2f0ae3d68ae83b6"),
    (0x0216E4B0, "dde00de11a994a2a"),
    (0x0216E4EC, "210290d055448568"),
    (0x0216E51C, "71656a809497f528"),
    (0x0216E544, "006ebffaa5592303"),
    (0x0216E57C, "9f021f0789981fd9"),
    (0x0216E5B0, "6c882fcd63c9ac3f"),
    (0x0216E5FC, "c60480d17f8e2386"),
    (0x0216E618, "a42b816cd96894a2"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/fldmmdl.spec");

/// A `u32` result.
fn u(v: u32) -> u64 {
    v as u64
}

/// A signed (`ldrsh`) result: the register holds the sign-extended value.
fn s(v: i32) -> u64 {
    v as u32 as u64
}

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = &mut e.mem;
        match addr {
            0x0216_CC58 => ofs_arg(sys_create(e, v[0], v[1], v[2]), 3),
            0x0216_CCB0 => {
                sys_delete(e, v[0]);
                0
            }
            0x0216_CCCC => {
                clear_obj_table(m, v[0]);
                0
            }
            0x0216_CCDC => {
                tasks_start(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x0216_CD30 => {
                tasks_stop(e, v[0]);
                0
            }
            0x0216_CD50 => {
                sys_stop(e, v[0]);
                0
            }
            0x0216_CD64 => {
                sys_update(e, v[0]);
                0
            }
            0x0216_CD78 => {
                sys_draw(e, v[0]);
                0
            }
            0x0216_CD88 => {
                sync_draw(e, v[0]);
                0
            }
            0x0216_CDB8 => {
                sys_set_40(m, v[0], v[1]);
                0
            }
            0x0216_CDC4 => ofs_work(create(e, v[0], v[1], v[2], v[3])),
            0x0216_CE60 => ofs_work(create_simple(e, v[0], v[1], v[2])),
            0x0216_CE6C => ofs_work(create_at(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])),
            0x0216_CEAC => {
                create_all(e, v[0], v[1], v[2], v[3], v[4]);
                0
            }
            0x0216_CEE4 => ofs_work(create_by_id(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x0216_CF2C => {
                delete(e, v[0]);
                0
            }
            0x0216_CF6C => {
                delete_all(e, v[0]);
                0
            }
            0x0216_CFC4 => {
                init_header_fields(m, v[0], v[1]);
                0
            }
            0x0216_D02C => {
                init_pos(e, v[0], v[1]);
                0
            }
            0x0216_D040 => {
                init_from_header(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x0216_D0E8 => {
                header_post(e, v[0], v[1], v[2]);
                0
            }
            0x0216_D0F8 => {
                init_grid_pos(e, v[0], v[1]);
                0
            }
            0x0216_D1C8 => {
                init_base(m, v[0], v[1], v[2]);
                0
            }
            0x0216_D208 => {
                init_dirs(m, v[0]);
                0
            }
            0x0216_D230 => {
                set_move_procs(e, v[0]);
                0
            }
            0x0216_D244 => {
                move_setup(e, v[0], v[1]);
                0
            }
            0x0216_D268 => {
                move_init_kind(e, v[0], v[1]);
                0
            }
            0x0216_D28C => {
                move_update_kind(e, v[0]);
                0
            }
            0x0216_D2AC => {
                suspend_all(e, v[0]);
                0
            }
            0x0216_D2F8 => {
                resume_all(e, v[0]);
                0
            }
            0x0216_D38C => {
                task_body(e, v[0]);
                0
            }
            0x0216_D3A8 => {
                task_callback(e, v[0], v[1]);
                0
            }
            0x0216_D3B4 => {
                draw_update(e, v[0]);
                0
            }
            0x0216_D3CC => u(acmd_can_set(m, v[0])),
            0x0216_D410 => {
                acmd_set(m, v[0], v[1]);
                0
            }
            0x0216_D438 => {
                acmd_change(m, v[0], v[1]);
                0
            }
            0x0216_D454 => u(acmd_done(m, v[0])),
            0x0216_D480 => u(acmd_end(m, v[0])),
            0x0216_D4B4 => {
                acmd_reset(m, v[0]);
                0
            }
            0x0216_D4D4 => u(sys_flags(m, v[0], v[1])),
            0x0216_D4DC => {
                sys_set_flags(m, v[0], v[1]);
                0
            }
            0x0216_D4E4 => {
                sys_clear_flags(m, v[0], v[1]);
                0
            }
            0x0216_D4F0 => count(m, v[0]) as u64,
            0x0216_D4F4 => task_priority(m, v[0]) as u64,
            0x0216_D4F8 => sys_heap(m, v[0]) as u64,
            0x0216_D4FC => {
                inc_in_use(m, v[0]);
                0
            }
            0x0216_D508 => {
                dec_in_use(m, v[0]);
                0
            }
            0x0216_D514 => u(task_sys(m, v[0])),
            0x0216_D518 => u(arc49(m, v[0])),
            0x0216_D51C => {
                set_draw_sys(m, v[0], v[1]);
                0
            }
            0x0216_D520 => u(draw_sys(m, v[0])),
            0x0216_D524 => {
                set_sys_28(m, v[0], v[1]);
                0
            }
            0x0216_D528 => u(sys_28(m, v[0])),
            0x0216_D52C => u(sys_34(m, v[0])),
            0x0216_D530 => u(sys_38(m, v[0])),
            0x0216_D534 => u(sys_3c(m, v[0])),
            0x0216_D538 => u(sys_40_value(m, v[0])),
            0x0216_D548 => {
                set_status(m, v[0], v[1]);
                0
            }
            0x0216_D550 => {
                clear_status(m, v[0], v[1]);
                0
            }
            0x0216_D55C => u(status(m, v[0], v[1])),
            0x0216_D564 => u(move_bits(m, v[0])),
            0x0216_D568 => {
                set_bits(m, v[0], v[1]);
                0
            }
            0x0216_D570 => {
                clear_bits(m, v[0], v[1]);
                0
            }
            0x0216_D57C => u(bits(m, v[0], v[1])),
            0x0216_D584 => {
                set_obj_id(m, v[0], v[1]);
                0
            }
            0x0216_D588 => obj_id(m, v[0]) as u64,
            0x0216_D58C => {
                set_zone(m, v[0], v[1]);
                0
            }
            0x0216_D590 => zone(m, v[0]) as u64,
            0x0216_D594 => {
                set_obj_code(m, v[0], v[1]);
                0
            }
            0x0216_D598 => obj_code(m, v[0]) as u64,
            0x0216_D59C => {
                set_move_code(m, v[0], v[1]);
                0
            }
            0x0216_D5A0 => move_code(m, v[0]) as u64,
            0x0216_D5A4 => {
                set_event_type(m, v[0], v[1]);
                0
            }
            0x0216_D5A8 => event_type(m, v[0]) as u64,
            0x0216_D5AC => {
                set_event_flag(m, v[0], v[1]);
                0
            }
            0x0216_D5B0 => event_flag(m, v[0]) as u64,
            0x0216_D5B4 => {
                set_event_id(m, v[0], v[1]);
                0
            }
            0x0216_D5B8 => event_id(m, v[0]) as u64,
            0x0216_D5BC => u(is_alias_id(m, v[0])),
            0x0216_D5D4 => header_dir(m, v[0]) as u64,
            0x0216_D5D8 => {
                set_dir_disp_raw(m, v[0], v[1]);
                0
            }
            0x0216_D5E0 => {
                set_dir_disp(m, v[0], v[1]);
                0
            }
            0x0216_D5F8 => dir_disp(m, v[0]) as u64,
            0x0216_D5FC => dir_disp_old(m, v[0]) as u64,
            0x0216_D600 => {
                set_dir_move(m, v[0], v[1]);
                0
            }
            0x0216_D608 => dir_move(m, v[0]) as u64,
            0x0216_D60C => {
                set_dir(m, v[0], v[1]);
                0
            }
            0x0216_D620 => {
                set_param(m, v[0], v[1], v[2]);
                0
            }
            0x0216_D63C => param(m, v[0], v[1]) as u64,
            0x0216_D65C => {
                set_limit_x(m, v[0], v[1]);
                0
            }
            0x0216_D660 => s(limit_x(m, v[0])),
            0x0216_D668 => {
                set_limit_z(m, v[0], v[1]);
                0
            }
            0x0216_D66C => s(limit_z(m, v[0])),
            0x0216_D674 => {
                set_field_2a(m, v[0], v[1]);
                0
            }
            0x0216_D678 => field_2a(m, v[0]) as u64,
            0x0216_D67C => u(sys_of(m, v[0])),
            0x0216_D684 => ofs_work(move_work_init(m, v[0], v[1])),
            0x0216_D69C => ofs_work(move_work(v[0])),
            0x0216_D6A0 => ofs_work(sub_work_init(m, v[0], v[1])),
            0x0216_D6B8 => ofs_work(sub_work(v[0])),
            0x0216_D6BC => ofs_work(acmd_work_init(m, v[0], v[1])),
            0x0216_D6D4 => ofs_work(acmd_work(v[0])),
            0x0216_D6D8 => ofs_work(draw_work_init(m, v[0], v[1])),
            0x0216_D6F0 => ofs_work(draw_work(v[0])),
            0x0216_D6F4 => {
                move_init(e, v[0]);
                0
            }
            0x0216_D708 => {
                move_run(e, v[0]);
                0
            }
            0x0216_D71C => {
                move_delete(e, v[0]);
                0
            }
            0x0216_D730 => {
                move_return(e, v[0]);
                0
            }
            0x0216_D744 => {
                draw_init(e, v[0]);
                0
            }
            0x0216_D758 => {
                draw(e, v[0]);
                0
            }
            0x0216_D76C => {
                draw_delete(e, v[0]);
                0
            }
            0x0216_D780 => {
                draw_push(e, v[0]);
                0
            }
            0x0216_D794 => {
                draw_pop(e, v[0]);
                0
            }
            0x0216_D7A8 => u(draw_get(e, v[0], v[1])),
            0x0216_D7C0 => {
                set_acmd_code(m, v[0], v[1]);
                0
            }
            0x0216_D7C4 => acmd_code(m, v[0]) as u64,
            0x0216_D7C8 => {
                set_acmd_seq(m, v[0], v[1]);
                0
            }
            0x0216_D7CC => {
                inc_acmd_seq(m, v[0]);
                0
            }
            0x0216_D7D4 => acmd_seq(m, v[0]) as u64,
            0x0216_D7D8 => {
                set_field_74(m, v[0], v[1]);
                0
            }
            0x0216_D7DC => u(field_74(m, v[0])),
            0x0216_D7E0 => {
                set_field_78(m, v[0], v[1]);
                0
            }
            0x0216_D7E4 => u(field_78(m, v[0])),
            0x0216_D7E8 => s(init_gx(m, v[0])),
            0x0216_D7F0 => {
                set_init_gx(m, v[0], v[1]);
                0
            }
            0x0216_D7F4 => {
                set_init_gy(m, v[0], v[1]);
                0
            }
            0x0216_D7F8 => s(init_gz(m, v[0])),
            0x0216_D800 => {
                set_init_gz(m, v[0], v[1]);
                0
            }
            0x0216_D804 => s(old_gx(m, v[0])),
            0x0216_D80C => {
                set_old_gx(m, v[0], v[1]);
                0
            }
            0x0216_D810 => s(old_gy(m, v[0])),
            0x0216_D818 => {
                set_old_gy(m, v[0], v[1]);
                0
            }
            0x0216_D81C => s(old_gz(m, v[0])),
            0x0216_D824 => {
                set_old_gz(m, v[0], v[1]);
                0
            }
            0x0216_D828 => {
                grid(m, v[0], v[1]);
                0
            }
            0x0216_D83C => s(gx(m, v[0])),
            0x0216_D844 => {
                set_gx(m, v[0], v[1]);
                0
            }
            0x0216_D848 => {
                add_gx(m, v[0], v[1]);
                0
            }
            0x0216_D854 => s(gy(m, v[0])),
            0x0216_D85C => {
                set_gy(m, v[0], v[1]);
                0
            }
            0x0216_D860 => {
                add_gy(m, v[0], v[1]);
                0
            }
            0x0216_D86C => s(gz(m, v[0])),
            0x0216_D874 => {
                set_gz(m, v[0], v[1]);
                0
            }
            0x0216_D87C => {
                add_gz(m, v[0], v[1]);
                0
            }
            0x0216_D888 => ofs_work(pos_ptr(v[0])),
            0x0216_D88C => {
                get_pos(m, v[0], v[1]);
                0
            }
            0x0216_D89C => {
                set_pos(m, v[0], v[1]);
                0
            }
            0x0216_D8AC => u(pos_y(m, v[0])),
            0x0216_D8B0 => {
                get_vec50(m, v[0], v[1]);
                0
            }
            0x0216_D8C0 => {
                set_vec50(m, v[0], v[1]);
                0
            }
            0x0216_D8D0 => {
                get_vec5c(m, v[0], v[1]);
                0
            }
            0x0216_D8E0 => {
                set_vec5c(m, v[0], v[1]);
                0
            }
            0x0216_D8F0 => {
                get_vec68(m, v[0], v[1]);
                0
            }
            0x0216_D900 => {
                set_vec68(m, v[0], v[1]);
                0
            }
            0x0216_D910 => u(draw_sys_of(m, v[0])),
            0x0216_D91C => size_x(m, v[0]) as u64,
            0x0216_D924 => size_z(m, v[0]) as u64,
            0x0216_D92C => {
                offsets(m, v[0], v[1]);
                0
            }
            0x0216_D948 => u(draw_on(m, v[0])),
            0x0216_D95C => {
                set_draw_on(m, v[0], v[1]);
                0
            }
            0x0216_D974 => {
                set_flag8(m, v[0], v[1]);
                0
            }
            0x0216_D98C => u(flag8_clear(m, v[0])),
            0x0216_D9A0 => {
                pause_all(m, v[0]);
                0
            }
            0x0216_D9E4 => {
                unpause_all(m, v[0]);
                0
            }
            0x0216_DA20 => u(sys_flags_of(m, v[0], v[1])),
            0x0216_DA30 => u(in_use(m, v[0])),
            0x0216_DA44 => {
                set_bit10(m, v[0]);
                0
            }
            0x0216_DA50 => {
                clear_bit10(m, v[0]);
                0
            }
            0x0216_DA5C => u(bit10(m, v[0])),
            0x0216_DA70 => {
                set_bit20(m, v[0]);
                0
            }
            0x0216_DA7C => {
                clear_bit40(m, v[0]);
                0
            }
            0x0216_DA88 => {
                set_draw_ready(m, v[0]);
                0
            }
            0x0216_DA94 => {
                clear_draw_ready(m, v[0]);
                0
            }
            0x0216_DAA0 => u(draw_ready(m, v[0])),
            0x0216_DAB4 => u(status4(m, v[0])),
            0x0216_DAC8 => {
                set_status4(m, v[0], v[1]);
                0
            }
            0x0216_DAE0 => {
                set_status80_inv(m, v[0], v[1]);
                0
            }
            0x0216_DAF8 => {
                pause(m, v[0]);
                0
            }
            0x0216_DB04 => {
                unpause(m, v[0]);
                0
            }
            0x0216_DB10 => u(paused(m, v[0])),
            0x0216_DB24 => u(drawable(m, v[0])),
            0x0216_DB4C => {
                set_status200(m, v[0], v[1]);
                0
            }
            0x0216_DB68 => u(status200(m, v[0])),
            0x0216_DB80 => {
                set_status20(m, v[0], v[1]);
                0
            }
            0x0216_DB98 => {
                set_alias(m, v[0], v[1]);
                0
            }
            0x0216_DBB4 => u(is_alias(m, v[0])),
            0x0216_DBCC => {
                set_bit40000(m, v[0], v[1]);
                0
            }
            0x0216_DBE8 => u(bit_40000(m, v[0])),
            0x0216_DC00 => u(status1000(m, v[0])),
            0x0216_DC18 => {
                set_bit20000(m, v[0], v[1]);
                0
            }
            0x0216_DC34 => u(bit_20000(m, v[0])),
            0x0216_DC4C => u(acmd_active(m, v[0])),
            0x0216_DC64 => {
                set_bit80000(m, v[0], v[1]);
                0
            }
            0x0216_DC80 => u(bit_80000(m, v[0])),
            0x0216_DC98 => {
                set_status400(m, v[0], v[1]);
                0
            }
            0x0216_DCB4 => u(status400(m, v[0])),
            0x0216_DCCC => {
                set_bit100000(m, v[0], v[1]);
                0
            }
            0x0216_DCE8 => {
                set_bit200000(m, v[0], v[1]);
                0
            }
            0x0216_DD04 => u(bit_200000(m, v[0])),
            0x0216_DD1C => next_in_use(m, v[0], v[1], v[2]) as u64,
            0x0216_DD60 => u(list_at(e, v[0], v[1], v[2], v[3], v[4])),
            0x0216_DDC0 => ofs_work(find_at(e, v[0], v[1], v[2], v[3])),
            0x0216_DDDC => ofs_work(find_at_height(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x0216_DE54 => ofs_work(find_by_move_code(m, v[0], v[1])),
            0x0216_DE94 => ofs_work(find_by_id(m, v[0], v[1])),
            0x0216_DEE0 => ofs_work(find_player(m, v[0])),
            0x0216_DEEC => ofs_work(first_free(m, v[0])),
            0x0216_DF1C => {
                delete_unkept(e, v[0]);
                0
            }
            0x0216_DF64 => {
                task_add(e, v[0], v[1]);
                0
            }
            0x0216_DFAC => {
                task_del(e, v[0]);
                0
            }
            0x0216_DFC4 => u(obj_code_in_use(m, v[0], v[1])),
            0x0216_E014 => {
                set_position(e, v[0], v[1], v[2]);
                0
            }
            0x0216_E084 => {
                set_grid_position(e, v[0], v[1], v[2], v[3], v[4]);
                0
            }
            0x0216_E0AC => {
                reset_from_header(e, v[0], v[1]);
                0
            }
            0x0216_E0E8 => {
                change_move_code(e, v[0], v[1]);
                0
            }
            0x0216_E114 => {
                draw_setup(e, v[0]);
                0
            }
            0x0216_E160 => {
                set_draw_procs(e, v[0]);
                0
            }
            0x0216_E174 => {
                clear_bits_6c000(m, v[0]);
                0
            }
            0x0216_E180 => {
                clear(m, v[0]);
                0
            }
            0x0216_E190 => {
                mark_moved(m, v[0]);
                0
            }
            0x0216_E1A4 => {
                key(m, v[0], v[1]);
                0
            }
            0x0216_E1C4 => u(matches_key(m, v[0], v[1])),
            0x0216_E204 => {
                change_obj_code(e, v[0], v[1]);
                0
            }
            0x0216_E280 => {
                open_obj_arc(e, v[0], v[1]);
                0
            }
            0x0216_E290 => {
                close_obj_arc(e, v[0]);
                0
            }
            0x0216_E29C => {
                load_obj_table(e, v[0], v[1]);
                0
            }
            0x0216_E2AC => {
                table_record(m, v[0], v[1], v[2]);
                0
            }
            0x0216_E2E0 => table_count(m, v[0]) as u64,
            0x0216_E2E8 => {
                get_obj_data(e, v[0], v[1], v[2]);
                0
            }
            0x0216_E39C => {
                load_obj_data(e, v[0], v[1], v[2]);
                0
            }
            0x0216_E3C4 => ofs_work(obj_data(v[0])),
            0x0216_E3C8 => u(plus_10a(v[0])),
            0x0216_E3CC => u(plus_10b(v[0])),
            0x0216_E3D8 => u(resolve_obj_code(e, v[0], v[1])),
            0x0216_E400 => u(move_proc_table(v[0])),
            0x0216_E40C => u(draw_proc_table(v[0])),
            0x0216_E418 => u(header_is_alias(m, v[0])),
            0x0216_E434 => u(kind_434(v[0])),
            0x0216_E464 => {
                nop_464(v[0]);
                0
            }
            0x0216_E468 => {
                nop_468(v[0]);
                0
            }
            0x0216_E46C => {
                nop_46c(v[0]);
                0
            }
            0x0216_E470 => {
                nop_470(v[0]);
                0
            }
            0x0216_E474 => u(save_work_size()),
            0x0216_E47C => {
                save_work_init(m, v[0]);
                0
            }
            0x0216_E490 => u(is_stored(m, v[0])),
            0x0216_E4B0 => save_slot(v[0], v[1]) as u64,
            0x0216_E4EC => save_slot_of(m, v[0]) as u64,
            0x0216_E51C => u(has_saved_pos(m, v[0])),
            0x0216_E544 => u(saved_pos(m, v[0], v[1])),
            0x0216_E57C => {
                store_pos(m, v[0]);
                0
            }
            0x0216_E5B0 => u(stored_object_at(e, v[0], v[1])),
            0x0216_E5FC => u(keeps_position(v[0])),
            0x0216_E618 => {
                header_set_grid(m, v[0], v[1], v[2], v[3]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field fldmmdl`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("fldmmdl", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn iterator_skips_free_objects() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x1000]);
    mem.w16(WORK + 4, 3); // three MMDLs
    mem.w32(WORK + 0x18, WORK + 0x100);
    mem.w32(WORK + 0x200, ST_IN_USE); // MMDL 1 in use
    let (out, idx) = (WORK + 0x800, WORK + 0x804);
    assert!(next_in_use(&mut mem, WORK, out, idx));
    assert_eq!((mem.r32(out), mem.r32(idx)), (WORK + 0x200, 2));
    assert!(!next_in_use(&mut mem, WORK, out, idx));
    assert_eq!(mem.r32(idx), 3);
}

#[test]
fn save_slots_by_zone_and_id() {
    assert_eq!(save_slot(0x14D, 1), 2);
    assert_eq!(save_slot(0x15A, 5), 25);
    assert_eq!(save_slot(0x14D, 3), NO_SLOT);
    assert_eq!(save_slot(0x1_014D, 1), NO_SLOT);
}

#[test]
fn obj_table_records_follow_the_code_list() {
    // (((count + 1) / 2 * 2) + 2) halfwords before the records
    let t = WORK + 0x1100;
    assert_eq!(table_records(t, 0), t + 4);
    assert_eq!(table_records(t, 1), t + 8);
    assert_eq!(table_records(t, 2), t + 8);
    assert_eq!(table_records(t, 3), t + 12);
}

#[test]
fn grid_rounds_toward_zero() {
    assert_eq!(grid_of(0x8_0000), 8);
    assert_eq!(grid_of(0x8000), 0);
    assert_eq!(grid_of((-0x1_0000i32) as u32), -1);
    assert_eq!(grid_of((-0x8000i32) as u32), 0);
}
