//! `btl_server.c` proper (`server_side`, 0x021BAA94-0x021BB968): adapters to the ports. The
//! server object S (0xCC4 bytes) is held by the port that runs on it; slot functions get the
//! slot's bytes (`S + 0x14 + 12 i`) to the end of S.

use super::{val, Bufs, Env};
use crate::server_flow::env::{Arg, Lent};
use crate::server_flow::server_side as ss;
use crate::server_flow::server_side::StateWord;

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v: Vec<u32> = (0..6)
        .map(|i| match args.get(i) {
            Some(Arg::V(x)) | Some(Arg::PAt(x)) => *x,
            _ => 0,
        })
        .collect();
    let a = v[0];
    macro_rules! s {
        (|$e:tt, $s:ident| $body:expr) => {
            env.with_obj(lent, a, |$e, $s| $body)
        };
    }
    macro_rules! state {
        ($f:path) => {{
            // a state function called through `env.call(f, (S, S + 4))`: the state word is in S
            s!(|e, s| {
                let mut st = StateWord::InS;
                $f(e, a, s, &mut st)
            })
        }};
    }
    Some(match addr {
        0x021B_B4E0 => s!(|e, s| {
            ss::load_record_stream(e, a, s, v[1], v[2]);
            0
        }),
        0x021B_B6A8 => s!(|e, s| {
            ss::collect_slot_records(e, a, s);
            0
        }),
        0x021B_B828 => s!(|_, s| {
            ss::slot_clear(s);
            0
        }),
        0x021B_B830 => s!(|_, s| ss::slot_used(s) as u32),
        0x021B_B840 => s!(|_, s| {
            ss::slot_set(s, v[1] as u8, v[2], v[3] as u8);
            0
        }),
        0x021B_B848 => s!(|e, s| {
            ss::slot_set_data(e, s, v[1], v[2] as u8);
            0
        }),
        // slot_checked / slot_unchecked return the slot's offset in S: the original's pointer
        0x021B_B85C => s!(|_, s| ss::slot_checked(s, v[1]).map_or(0, |o| a + o as u32)),
        0x021B_B878 => s!(|_, s| ss::slot_unchecked(s, v[1] as i32).map_or(0, |o| a + o as u32)),
        0x021B_B890 => s!(|_, s| ss::slot_used_idx(s, v[1]) as u32),
        0x021B_B8A8 => s!(|_, s| ss::byte_0x30(s, v[1] as i32) as u32),
        0x021B_B8B0 => s!(|_, s| ss::word_at(s, v[1] as i32, v[2] as i32)),
        0x021B_B964 => s!(|_, s| ss::ptr_0x44(s)),
        0x021B_B920 => s!(|_, s| {
            ss::list_clear(s);
            0
        }),
        0x021B_B930 => s!(|_, s| {
            ss::list_add(s, v[1]);
            0
        }),
        0x021B_ABDC => {
            let mut out = Bufs::new(args).get(env, lent, 1, 8);
            s!(|_, s| ss::copy_pair(s, a, &mut out));
            Bufs::new(args).put(env, lent, 1, &out);
            0
        }
        0x021B_ABEC => s!(|_, s| {
            ss::set_state(s, v[1]);
            0
        }),
        0x021B_AE5C => ss::always_true(),
        0x021B_B56C => 0,
        0x021B_B724 => s!(|e, s| {
            ss::broadcast(e, s, v[1], v[2], v[3]);
            0
        }),
        0x021B_B718 => s!(|e, s| {
            ss::broadcast_code(e, s, v[1]);
            0
        }),
        0x021B_B768 => s!(|e, s| {
            ss::send_one(e, s, v[1], v[2], v[3], v[4]);
            0
        }),
        0x021B_B798 => s!(|e, s| ss::all_ready(e, s) as u32),
        0x021B_B7D0 => s!(|e, s| {
            ss::each_eeb70(e, s);
            0
        }),
        0x021B_B7FC => s!(|e, s| {
            ss::each_eead0(e, s);
            0
        }),
        0x021B_AA94 => s!(|e, s| {
            ss::setup_slot_handle(e, s, v[1], v[2], v[3]);
            0
        }),
        0x021B_AACC => s!(|e, s| {
            ss::setup_slot_created(e, s, v[2], v[3], v[4]);
            0
        }),
        0x021B_AB10 => s!(|e, s| {
            ss::setup_slot_empty(e, s, v[1], v[2]);
            0
        }),
        0x021B_AB40 => s!(|e, s| {
            ss::teardown(e, s);
            0
        }),
        0x021B_AB5C => s!(|e, s| {
            ss::destroy(e, a, s);
            0
        }),
        0x021B_ABF4 => s!(|_, s| {
            ss::set_state_acad(s);
            0
        }),
        0x021B_B914 => s!(|e, s| {
            ss::call_9234(e, s);
            0
        }),
        0x021B_B8BC => s!(|e, s| {
            ss::pokemon_set_73(e, s, v[1]);
            0
        }),
        0x021B_B8DC => s!(|e, s| {
            ss::pokemon_set_5(e, s, v[1]);
            0
        }),
        0x021B_B5FC => s!(|e, s| ss::builder_load(e, a, s, v[1])),
        0x021B_B650 => s!(|e, s| ss::builder_finish(e, a, s, v[1], v[2], ss::FinishOut::Ptr(v[3]))),
        0x021B_B628 => s!(|e, s| ss::builder_flush(e, a, s, v[1], v[2]) as u32),
        0x021B_AC04 => state!(ss::state_ac04),
        0x021B_ACAC => state!(ss::state_acac),
        0x021B_AE60 => state!(ss::state_ae60),
        0x021B_AEF8 => state!(ss::state_aef8),
        0x021B_B0E0 => state!(ss::state_b0e0),
        0x021B_B208 => state!(ss::state_b208),
        0x021B_B368 => state!(ss::state_b368),
        0x021B_B320 => s!(|e, s| {
            let mut st = StateWord::InS;
            ss::state_b320(e, s, &mut st)
        }),
        0x021B_B3B0 => state!(ss::state_b3b0),
        0x021B_B3F8 => state!(ss::state_b3f8),
        0x021B_B43C => state!(ss::state_b43c),
        0x021B_B484 => state!(ss::state_b484),
        0x021B_AB9C => s!(|e, s| ss::run_state(e, a, s)),
        0x021B_B570 => s!(|e, s| ss::compare_header(e, a, s, v[1], v[2], v[3]) as u32),
        0x021B_B040 => s!(|e, s| ss::party_ready(e, s) as u32),
        0x021B_B0A0 => s!(|e, s| ss::slot1_value(e, a, s)),
        0x021B_B280 => s!(|e, s| ss::pick_next_state(e, s)),
        _ => {
            let _ = val;
            return None;
        }
    })
}
