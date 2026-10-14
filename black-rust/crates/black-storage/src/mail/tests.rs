use super::*;
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008EAC, "ee65bb5b827fb6ab"),
    (0x02008EF8, "d6994eceb4ef4d19"),
    (0x02008F3C, "799b538cb7ac97ee"),
    (0x02008F48, "c121f66ed90b5fe4"),
    (0x02008F80, "774fb07ca585bc8f"),
    (0x02008F84, "3e00789d87e85f95"),
    (0x02008F88, "fd85cb4d2883fe50"),
    (0x02008F9C, "0cf90b64dfad812d"),
    (0x02008FA0, "3f250f3b86d28ada"),
    (0x02008FA4, "49a5964bbd780fe4"),
    (0x02008FA8, "02fbfecc8fc92283"),
    (0x02008FB0, "e3d68bb3e539c972"),
    (0x02008FB4, "86b2eac08477ffd5"),
    (0x02008FB8, "419f09155c7d44fc"),
    (0x02008FBC, "d13bcba45c7df484"),
    (0x02008FC0, "9cd0f31fe322947f"),
    (0x02008FC4, "f7b874d1b5431c47"),
    (0x02008FC8, "e5984d1313188085"),
    (0x02008FD8, "a1903d1113cf679a"),
    (0x02008FFC, "1c7678c27c887bd5"),
    (0x02009004, "40e910dbf0e72307"),
    (0x02009020, "23bdc3ab55722f07"),
    (0x0200902C, "2ae2aeed38d7bcc8"),
    (0x0200903C, "47a837197a44e284"),
    (0x02009074, "5c455f519f36cfab"),
    (0x020090A0, "9e826a3a4e1203cd"),
];
// fndiff-recorded-end

const K: GameConsts = GameConsts::BLACK_US;

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8F3C => {
            let src = Mail::from_bytes(&c.bufs[0]);
            let mut dst = Mail::from_bytes(&c.bufs[1]);
            Mail::copy(&src, &mut dst);
            c.bufs[1] = dst.to_bytes().to_vec();
            return 0;
        }
        0x0200_8FFC => return MailBox::size() as u64,
        0x0200_9004..=0x0200_90A0 => {
            let mut b = MailBox::from_bytes(&c.bufs[0]);
            let ret = match addr {
                0x0200_9004 => {
                    b.init(K);
                    0
                }
                0x0200_9020 => b.find_empty() as u32,
                0x0200_9074 => b.find_empty_in(v[1] as i32) as u32,
                0x0200_902C => {
                    b.clear(v[2] as i32, K);
                    0
                }
                0x0200_903C => {
                    b.set(v[2] as i32, &Mail::from_bytes(&c.bufs[3]));
                    0
                }
                0x0200_90A0 => MailBox::slot(v[2] as i32).map_or(0, |i| (i * MAIL_SIZE) as u32),
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            };
            c.bufs[0] = b.to_bytes();
            return ret as u64;
        }
        _ => {}
    }
    let mut m = Mail::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_8EAC => {
            m.init(K);
            0
        }
        0x0200_8EF8 => m.is_valid() as u32,
        0x0200_8F48 => {
            // FUN_02012944(gamedata): the player status the port is handed directly.
            let got = c.env.call(0x0201_2944, &mut [Arg::V(v[3])]);
            assert_eq!(got, 0x02E5_0000);
            let status = MyStatus::from_bytes(&c.bufs[4]);
            m.create(K, v[1], &status);
            0
        }
        0x0200_8F80 => m.trainer_id(),
        0x0200_8F84 => {
            let _ = m.name();
            8
        }
        0x0200_8F88 => {
            m.set_name(&crate::le::r16s(&c.bufs[1], 0));
            0
        }
        0x0200_8F9C => m.gender(),
        0x0200_8FA0 => {
            m.set_gender(v[1]);
            0
        }
        0x0200_8FA4 => m.design(),
        0x0200_8FA8 => {
            m.set_design(v[1]);
            0
        }
        0x0200_8FB0 => m.language(),
        0x0200_8FB4 => {
            m.set_language(v[1]);
            0
        }
        0x0200_8FB8 => m.version(),
        0x0200_8FBC => {
            m.set_version(v[1]);
            0
        }
        0x0200_8FC0 => m.field_1e(),
        0x0200_8FC4 => {
            m.set_field_1e(v[1]);
            0
        }
        0x0200_8FC8 => 0x20 + 8 * Mail::pms_index(v[1]) as u32,
        0x0200_8FD8 => {
            m.set_pms(&Pms::from_bytes(&c.bufs[1]), v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = m.to_bytes().to_vec();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage mail`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "mail",
            include_str!("../../../../tools/rom-diff/specs/black-storage/mail.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_mailbox_is_empty() {
    let b = MailBox::new_game(K);
    assert_eq!(b.find_empty(), 0);
    let img = b.to_bytes();
    assert_eq!(img.len(), MAILBOX_SIZE);
    let m = &img[..MAIL_SIZE];
    assert_eq!(&m[..8], &[0, 0, 0, 0, 0, 2, 0x15, 0xFF]);
    assert!(m[8..0x18].iter().all(|&x| x == 0xFF));
    assert_eq!(&m[0x1E..0x22], &[0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(MailBox::from_bytes(&img), b);
}
