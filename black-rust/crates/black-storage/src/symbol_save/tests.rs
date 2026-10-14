use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200E090, "a0d6b56199ef4aa1"),
    (0x0200E098, "9c0669a40a46b301"),
    (0x0200E0BC, "8d613ecadac86980"),
    (0x0200E0E8, "ad43196144e82ad2"),
    (0x0200E124, "39cfcf7ec54e2b97"),
    (0x0200E164, "07c9c9b037b9f7e9"),
    (0x0200E184, "77a2f2652001f4e9"),
    (0x0200E18C, "0aa2cb6271ce6f61"),
    (0x0200E1A8, "90dce32db706d8e0"),
    (0x0200E27C, "32298a12bb4c0e76"),
];
// fndiff-recorded-end

/// Fixed emulator address of the block (pointer argument 0).
const BLOCK_ADDR: u32 = 0x02E1_0000;

/// The cipher calls routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl SymbolCrypt for Rom<'_> {
    fn seal(&mut self) {
        self.0.call(0x0200_E024, &mut [Arg::V(BLOCK_ADDR)]);
    }
    fn open(&mut self) {
        self.0.call(0x0200_E06C, &mut [Arg::V(BLOCK_ADDR)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_E090 => return SymbolSave::size() as u64,
        0x0200_E0BC => return SymbolSave::zone_of(v[0]) as u64,
        0x0200_E184 => return SymbolSave::zone_of_index(v[0]) as u64,
        _ => {}
    }
    let mut s = SymbolSave::from_bytes(&c.bufs[0]);
    let h = &mut Rom(&mut c.env);
    let ret = match addr {
        0x0200_E098 => {
            s.init(h);
            0
        }
        0x0200_E0E8 => s.find_free_raw(v[1]),
        0x0200_E124 => {
            s.remove_raw(v[1]);
            0
        }
        0x0200_E164 => s.find_free(v[1], h),
        0x0200_E18C => {
            s.remove(v[1], h);
            0
        }
        0x0200_E1A8 => {
            let r = SymbolRecord {
                species: v[1],
                a: v[2],
                b: v[3],
                c: v[4],
                d: v[5],
            };
            s.add(r, v[6], h);
            0
        }
        0x0200_E27C => {
            s.set_zone3_level(v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage symbol_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "symbol_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/symbol_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn full_zone_drops_the_oldest() {
    let mut s = SymbolSave::default();
    s.init(&mut Plain);
    for sp in 1..=11 {
        s.add(
            SymbolRecord {
                species: sp,
                ..Default::default()
            },
            0,
            &mut Plain,
        );
    }
    let species: Vec<u32> = s.records[..10].iter().map(|r| r & 0x7FF).collect();
    assert_eq!(species, (2..=11).collect::<Vec<_>>());
    assert_eq!(s.find_free_raw(0), NO_SLOT);
    assert_eq!(s.find_free_raw(1), 10);
}
