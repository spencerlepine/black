use super::*;
use crate::hooks::{HeapHooks, Mem, MemEnv};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020142D0, "f7c62135cf409775"),
    (0x020142D8, "6aaac4be41d5dc5f"),
    (0x020142E8, "46150ba05149e7be"),
    (0x02014314, "59a10934006a96f6"),
    (0x0201433C, "cb75fa58484ed01a"),
    (0x02014368, "40f0f310f42aa6e8"),
    (0x02014388, "7889a8435d80202e"),
    (0x020143CC, "c6b6cccb63c2f2bd"),
    (0x020143D8, "f3cb7c1315c0a6cd"),
    (0x020143F0, "33dda04e88181770"),
    (0x0201440C, "8dabebe6be449d55"),
    (0x0201441C, "290fb5d00cd0e082"),
    (0x02014428, "9f8615763d07f979"),
    (0x02014444, "32d5bf843b0dc7df"),
    (0x0201444C, "9e075381777ccea7"),
    (0x02014450, "d09227f1bddba83d"),
    (0x02014454, "199be5c30d7a3f4e"),
    (0x02014458, "f36f0367c6f8a806"),
    (0x0201445C, "5f5696411a2c18e0"),
    (0x02014460, "8dd3282cc809da7e"),
    (0x02014464, "11e34d5fe44de039"),
    (0x02014468, "ca58e4d78cf97cd6"),
    (0x0201446C, "46de78ea79ce73ee"),
    (0x02014470, "bb2054e60fc352d2"),
    (0x02014474, "9ef5014929f5b784"),
    (0x02014478, "5e79bd8d988fbc55"),
    (0x0201447C, "2e9ec5f641c8a866"),
    (0x02014484, "222d2e309fe7dccf"),
];
// fndiff-recorded-end

/// The heap stubs (`FUN_02030734` allocate, `FUN_020307B0` free) of the spec, with the buffer of
/// argument 1 as the memory window.
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
        let mut a = [heap, size, flag, file, line].map(Arg::V);
        self.stubs.call(0x0203_0734, &mut a)
    }
    fn heap_free(&mut self, block: u32) {
        self.stubs.call(0x0203_07B0, &mut [Arg::V(block)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let ew = c.vals.first().copied().unwrap_or(0);
    let v1 = c.vals.get(1).copied().unwrap_or(0);
    let v2 = c.vals.get(2).copied().unwrap_or(0);
    match addr {
        0x0201_4388 => flag_addr(ew, v1) as u64,
        0x0201_42E8 => check_flag(&c.bufs[0], &c.globals[0], v1) as u64,
        0x0201_4314 => {
            set_flag(&mut c.bufs[0], &mut c.globals[0], v1);
            0
        }
        0x0201_433C => {
            clear_flag(&mut c.bufs[0], &mut c.globals[0], v1);
            0
        }
        0x0201_43CC => var_addr(ew, v1) as u64,
        0x0201_42D0 => event_work_size() as u64,
        0x0201_42D8 => {
            event_work_clear(&mut c.bufs[0]);
            0
        }
        0x0201_4368 => {
            clear_flag_range(&mut c.bufs[0], &mut c.globals[0], v1 as i32, v2 as i32);
            0
        }
        0x0201_43D8 => {
            clear_vars(&mut c.bufs[0], v1, v2);
            0
        }
        0x0201_43F0 => {
            clear_temp_flags(&mut c.bufs[0], &mut c.globals[0]);
            0
        }
        0x0201_440C => take_temp_marker(&mut c.bufs[0]) as u64,
        0x0201_441C => table_0209e050(ew) as u64,
        0x0201_4428 => {
            let mut rom = Rom {
                stubs: &mut c.env,
                mem: Mem::new(c.vals[1], std::mem::take(&mut c.bufs[1])),
            };
            let b = field_status_new(&mut rom, ew);
            c.bufs[1] = std::mem::take(&mut rom.mem.bytes);
            // `ret=ofs:arg1`: the port reports the offset into the buffer
            b.wrapping_sub(c.vals[1]) as u64
        }
        0x0201_4444 => {
            let mut rom = Rom {
                stubs: &mut c.env,
                mem: Mem::default(),
            };
            field_status_free(&mut rom, ew);
            0
        }
        0x0201_444C => fs_get_0(&c.bufs[0]) as u64,
        0x0201_4450 => {
            fs_set_0(&mut c.bufs[0], v1);
            0
        }
        0x0201_4454 => {
            fs_set_4(&mut c.bufs[0], v1 as u8);
            0
        }
        0x0201_4458 => fs_get_4(&c.bufs[0]) as u64,
        0x0201_445C => {
            fs_set_5(&mut c.bufs[0], v1 as u8);
            0
        }
        0x0201_4460 => fs_get_5(&c.bufs[0]) as u64,
        0x0201_4464 => {
            fs_set_6(&mut c.bufs[0], v1 as u8);
            0
        }
        0x0201_4468 => fs_get_6(&c.bufs[0]) as u64,
        0x0201_446C => {
            fs_set_8(&mut c.bufs[0], v1 as u16);
            0
        }
        0x0201_4470 => fs_get_8(&c.bufs[0]) as u64,
        0x0201_4474 => {
            fs_set_12(&mut c.bufs[0], v1 as u16);
            0
        }
        0x0201_4478 => fs_get_12(&c.bufs[0]) as u64,
        0x0201_447C => fs_swap_11(&mut c.bufs[0], v1 as u8) as u64,
        0x0201_4484 => fs_get_11(&c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field event_work`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "event_work",
            include_str!("../../../../tools/rom-diff/specs/black-field/event_work.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

fn fresh() -> (Vec<u8>, [u8; SYS_FLAGS_SIZE]) {
    (vec![0u8; EVENT_WORK_SIZE], [0u8; SYS_FLAGS_SIZE])
}

#[test]
fn flag_set_check_clear_round_trip() {
    let (mut w, mut s) = fresh();
    for id in [1u32, 7, 8, 0x96D, 0xAA0, 0xB5F] {
        assert!(!check_flag(&w, &s, id), "{id:#x} starts clear");
        set_flag(&mut w, &mut s, id);
        assert!(check_flag(&w, &s, id), "{id:#x} set");
        clear_flag(&mut w, &mut s, id);
        assert!(!check_flag(&w, &s, id), "{id:#x} cleared");
        assert!(
            w.iter().all(|&b| b == 0),
            "{id:#x} touched only its own bit"
        );
    }
}

#[test]
fn flag_bits_are_lsb_first_in_the_flag_block() {
    let (mut w, mut s) = fresh();
    set_flag(&mut w, &mut s, 0x96D); // byte 0x12D, bit 5
    assert_eq!(w[FLAGS_OFS + 0x12D], 1 << 5);
    set_flag(&mut w, &mut s, 1);
    assert_eq!(w[FLAGS_OFS], 2);
    // neighbouring bits are independent
    set_flag(&mut w, &mut s, 0x96C);
    clear_flag(&mut w, &mut s, 0x96D);
    assert_eq!(w[FLAGS_OFS + 0x12D], 1 << 4);
    // the last valid flag is the last bit of the last flag byte
    set_flag(&mut w, &mut s, 0xB5F);
    assert_eq!(w[FLAGS_OFS + 0x16B], 0x80);
}

#[test]
fn ids_without_a_byte_are_ignored() {
    let (mut w, mut s) = fresh();
    for id in [0u32, 0xB60, 0xFFFF, 0x3FFF, 0x4040, 0x7FFF_FFFF] {
        assert_eq!(flag_slot(id), None, "{id:#x}");
        assert_eq!(flag_addr(0x0230_0000, id), 0);
        set_flag(&mut w, &mut s, id);
        assert!(!check_flag(&w, &s, id));
    }
    assert!(w.iter().all(|&b| b == 0));
    assert_eq!(s, [0; SYS_FLAGS_SIZE]);
}

#[test]
fn system_flags_live_in_the_ram_table() {
    let (mut w, mut s) = fresh();
    assert_eq!(flag_slot(0x4000), Some(FlagSlot::Sys(0)));
    assert_eq!(flag_slot(0x403F), Some(FlagSlot::Sys(7)));
    assert_eq!(flag_addr(0x0230_0000, 0x4009), SYS_FLAGS_ADDR + 1);
    set_flag(&mut w, &mut s, 0x4009);
    assert_eq!(s[1], 2);
    assert!(check_flag(&w, &s, 0x4009));
    assert!(w.iter().all(|&b| b == 0));
    clear_flag(&mut w, &mut s, 0x4009);
    assert_eq!(s, [0; SYS_FLAGS_SIZE]);
}

#[test]
fn flag_address_is_the_object_plus_the_byte() {
    assert_eq!(flag_addr(0x0230_0000, 1), 0x0230_027C);
    assert_eq!(flag_addr(0x0230_0000, 0xB5F), 0x0230_027C + 0x16B);
    // the original does not check the low end of the system table: id 0x8000_4000 maps below it
    assert_eq!(flag_slot(0x8000_4000), Some(FlagSlot::Sys(-0x1000_0000)));
}

#[test]
fn vars_are_halfwords_from_0x4000() {
    let (mut w, _) = fresh();
    assert_eq!(var_addr(0x0230_0000, 0x4000), 0x0230_0000);
    assert_eq!(var_addr(0x0230_0000, 0x4005), 0x0230_000A);
    assert_eq!(var_offset(0x4000), Some(0));
    assert_eq!(var_offset(0x4000 + VAR_COUNT - 1), Some(FLAGS_OFS - 2));
    assert_eq!(var_offset(0x4000 + VAR_COUNT), None);
    assert_eq!(var_offset(0x3FFF), None);
    assert!(var_set(&mut w, 0x4005, 0xBEEF));
    assert_eq!(var_get(&w, 0x4005), Some(0xBEEF));
    assert_eq!(&w[10..12], [0xEF, 0xBE]);
    assert!(!var_set(&mut w, 0x3FFF, 1));
    assert_eq!(var_get(&w, 0x3FFF), None);
    // vars do not overlap the flag block
    set_flag(&mut w, &mut [0; 8], 1);
    assert_eq!(var_get(&w, 0x4000 + VAR_COUNT - 1), Some(0));
}

#[test]
fn temp_flags_clear_only_their_range_and_set_the_marker() {
    let (mut w, mut s) = fresh();
    for id in [0xA9Fu32, 0xAA0, 0xB5F, 0x100] {
        set_flag(&mut w, &mut s, id);
    }
    assert_eq!(take_temp_marker(&mut w), 0);
    clear_temp_flags(&mut w, &mut s);
    assert!(check_flag(&w, &s, 0xA9F) && check_flag(&w, &s, 0x100));
    assert!(!check_flag(&w, &s, 0xAA0) && !check_flag(&w, &s, 0xB5F));
    assert_eq!(take_temp_marker(&mut w), 1);
    assert_eq!(take_temp_marker(&mut w), 0);
}

#[test]
fn flag_range_is_signed_and_inclusive() {
    let (mut w, mut s) = fresh();
    for id in 1..=8u32 {
        set_flag(&mut w, &mut s, id);
    }
    clear_flag_range(&mut w, &mut s, 3, 5);
    let left: Vec<u32> = (1..=8).filter(|&i| check_flag(&w, &s, i)).collect();
    assert_eq!(left, [1, 2, 6, 7, 8]);
    clear_flag_range(&mut w, &mut s, 5, 3); // empty
    assert_eq!(
        left.len(),
        (1..=8).filter(|&i| check_flag(&w, &s, i)).count()
    );
}

#[test]
fn vars_clear_is_a_halfword_range() {
    let (mut w, _) = fresh();
    for id in 0x4000..0x4008u32 {
        var_set(&mut w, id, 0xFFFF);
    }
    clear_vars(&mut w, 0x4002, 0x4005);
    let got: Vec<u16> = (0x4000..0x4008u32)
        .map(|i| var_get(&w, i).unwrap())
        .collect();
    assert_eq!(got, [0xFFFF, 0xFFFF, 0, 0, 0, 0xFFFF, 0xFFFF, 0xFFFF]);
    event_work_clear(&mut w);
    assert!(w.iter().all(|&b| b == 0));
    assert_eq!(event_work_size(), 0x3EA);
}

#[test]
fn field_status_accessors_use_their_offsets() {
    let mut fs = [0u8; 0x14];
    fs_set_0(&mut fs, 0x0102_0304);
    fs_set_4(&mut fs, 5);
    fs_set_5(&mut fs, 6);
    fs_set_6(&mut fs, 7);
    fs_set_8(&mut fs, 0x0908);
    fs_set_12(&mut fs, 0x0B0A);
    assert_eq!(fs_swap_11(&mut fs, 0xCC), 0);
    assert_eq!(fs_get_11(&fs), 0xCC);
    assert_eq!(&fs[..9], [4, 3, 2, 1, 5, 6, 7, 0, 8]);
    assert_eq!(
        (fs_get_0(&fs), fs_get_4(&fs), fs_get_5(&fs), fs_get_6(&fs)),
        (0x0102_0304, 5, 6, 7)
    );
    assert_eq!((fs_get_8(&fs), fs_get_12(&fs)), (0x0908, 0x0B0A));
}

#[test]
fn rom_table_starts_with_the_known_entries() {
    assert_eq!([0, 1, 2, 3].map(table_0209e050), [1, 3, 4, 2]);
}
