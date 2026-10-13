use super::*;
use crate::hooks::Mem;
use crate::testenv::{ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02159B48, "890bcb1673ea291d"),
    (0x02159B94, "e52c2052ee927173"),
    (0x02159BC4, "b5615728646bbd48"),
    (0x02159BD4, "8e361c93695a2fcf"),
    (0x02159BF4, "2a5ba9df95295a29"),
    (0x02159BF8, "245d2aac5f609694"),
    (0x02159BFC, "84c5cbda7a1faf6b"),
    (0x02159C00, "548b531c5a75554d"),
    (0x02159C20, "c1db8efc37a7273c"),
    (0x02159C2C, "76739da8447886e2"),
    (0x02159C48, "bd7eb775c9eb3cbe"),
    (0x02159C68, "954067d45a6ff32b"),
    (0x02159CC4, "5990505b4fc6fd47"),
    (0x02159D04, "9d6081127837d2e5"),
    (0x02159D68, "a09289ab0a748193"),
    (0x02159D84, "5f4dbe8e7a4f6f9d"),
    (0x02159DC8, "971a3a5b39926ca8"),
    (0x02159DCC, "40487355d9e705e7"),
    (0x02159DEC, "1e195b941806013b"),
    (0x02159E00, "9481eb1896b1fe9a"),
    (0x02159E60, "45c0a953cca7ee78"),
    (0x02159E8C, "e789ec6e6a42d992"),
    (0x02159F04, "1728fb7df3720545"),
    (0x02159F28, "58a8bb1cc12317b9"),
    (0x02159F54, "b52e08fd3c3a1ef2"),
    (0x02159F80, "0bd1687f60a4db8e"),
    (0x02159F84, "b992fc37defe2af1"),
    (0x02159F94, "9a945504096c67e8"),
    (0x02159FA8, "01c50e715f23e221"),
    (0x02159FE0, "a9848964e4fcb5a7"),
    (0x0215A014, "30e53fad5bc6310f"),
    (0x0215A048, "bb48cda08decba8e"),
    (0x0215A064, "63ddde7ed71806f1"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/map_matrix.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0215_9B48 => ofs_work(create(e, v[0])),
        0x0215_9B94 => {
            load(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_9BC4 => zone_at(&e.mem, v[0], v[1], v[2]) as u64,
        0x0215_9BD4 => zone_at_position(&e.mem, v[0], v[1] as i32, v[2] as i32) as u64,
        0x0215_9BF4 => width(&e.mem, v[0]) as u64,
        0x0215_9BF8 => height(&e.mem, v[0]) as u64,
        0x0215_9BFC => cell_count(&e.mem, v[0]) as u64,
        0x0215_9C00 => {
            set_size(&mut e.mem, v[0], v[1], v[2]);
            0
        }
        0x0215_9C20 => ofs_work(land_table(v[0])),
        0x0215_9C2C => contains(&e.mem, v[0], v[1] as i32, v[2] as i32) as u64,
        0x0215_9C48 => contains_position(&e.mem, v[0], v[1] as i32, v[2] as i32) as u64,
        0x0215_9C68 => {
            apply_replacement(e, v[0], v[1], v[2]);
            0
        }
        0x0215_9CC4 => {
            apply_replacements(e, v[0], v[1], v[2]);
            0
        }
        0x0215_9D04 => {
            load_from(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_9D68 => block_of(v[0] as i32) as u32 as u64,
        0x0215_9D84 => ofs_work(replace_create(e, v[0], v[1])),
        0x0215_9DC8 => replace_count(&e.mem, v[0]) as u64,
        0x0215_9DCC => replace_read(e, v[0], v[1]) as u64,
        0x0215_9DEC => {
            replace_free(e, v[0]);
            0
        }
        0x0215_9E00 => replacement_at(e, v[0], v[1], v[2]) as u64,
        0x0215_9E60 => var_key_index(v[0]) as u64,
        0x0215_9E8C => {
            replace_state(e, v[0], v[1]);
            0
        }
        0x0215_9F04 => var_entry(v[0], v[1]) as u64,
        0x0215_9F28 => {
            var_switch_set(e, v[0], v[1], v[2]);
            0
        }
        0x0215_9F54 => var_switch_on(e, v[0], v[1]) as u64,
        0x0215_9F80 => fly_default() as u64,
        0x0215_9F84 => fly_valid(v[0]) as u64,
        0x0215_9F94 => fly_zone(v[0]) as u64,
        0x0215_9FA8 => {
            fly_location(e, v[0], v[1]);
            0
        }
        0x0215_9FE0 => fly_point_of_zone(v[0]) as u64,
        0x0215_A014 => {
            fly_arrive(e, v[0], v[1]);
            0
        }
        0x0215_A048 => {
            set_location(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x0215_A064 => fly_index(v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field map_matrix`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("map_matrix", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn blocks_round_toward_zero() {
    assert_eq!(block_of(0), 0);
    assert_eq!(block_of(0x1F_FFFF), 0);
    assert_eq!(block_of(0x20_0000), 1);
    assert_eq!(block_of(-1), 0);
    assert_eq!(block_of(-0x20_0000), -1);
    assert_eq!(block_of(-0x20_0001), -1);
}

#[test]
fn zone_cells_are_row_major() {
    let mut bytes = vec![0u8; MATRIX_SIZE as usize];
    bytes[8] = 3; // width 3
    let cell = 2 + 3 * 3;
    bytes[(ZONE_OFS + 2 * cell) as usize] = 0x34;
    bytes[(ZONE_OFS + 2 * cell + 1) as usize] = 0x12;
    let mem = Mem::new(WORK, bytes);
    assert_eq!(zone_at(&mem, WORK, 2, 3), 0x1234);
    assert_eq!(zone_at(&mem, WORK, 0, 0), 0);
}
