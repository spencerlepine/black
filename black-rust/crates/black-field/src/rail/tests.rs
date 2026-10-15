use super::*;
use crate::testenv::{overlay21, run};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B7884, "b2d334f266b76d5c"),
    (0x021B791C, "c7e4fa8d71438372"),
    (0x021B7934, "f5b249436406570e"),
    (0x021B7954, "b5f5df65f333968d"),
    (0x021B7970, "c0f20cfc6022a239"),
    (0x021B79AC, "2489fb505c198a9d"),
    (0x021B79C4, "990ad2e85bafb0c0"),
    (0x021B79CC, "482240d5fa638b3b"),
    (0x021B79D4, "cffdd5dc24f24af0"),
    (0x021B79E4, "d61fd5da7635d3c3"),
    (0x021B79F0, "c100ef76c4ec8d03"),
    (0x021B7A84, "b98163fe163b41a3"),
    (0x021B7C04, "74d5fdcd2e251ef4"),
    (0x021B7C38, "50de9e4cd71b9502"),
    (0x021B7C58, "edc79c2e997a456d"),
    (0x021B7C90, "b3ec64f1e8514610"),
    (0x021B7CD8, "962d31a73215a011"),
    (0x021B7D38, "5fd0bc792ac4cfdf"),
    (0x021B7D50, "65fb38068c4e0754"),
    (0x021B7D68, "68338992af6cae1d"),
    (0x021B7DC0, "108790691c1d657b"),
    (0x021B7DE0, "052bac57e09c677b"),
    (0x021B7E60, "fa725ebf59715695"),
    (0x021B7ED4, "b311592e56dd4653"),
    (0x021B7EE4, "78765425e3b1ebff"),
    (0x021B8028, "1d95bc388bb36cd1"),
    (0x021B8038, "c55766859232c653"),
    (0x021B8040, "25e9dc0c4786bcc7"),
    (0x021B8048, "f68738cd94b4067e"),
    (0x021B8050, "5526ed4e936f6817"),
    (0x021B80D8, "edd85848d689fd2b"),
    (0x021B80E0, "3fe18ecb6b932762"),
    (0x021B810C, "d11d738dfcbc3011"),
    (0x021B821C, "c989657acfc4c74e"),
    (0x021B8220, "507c887aa52e7772"),
    (0x021B8224, "5c73488f0d475019"),
    (0x021B8230, "f9d956873b076119"),
    (0x021B823C, "3f9beb3b910a2064"),
    (0x021B8248, "2ac7ab9755bfc32a"),
    (0x021B824C, "355f0944b695afcf"),
    (0x021B8250, "2990a13709004c74"),
    (0x021B82F0, "8709e9021eb93a35"),
    (0x021B82F4, "d05324c443cb59b0"),
    (0x021B82F8, "ba83644a650b9cd8"),
    (0x021B82FC, "cffc3067c8f6b9af"),
    (0x021B8300, "57704d6b4b5dfc48"),
    (0x021B8304, "5691ab6a9755b1bb"),
    (0x021B8308, "67661b64cbc2229a"),
    (0x021B830C, "7d55e00b14f7adaf"),
    (0x021B8314, "e769ea68bb99c560"),
    (0x021B8328, "115129b815079a62"),
    (0x021B833C, "803fcce1cca07603"),
    (0x021B834C, "7de4a59c5fd97e13"),
    (0x021B835C, "48acd61b1aff446c"),
    (0x021B836C, "08085c654af5969d"),
    (0x021B8378, "cdf531464e40ab6b"),
    (0x021B83B0, "e63272265c3035a6"),
    (0x021B83FC, "ade257f8df1d4b96"),
    (0x021B8484, "54a6091c832c5f0b"),
    (0x021B8514, "43621238f6aba0f4"),
    (0x021B854C, "bafdd0d336a32b59"),
    (0x021B8658, "e4a99011fd837e2d"),
    (0x021B8674, "4b2d8bb3c3da457e"),
    (0x021B868C, "27a11082b44d42ba"),
    (0x021B86A4, "7de0c30f24c8832c"),
    (0x021B87BC, "350c6410c9c4c26c"),
    (0x021B87C8, "4ba06bb86863c933"),
    (0x021B87D8, "3ce63aa5bf3eebdd"),
    (0x021B87FC, "e20848cee991fbe1"),
    (0x021B8820, "765d7e715214cb42"),
    (0x021B8848, "81edff04a5a6da9d"),
    (0x021B8870, "b63ed80e5c9254b1"),
    (0x021B8894, "627198f6c30497ea"),
    (0x021B88B8, "8c7832004b350272"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/rail.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_7884 => u64::from(create(e, v[0], v[1], v[2])),
        0x021B_791C => {
            delete(e, v[0]);
            0
        }
        0x021B_7934 => {
            set_data(e, v[0], v[1]);
            0
        }
        0x021B_7954 => {
            clear_data(e, v[0]);
            0
        }
        0x021B_7970 => u64::from(obj_alloc(e, v[0])),
        0x021B_79AC => {
            obj_free(e, v[0], v[1]);
            0
        }
        0x021B_79C4 => {
            set_current(e, v[0], v[1]);
            0
        }
        0x021B_79CC => {
            clear_current(e, v[0]);
            0
        }
        0x021B_79D4 => {
            current_pos(e, v[0], v[1]);
            0
        }
        0x021B_79E4 => u64::from(frame_state(e, v[0])),
        0x021B_79F0 => u64::from(frame(e, v[0], v[1])),
        0x021B_7A84 => u64::from(line_hit(e, v[0], v[1], v[2], v[3], v[4])),
        0x021B_7C04 => u64::from(block_line(e, v[0], v[1], v[2])),
        0x021B_7C38 => u64::from(unit_fx(e, v[0])),
        0x021B_7C58 => {
            loc_from_line(e, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        0x021B_7C90 => u64::from(move_loc(e, v[0], v[1], v[2], v[3])),
        0x021B_7CD8 => u64::from(step_loc(e, v[0], v[1], v[2], v[3])),
        0x021B_7D38 => {
            loc_pos(e, v[0], v[1], v[2]);
            0
        }
        0x021B_7D50 => u64::from(loc_width(e, v[0], v[1])),
        0x021B_7D68 => {
            run_loc_proc(e, v[0], v[1]);
            0
        }
        0x021B_7DC0 => {
            get_loc(e, v[0], v[1]);
            0
        }
        0x021B_7DE0 => u64::from(loc_valid(e, v[0], v[1])),
        0x021B_7E60 => u64::from(cached_step(e, v[0], v[1], v[2], v[3])),
        0x021B_7ED4 => {
            old_pos(e, v[0], v[1]);
            0
        }
        0x021B_7EE4 => {
            set_loc(e, v[0], v[1]);
            0
        }
        0x021B_8028 => {
            get_pos(e, v[0], v[1]);
            0
        }
        0x021B_8038 => u64::from(dir_byte(e, v[0])),
        0x021B_8040 => u64::from(moving(e, v[0])),
        0x021B_8048 => u64::from(moved(e, v[0])),
        0x021B_8050 => {
            dir_vec(e, v[0], v[1]);
            0
        }
        0x021B_80D8 => u64::from(rec_word8(e, v[0])),
        0x021B_80E0 => u64::from(start_move(e, v[0], v[1], v[2])),
        0x021B_810C => {
            move_frame(e, v[0]);
            0
        }
        0x021B_821C => u64::from(word10(e, v[0])),
        0x021B_8220 => u64::from(word14(e, v[0])),
        0x021B_8224 => u64::from(sys_point(e, v[0], v[1])),
        0x021B_8230 => u64::from(sys_line(e, v[0], v[1])),
        0x021B_823C => u64::from(sys_line_rec(e, v[0], v[1])),
        0x021B_8248 => u64::from(sys_unit(e, v[0])),
        0x021B_824C => u64::from(main_work(e, v[0])),
        0x021B_8250 => {
            clamp_loc(e, v[0], v[1], v[2]);
            0
        }
        0x021B_82F0 => u64::from(kind(e, v[0])),
        0x021B_82F4 => u64::from(rec(e, v[0])),
        0x021B_82F8 => u64::from(along(e, v[0])),
        0x021B_82FC => u64::from(length(e, v[0])),
        0x021B_8300 => u64::from(lateral(e, v[0])),
        0x021B_8304 => u64::from(width(e, v[0])),
        0x021B_8308 => u64::from(unit(e, v[0])),
        0x021B_830C => u64::from(cam_of(e, v[0])),
        0x021B_8314 => {
            pos_no_lat(e, v[0], v[1]);
            0
        }
        0x021B_8328 => {
            pos_at(e, v[0], v[1], v[2]);
            0
        }
        0x021B_833C => u64::from(line_start(e, v[0])),
        0x021B_834C => u64::from(line_end(e, v[0])),
        0x021B_835C => u64::from(line_record(e, v[0])),
        0x021B_836C => u64::from(point_cam(e, v[0], v[1])),
        0x021B_8378 => u64::from(round16(v[0])),
        0x021B_83B0 => u64::from(near(e, v[0], v[1], v[2])),
        0x021B_83FC => {
            loc_numbers(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_8484 => {
            set_loc_xz(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_8514 => u64::from(point_line(e, v[0], v[1], v[2], v[3], v[4])),
        0x021B_854C => u64::from(try_line(
            e,
            v[0],
            v[1],
            v[2],
            v[3] as i32,
            v[4] as i32,
            v[5] as i32,
            v[6] as i32,
        )),
        0x021B_8658 => {
            set_line(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021B_8674 => u64::from(starts_at(e, v[0], v[1], v[2])),
        0x021B_868C => u64::from(ends_at(e, v[0], v[1], v[2])),
        0x021B_86A4 => {
            line_bounds(e, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        0x021B_87BC => {
            clear_rd(e, v[0]);
            0
        }
        0x021B_87C8 => {
            copy_data(e, v[0], v[1]);
            0
        }
        0x021B_87D8 => u64::from(point_rec(e, v[0], v[1])),
        0x021B_87FC => u64::from(line_rec(e, v[0], v[1])),
        0x021B_8820 => u64::from(cam_rec(e, v[0], v[1])),
        0x021B_8848 => u64::from(line_rec_rec(e, v[0], v[1])),
        0x021B_8870 => u64::from(cam_proc(e, v[0], v[1])),
        0x021B_8894 => u64::from(pos_proc(e, v[0], v[1])),
        0x021B_88B8 => u64::from(hit_proc(e, v[0], v[1])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field rail`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("rail", SPEC, PORTED, RECORDED), &call);
}

/// The embedded ROM data equals the overlay (`ROM_DEFAULTS`, `STEP_TABLE`).
#[test]
fn rom_tables_match_overlay() {
    let Some(rd) = overlay21() else { return };
    for (i, w) in ROM_DEFAULTS.iter().enumerate() {
        assert_eq!(
            *w,
            rd(ROM_DEFAULTS_ADDR + 4 * i as u32),
            "ROM_DEFAULTS[{i}]"
        );
    }
    let w = rd(0x021D_E298);
    for (i, b) in STEP_TABLE.iter().take(4).enumerate() {
        assert_eq!(u32::from(*b), (w >> (8 * i)) & 0xFF, "STEP_TABLE[{i}]");
    }
    assert_eq!(u32::from(STEP_TABLE[4]), rd(0x021D_E29C) & 0xFF);
}
