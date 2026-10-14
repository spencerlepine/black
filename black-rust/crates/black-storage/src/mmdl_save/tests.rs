use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200EE48, "a37b211ef50e77b1"),
    (0x0200EE50, "07b614110515decd"),
    (0x0200EED8, "e81616f3b9f302db"),
    (0x0200EF78, "9a61605db79f10ca"),
];
// fndiff-recorded-end

/// The OBJ-data load routed to the spec's stub line.
struct Rom<'a>(&'a mut StubEnv);

impl MmdlHooks for Rom<'_> {
    fn obj_data(&mut self, sys: u32, code: u32, dst: &mut [u8]) {
        self.0
            .call(0x0216_E2E8, &mut [Arg::V(sys), Arg::V(code), Arg::Out(dst)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_EE48 => return MmdlSave::size() as u64,
        0x0200_EE50 => {
            let mut s = MmdlSave::from_bytes(&c.bufs[0]);
            s.init();
            c.bufs[0] = s.to_bytes();
        }
        0x0200_EED8 => {
            let m = c.bufs[0].clone();
            pack(&m, &mut c.bufs[1]);
        }
        0x0200_EF78 => {
            let rec = c.bufs[1].clone();
            unpack(&mut c.bufs[0], &rec, v[2], &mut Rom(&mut c.env));
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
    0
}

/// Every ported function against the original (`tools/check-fn.sh black-storage mmdl_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "mmdl_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/mmdl_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// OBJ data that writes nothing.
struct NoObj;

impl MmdlHooks for NoObj {
    fn obj_data(&mut self, _sys: u32, _code: u32, _dst: &mut [u8]) {}
}

#[test]
fn pack_unpack_keeps_the_saved_fields() {
    let mut m = vec![0u8; MMDL_SIZE];
    for (i, b) in m.iter_mut().enumerate() {
        *b = (i * 7 + 3) as u8;
    }
    let mut rec = vec![0u8; RECORD_SIZE];
    pack(&m, &mut rec);
    let mut back = vec![0u8; MMDL_SIZE];
    unpack(&mut back, &rec, 0x1234, &mut NoObj);
    let mut rec2 = vec![0u8; RECORD_SIZE];
    pack(&back, &mut rec2);
    // status bit 0 and the +0x04 word differ by design; the rest of the record survives
    assert_eq!(&rec[4..], &rec2[4..]);
    assert_eq!(
        MmdlSave::from_bytes(&MmdlSave::default().to_bytes())
            .records
            .len(),
        RECORDS
    );
}
