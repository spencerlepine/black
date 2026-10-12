use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021EEA30, "4eb9f5f77b8c515b"),
    (0x021EEA54, "9280e7dec0439b85"),
    (0x021EEA58, "3e96161184ff3860"),
    (0x021EEAD0, "51fa1abc3ef75d2f"),
    (0x021EEAE0, "7cc4bfe2901d82b9"),
    (0x021EEAE8, "35deea9b2361b041"),
    (0x021EEAFC, "9280e7dec0439b85"),
    (0x021EEB00, "c29d20a828d51bf5"),
    (0x021EEB2C, "1ca365b5f73c8143"),
    (0x021EEB64, "a267b437c012ec88"),
    (0x021EEB70, "5031afa0f5cee9d6"),
    (0x021EEB84, "cbd5fb829b6dd153"),
    (0x021EEBA4, "ca76fac09bc200e0"),
    (0x021EEBDC, "178fccaa48d41699"),
    (0x021EEBE8, "c90a884c18521919"),
    (0x021EEC20, "9e7e4475c0971c70"),
    (0x021EEC4C, "13f2326071e61856"),
    (0x021EEC78, "485872f735d1d0c5"),
    (0x021EECD8, "0cce92097301a53a"),
    (0x021EED00, "7c74dea2fddff292"),
    (0x021EED24, "67a87533088bef14"),
    (0x021EED48, "cabc7ca6857a4296"),
    (0x021EED6C, "a3666137724d4f9d"),
    (0x021EED90, "24cb6e9e6feccc50"),
    (0x021EEE24, "651c944c8a4f7301"),
    (0x021EEE80, "95ef9d3726fa997c"),
    (0x021EEEC8, "9f615ee3d6048b71"),
    (0x021EEEF4, "ac83a4030371c0b6"),
    (0x021EEF20, "8df211554e37a385"),
    (0x021EEF98, "afce076de20a382c"),
    (0x021EEFE0, "0e3f1e04aadc77ef"),
    (0x021EF1CC, "d684df56b81c8d9e"),
    (0x021EF204, "4bab633af2c3de5a"),
    (0x021EF280, "657339cb2d9f6888"),
    (0x021EF3A4, "8f2e9de9f49fcb1f"),
    (0x021EF460, "06484fbab515e5d7"),
    (0x021EF508, "ed1de43fc092bb58"),
    (0x021EF594, "7ccd35f8c305f2fd"),
    (0x021EF5E8, "c7d24720325e9e50"),
    (0x021EF664, "c3b4146f4b38fdc1"),
    (0x021EF694, "26598a94756d6cf0"),
    (0x021EF6A8, "6e5b7e7e4a9de78a"),
    (0x021EF6B0, "44a248a4f04df25f"),
    (0x021EF6E8, "4999bb6a4dbd96f7"),
    (0x021EF6F0, "26f13036c32ac43b"),
];
// svflow-recorded-end

/// A pointer returned as an address -> offset into the `arg buf` at `base` (0 stays 0).
fn ofs(addr: u32, base: u32) -> u64 {
    if addr == 0 {
        0
    } else {
        addr.wrapping_sub(base) as u64
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    use super::printer::{self as p, Msg, TagOut};
    let Case {
        vals,
        bufs,
        globals,
        regions,
        env,
        ..
    } = c;
    // the message string of the printer functions is the `arg buf 0x100` at index `i`
    let msg = |i: usize| Msg {
        base: vals[i],
        bytes: &bufs[i],
    };
    match addr {
        // adapter proper
        0x021E_EA30 => {
            // global 0: the flag word and the four table slots at 0x021F6384
            super::table_reset(&mut globals[0], vals[0]);
            0
        }
        0x021E_EA54 => {
            super::nop();
            0
        }
        0x021E_EA58 => {
            let r = super::open_slot(
                env,
                &mut globals[0],
                &mut bufs[4],
                vals[4],
                vals[0],
                vals[1],
                vals[2],
                vals[3],
            );
            ofs(r, vals[4])
        }
        0x021E_EAD0 => {
            super::obj_reset(&mut bufs[0]);
            0
        }
        0x021E_EAE0 => {
            super::heap_free(env, vals[0]);
            0
        }
        0x021E_EAE8 => {
            super::comm_reset_if_flag(env, &globals[0]);
            0
        }
        0x021E_EAFC => {
            super::nop_eafc();
            0
        }
        0x021E_EB00 => {
            let (obj, src) = bufs.split_at_mut(1);
            super::queue_message(&mut obj[0], vals[1] as u16, &src[1], vals[3]);
            0
        }
        0x021E_EB2C => super::poll(env, &mut bufs[0], vals[3]) as u64,
        0x021E_EB64 => {
            let out = if vals[1] == 0 {
                None
            } else {
                Some(&mut regions[0][..])
            };
            super::reply_block(&bufs[0], out) as u64
        }
        0x021E_EB70 => {
            super::obj_close(env, &mut bufs[0]);
            0
        }
        0x021E_EB84 => super::send_step(env, &bufs[0], vals[3]) as u64,
        0x021E_EBDC => {
            super::block_reset(&mut bufs[0]);
            0
        }
        0x021E_EBA4 => super::recv_step(env, &mut bufs[0]) as u64,
        0x021E_EBE8 => super::first_byte(env, vals[0], &bufs[0]) as u64,
        0x021E_EC20 => {
            let (obj, out) = bufs.split_at_mut(1);
            super::block_data(env, vals[0], &obj[0], &mut out[0]) as u64
        }
        0x021E_EC4C => super::post_reply(env, &mut bufs[0], vals[1], vals[2]) as u64,
        // message printer: ctx = global 0
        0x021E_EC78 => {
            let w0 = u32::from_le_bytes(bufs[0][0..4].try_into().unwrap());
            p::init(env, &mut globals[0], vals[0], w0, vals[1], vals[2], vals[3]);
            0
        }
        0x021E_ECD8 => {
            p::free(env, &globals[0]);
            0
        }
        0x021E_ED00 => {
            p::word_visible_mon(env, &globals[0], vals[0], vals[1]);
            0
        }
        0x021E_ED24 => {
            p::word_pokemon(env, &globals[0], vals[0], vals[1]);
            0
        }
        0x021E_ED48 => {
            p::word_visible_mon2(env, &globals[0], vals[0], vals[1]);
            0
        }
        0x021E_ED6C => {
            p::word_from_value(env, &globals[0], vals[0], vals[1], vals[2]);
            0
        }
        0x021E_ED90 => {
            p::word_by_kind(env, &globals[0], vals[0], vals[1], vals[2]);
            0
        }
        0x021E_EE24 => {
            p::msg_printf(
                env,
                &mut globals[0],
                vals[0],
                vals[1],
                vals[2],
                &vals[3..11],
                &msg(11),
            );
            0
        }
        0x021E_EE80 => {
            p::msg_remap(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_EEC8 => {
            p::msg_show(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_EEF4 => {
            p::msg_dispatch(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_EF20 => {
            let (outs, string) = bufs.split_at_mut(4);
            let (g, rest) = outs[1..].split_at_mut(1);
            let (t, i) = rest.split_at_mut(1);
            let m = Msg {
                base: vals[4],
                bytes: &string[0],
            };
            let r = p::tag_next(
                env,
                &m,
                vals[0],
                &mut TagOut {
                    group: &mut g[0],
                    ty: &mut t[0],
                    idx: &mut i[0],
                },
            );
            ofs(r, vals[4])
        }
        0x021E_EF98 => p::count_tags(env, vals[0], &msg(1)) as u64,
        0x021E_EFE0 => {
            p::expand_tags(env, &globals[0], vals[0], &bufs[1], vals[2], &msg(3));
            0
        }
        0x021E_F1CC => {
            p::msg_by_tags(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F204 => {
            p::msg_variant(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F280 => {
            p::msg_two_mons(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F3A4 => {
            p::msg_count_b(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F460 => {
            p::msg_count_a(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F508 => {
            p::msg_mon(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F594 => {
            p::msg_by_side(env, &mut globals[0], vals[0], &bufs[2], &msg(3));
            0
        }
        0x021E_F5E8 => {
            p::msg_mon_side(env, &globals[0], vals[0], vals[1], vals[2]);
            0
        }
        0x021E_F664 => {
            p::msg_choice(env, &globals[0], vals[0], vals[1], vals[2]);
            0
        }
        0x021E_F694 => {
            p::msg_set_1c(env, &globals[0], vals[0], vals[1]);
            0
        }
        0x021E_F6A8 => {
            p::msg_flagged_thunk(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F6B0 => {
            p::msg_flagged(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F6E8 => {
            p::msg_bank28_thunk(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        0x021E_F6F0 => {
            p::msg_bank28(env, &mut globals[0], vals[0], vals[1], &bufs[2], &msg(3));
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py adapter`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "adapter",
        include_str!("../../../../../tools/rom-diff/svflow_specs/adapter.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
