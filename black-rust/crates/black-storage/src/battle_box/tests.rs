use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200BA78, "d85f686e076a6185"),
    (0x0200BA80, "b5ff11655b1adfe8"),
    (0x0200BAD4, "c5c8211520db3621"),
    (0x0200BAF4, "fb0589dd806f5554"),
    (0x0200BB58, "72252fa1d4c00334"),
    (0x0200BBA8, "cb966f375e7a8ead"),
    (0x0200BBC8, "38ad58451fc678ee"),
    (0x0200BBD8, "6b9405a124608233"),
    (0x0200BBEC, "847adc24af4c5f71"),
    (0x0200BBFC, "3b1ce14dae723771"),
];
// fndiff-recorded-end

/// Fixed emulator address of pointer argument 2 (the box-part arena of `0x0200BB58`).
const ARG2: u32 = 0x02E3_0000;

/// The callees routed to the spec's stub lines.
struct Rom<'a> {
    env: &'a mut StubEnv,
    arena: Vec<u8>,
}

impl BattleBoxHooks for Rom<'_> {
    fn msg_open(&mut self) -> u32 {
        self.env.call(
            0x0204_5B38,
            &mut [Arg::V(0), Arg::V(2), Arg::V(9), Arg::V(1)],
        )
    }
    fn msg_name(&mut self, msg: u32, name: &mut [u8; NAME_SIZE]) {
        self.env.call(
            0x0204_5F5C,
            &mut [Arg::V(msg), Arg::V(6), Arg::Out(name), Arg::V(0x14)],
        );
    }
    fn msg_close(&mut self, msg: u32) {
        self.env.call(0x0204_5C04, &mut [Arg::V(msg)]);
    }
    fn box_param(&mut self, mon: &[u8], id: u32) -> u32 {
        self.env
            .call(0x0201_7E88, &mut [Arg::In(mon), Arg::V(id), Arg::V(0)])
    }
    fn party_alloc(&mut self, heap: u32) -> u32 {
        self.env.call(0x0201_A920, &mut [Arg::V(heap)])
    }
    fn party_init(&mut self, party: u32, n: u32) {
        self.env.call(0x0201_A954, &mut [Arg::V(party), Arg::V(n)]);
    }
    fn party_add(&mut self, party: u32, mon: u32) {
        self.env
            .call(0x0201_A98C, &mut [Arg::V(party), Arg::V(mon)]);
    }
    fn party_count(&mut self, party: u32) -> u32 {
        self.env.call(0x0201_AA18, &mut [Arg::V(party)])
    }
    fn party_mon(&mut self, party: u32, i: u32) -> u32 {
        self.env.call(0x0201_AB28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn box_part(&mut self, mon: u32) -> [u8; MON_SIZE] {
        let p = (self.env.call(0x0201_85F4, &mut [Arg::V(mon)]) - ARG2) as usize;
        self.arena[p..p + MON_SIZE].try_into().unwrap()
    }
    fn party_mon_from_box(&mut self, mon: &[u8], heap: u32) -> u32 {
        self.env
            .call(0x0201_747C, &mut [Arg::In(mon), Arg::V(heap)])
    }
    fn free(&mut self, p: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(p)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_BA78 => return BattleBox::size() as u64,
        0x0200_BBEC => return BattleBox::slot_offset(v[1], v[2]) as u64,
        _ => {}
    }
    let mut b = BattleBox::from_bytes(&c.bufs[0]);
    let arena = c.bufs.get(2).cloned().unwrap_or_default();
    let h = &mut Rom {
        env: &mut c.env,
        arena,
    };
    let ret: u64 = match addr {
        0x0200_BA80 => {
            b.init(h);
            0
        }
        0x0200_BAD4 => {
            let src = c.bufs[3].clone();
            b.set_mon(v[2] as usize, &src);
            0
        }
        0x0200_BAF4 => b.to_party(v[1], h) as u64,
        0x0200_BB58 => {
            b.from_party(v[1], h);
            0
        }
        0x0200_BBA8 => b.has_team(h) as u64,
        0x0200_BBC8 => {
            b.set_flags(v[1] as u16);
            0
        }
        0x0200_BBD8 => {
            b.clear_flags(v[1] as u16);
            0
        }
        0x0200_BBFC => b.test_flags(v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = b.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage battle_box`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "battle_box",
            include_str!("../../../../tools/rom-diff/specs/black-storage/battle_box.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn flags_and_round_trip() {
    let mut b = BattleBox::default();
    b.set_flags(5);
    b.clear_flags(1);
    assert!(b.test_flags(4) && !b.test_flags(1));
    b.set_mon(2, &[3; MON_SIZE]);
    assert_eq!(b.mon(2)[0], 3);
    assert_eq!(BattleBox::from_bytes(&b.to_bytes()), b);
}
