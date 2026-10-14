use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200E730, "530ab80ab19aeff9"),
    (0x0200E734, "6bae85aaf95c9b99"),
    (0x0200E740, "7a6aea0bf9230b2f"),
    (0x0200E74C, "9ab9a7303e76737c"),
    (0x0200E76C, "fdbf92703453c35d"),
    (0x0200E780, "a4d7a595a030893a"),
    (0x0200E788, "121e781c31e67de4"),
    (0x0200E7A4, "35c933781b9f2a37"),
    (0x0200E7AC, "bc21700c442e734a"),
    (0x0200E7B4, "cb9d1b5dc120aa2c"),
    (0x0200E7BC, "5b7d29fab1b5c710"),
];
// fndiff-recorded-end

/// The callees routed to the spec's stub lines (the stubs write nothing into the record).
struct Rom<'a> {
    env: &'a mut StubEnv,
    msg: u32,
    /// B18b: emulator address of the record's name (`dst`, a `ptr` stub argument): the port fills
    /// its own copy of the record, which stands for record `i` of the block (`arg0`) at +4.
    dst: u32,
}

impl PokeRecordHooks for Rom<'_> {
    fn param(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::P])
    }
    fn nickname(&mut self, mon: u32, _dst: &mut [u16; NAME_LEN]) {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(116), Arg::P]);
    }
    fn species_name(&mut self, species: u32, _dst: &mut [u16; NAME_LEN]) {
        let mut buf = [0u8; 22];
        self.env.call(
            0x0204_5F5C,
            &mut [
                Arg::V(self.msg),
                Arg::V(species),
                Arg::Out(&mut buf),
                Arg::V(11),
            ],
        );
        self.env.call(
            0x0201_F41C,
            &mut [Arg::In(&buf), Arg::Ptr(self.dst), Arg::V(11)],
        );
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_E730 => return PokeRecords::size() as u64,
        0x0200_E788 => return PokeRecords::message(v[1], v[2]) as u64,
        _ => {}
    }
    let mut p = PokeRecords::from_bytes(&c.bufs[0]);
    let i = v.get(1).copied().unwrap_or(0) as usize;
    let ret: u64 = match addr {
        0x0200_E734 => {
            p.init();
            0
        }
        0x0200_E740 => p.filled(i) as u64,
        0x0200_E74C => p.level(i) as u64,
        0x0200_E76C => p.has_1b(i) as u64,
        0x0200_E780 => p.b1b(i) as u64,
        0x0200_E7A4 => p.pid(i) as u64,
        0x0200_E7AC => {
            let _ = p.name(i);
            (RECORD_SIZE * i + 4) as u64
        }
        0x0200_E7B4 => p.id(i) as u64,
        0x0200_E7BC => {
            let msg = u32::from_le_bytes(c.globals[0][..4].try_into().unwrap());
            p.fill(
                i,
                v[2],
                &mut Rom {
                    env: &mut c.env,
                    msg,
                    dst: black_fndiff::buf_addr(0) + (RECORD_SIZE * i + 4) as u32,
                },
            );
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = p.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage poke_records`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "poke_records",
            include_str!("../../../../tools/rom-diff/specs/black-storage/poke_records.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn level_window() {
    let mut p = PokeRecords::default();
    p.records[0].b1a = 3 << 1 | 1;
    p.records[1].b1a = 99 << 1;
    assert_eq!((p.level(0), p.level(1), p.filled(0)), (65, 100, 1));
    assert_eq!(PokeRecords::from_bytes(&p.to_bytes()), p);
}
