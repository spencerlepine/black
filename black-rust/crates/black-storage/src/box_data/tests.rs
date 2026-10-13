use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02007454, "653340130dec827e"),
    (0x0200745C, "82fb80c07f27a3f5"),
    (0x02007464, "194ca59441f6c97b"),
    (0x020074CC, "643348e0c785ef0b"),
    (0x020074D8, "cac2fd95f534aa7f"),
    (0x0200752C, "c8e7c3d8f5043cdb"),
    (0x020075B0, "0e336ab715ae267b"),
    (0x02007610, "1aee07a0104cb50f"),
    (0x02007664, "2ba4830ad37307b8"),
    (0x02007698, "31654f51f473d756"),
    (0x020076A4, "0a31c984e1c5c877"),
    (0x020076B4, "756d849d7be9555a"),
    (0x020076D0, "e96de003b2d44f51"),
    (0x02007774, "4875fb05dff8b5c8"),
    (0x020077B8, "f9e7de67e38e5614"),
    (0x0200780C, "2e7546a542c36359"),
    (0x0200781C, "60b3861ceedd5fef"),
    (0x02007838, "ff3d834cb0f8fe09"),
    (0x0200785C, "264d51449af7df2b"),
    (0x02007898, "f78c137c30fed99a"),
    (0x020078C4, "b5389312325c402d"),
    (0x0200791C, "56abdfaf16be336d"),
    (0x02007984, "f744cc27f899a002"),
    (0x020079A0, "737266b32654cbb7"),
    (0x020079BC, "0a62e15e3bc05e8c"),
    (0x02007A18, "19ad0ac22358d13c"),
    (0x02007A68, "d99b644af06cd28e"),
    (0x02007A80, "6841b8f2d302f763"),
    (0x02007AA4, "869da1adfe4926ec"),
    (0x02007AC8, "1764f0dacd36af90"),
    (0x02007AD0, "d64d479feb539e1d"),
    (0x02007AEC, "650b08180254e435"),
    (0x02007AF4, "6e92c455eec649f8"),
];
// fndiff-recorded-end

/// The unported callees of the box area routed to the spec's stub lines, and the case's buffers
/// as the memory the port reads and writes (each block is found by its address).
struct Rom<'a> {
    stubs: &'a mut StubEnv,
    blocks: Vec<(u32, &'a mut [u8])>,
}

fn rom(c: &mut Case) -> Rom<'_> {
    let blocks = c
        .bufs
        .iter_mut()
        .zip(c.vals.iter())
        .filter(|(b, _)| !b.is_empty())
        .map(|(b, &a)| (a, b.as_mut_slice()))
        .collect();
    Rom {
        stubs: &mut c.env,
        blocks,
    }
}

impl BoxEnv for Rom<'_> {
    fn meta_addr(&mut self, mgr: u32) -> u32 {
        self.stubs.call(0x0200_74CC, &mut [Arg::V(mgr)])
    }
    fn tray_addr(&mut self, mgr: u32, tray: u32) -> u32 {
        self.stubs
            .call(0x0200_7AF4, &mut [Arg::V(mgr), Arg::V(tray)])
    }
    fn mem(&mut self, addr: u32) -> &mut [u8] {
        let i = self
            .blocks
            .iter()
            .position(|(base, b)| (addr.wrapping_sub(*base) as usize) < b.len())
            .unwrap_or_else(|| panic!("address {addr:#010x} is in no buffer of the case"));
        let (base, b) = &mut self.blocks[i];
        &mut b[addr.wrapping_sub(*base) as usize..]
    }
    fn save_block(&mut self, savedata: u32, block: u32) -> u32 {
        self.stubs
            .call(0x0200_71CC, &mut [Arg::V(savedata), Arg::V(block)])
    }
    fn poke_get(&mut self, mon: u32, id: u32, buf: u32) -> u32 {
        self.stubs
            .call(0x0201_7E88, &mut [Arg::V(mon), Arg::V(id), Arg::V(buf)])
    }
    fn clear_mon(&mut self, mon: u32) {
        self.stubs.call(0x0201_7550, &mut [Arg::V(mon)]);
    }
    fn fixup_mon(&mut self, mon: u32) {
        self.stubs.call(0x0201_81A0, &mut [Arg::V(mon)]);
    }
    fn meta_init(&mut self, meta: u32) {
        self.stubs.call(0x0200_7464, &mut [Arg::V(meta)]);
    }
    fn put_tray(&mut self, mgr: u32, tray: u32, mon: u32) -> u32 {
        self.stubs
            .call(0x0200_752C, &mut [Arg::V(mgr), Arg::V(tray), Arg::V(mon)])
    }
    fn occupied_count(&mut self, mgr: u32, tray: u32) -> u32 {
        self.stubs
            .call(0x0200_78C4, &mut [Arg::V(mgr), Arg::V(tray)])
    }
    fn non_egg_count(&mut self, mgr: u32, tray: u32) -> u32 {
        self.stubs
            .call(0x0200_791C, &mut [Arg::V(mgr), Arg::V(tray)])
    }
    fn name_to_str(&mut self, dst: u32, name: &[u8]) {
        self.stubs
            .call(0x0204_5924, &mut [Arg::V(dst), Arg::In(name)]);
    }
    fn str_to_name(&mut self, src: u32, name: &mut [u8], n: u32) {
        self.stubs
            .call(0x0204_5A38, &mut [Arg::V(src), Arg::Out(name), Arg::V(n)]);
    }
    fn msg_string(&mut self, msg: u32, idx: u32, name: &mut [u8], n: u32) {
        self.stubs.call(
            0x0204_5F5C,
            &mut [Arg::V(msg), Arg::V(idx), Arg::Out(name), Arg::V(n)],
        );
    }
    fn msg_bank(&mut self) -> u32 {
        self.stubs.call(
            0x0204_5B38,
            &mut [Arg::V(0), Arg::V(2), Arg::V(9), Arg::V(1)],
        )
    }
    fn msg_free(&mut self, msg: u32) {
        self.stubs.call(0x0204_5C04, &mut [Arg::V(msg)]);
    }
    fn alloc(&mut self, a: [u32; 5]) -> u32 {
        self.stubs.call(
            0x0203_0734,
            &mut [
                Arg::V(a[0]),
                Arg::V(a[1]),
                Arg::V(a[2]),
                Arg::V(a[3]),
                Arg::V(a[4]),
            ],
        )
    }
    fn free(&mut self, ptr: u32) {
        self.stubs.call(0x0203_07B0, &mut [Arg::V(ptr)]);
    }
    fn assert_fail(&mut self) -> ! {
        // the assert's file name pointer, line 0, and the name again (the literal pool values)
        self.stubs.call(
            0x0203_4714,
            &mut [Arg::V(0x020A_7068), Arg::V(0), Arg::V(0x020A_7068)],
        );
        unreachable!("FUN_02034714 never returns")
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let vals = c.vals.clone();
    let v = |i: usize| vals.get(i).copied().unwrap_or(0);
    let mut e = rom(c);
    let e = &mut e;
    match addr {
        0x0200_745C => meta_size() as u64,
        0x0200_7AEC => tray_size() as u64,
        0x0200_7454 => {
            reset_meta(e, v(0));
            0
        }
        0x0200_7464 => {
            init_meta(e, v(0));
            0
        }
        0x0200_74CC => meta_block(e, v(0)).wrapping_sub(v(1)) as u64,
        0x0200_74D8 => put_mon(e, v(0), v(1)) as u64,
        0x0200_752C => put_tray(e, v(0), v(1), v(2)) as u64,
        0x0200_75B0 => put_slot(e, v(0), v(1), v(2), v(3)) as u64,
        0x0200_7610 => {
            swap_mons(e, v(0), v(1), v(2), v(3), v(4));
            0
        }
        0x0200_7664 => {
            clear_slot(e, v(0), v(1), v(2));
            0
        }
        0x0200_7698 => current_tray(e, v(0)) as u64,
        0x0200_76A4 => unlocked_count(e, v(0)) as u64,
        0x0200_76B4 => unlock_more(e, v(0)) as u64,
        0x0200_76D0 => find_free(e, v(0), v(1), v(2)) as u64,
        0x0200_7774 => count_free_all(e, v(0)) as u64,
        0x0200_77B8 => count_free_tray(e, v(0), v(1)) as u64,
        0x0200_780C => {
            set_current(e, v(0), v(1));
            0
        }
        0x0200_781C => wallpaper(e, v(0), v(1)) as u64,
        0x0200_7838 => {
            set_wallpaper(e, v(0), v(1), v(2) as u8);
            0
        }
        0x0200_785C => {
            name_get(e, v(0), v(1), v(2));
            0
        }
        0x0200_7898 => {
            name_set(e, v(0), v(1), v(2));
            0
        }
        0x0200_78C4 => occupied(e, v(0), v(1)) as u64,
        0x0200_791C => non_egg(e, v(0), v(1)) as u64,
        0x0200_7984 => sum_occupied(e, v(0)) as u64,
        0x0200_79A0 => sum_non_egg(e, v(0)) as u64,
        0x0200_79BC => mon_para_get(e, v(0), v(1), v(2), v(3), v(4)) as u64,
        // `ret=ofs:argK`: the spec compares the pointer as an offset into argument K's buffer
        0x0200_7A18 => slot_ptr(e, v(0), v(1), v(2)).wrapping_sub(v(4)) as u64,
        0x0200_7A68 => {
            flags_set(e, v(0), v(1) as u8);
            0
        }
        0x0200_7A80 => flags_test(e, v(0), v(1) as u8) as u64,
        0x0200_7AA4 => new_mgr(e, v(0), v(1)).wrapping_sub(v(2)) as u64,
        0x0200_7AC8 => {
            free_mgr(e, v(0));
            0
        }
        0x0200_7AD0 => {
            clear_tray(e, v(0));
            0
        }
        0x0200_7AF4 => tray_block(e, v(0), v(1)).wrapping_sub(v(2)) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original
/// (`tools/check-fn.sh black-storage box_data`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "box",
            include_str!("../../../../tools/rom-diff/specs/black-storage/box_data.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn layout_constants() {
    assert_eq!(SLOTS * MON_SIZE, TRAY_SIZE);
    assert_eq!(NAMES_OFS + MAX_BOXES as usize * NAME_SIZE, WALLPAPER_OFS);
    assert_eq!(WALLPAPER_OFS + MAX_BOXES as usize, FLAGS_OFS);
    assert_eq!(meta_size(), 0x3E0);
}
