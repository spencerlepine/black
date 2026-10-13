//! `btl_adapter.c` (0x021EEA30-0x021EEC76): the server <-> client channel objects (D7). Adapters
//! to the ports; the slot table is the global block at 0x021F6384 (flag word + 4 pointers).

use super::{val, Bufs, Env};
use crate::server_flow::adapter as ad;
use crate::server_flow::env::{Arg, Lent};

/// The flag word and the four slot pointers.
pub const TABLE: u32 = 0x021F_6384;
/// Size of a channel object.
pub const OBJ_SIZE: u32 = 0xBD0;

/// Runs `f` on the channel object at argument 0.
fn obj<R>(
    env: &mut Env,
    lent: &mut Lent<'_>,
    args: &[Arg<'_>],
    f: impl FnOnce(&mut Env, &mut [u8], u32) -> R,
) -> R {
    let a = val(args, 0);
    env.with_obj(lent, a, |e, o| f(e, o, a))
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v = |i: usize| val(args, i);
    Some(match addr {
        0x021E_EA30 => {
            let flag = v(0);
            env.with_obj(lent, TABLE, |_, g| ad::table_reset(g, flag));
            0
        }
        0x021E_EA54 | 0x021E_EAFC => 0,
        // 021eea58(owner, idx, transport, heap): the object the allocator returns is pre-allocated
        // here so the port can initialise it (the allocator hands out exactly this block)
        0x021E_EA58 => {
            let (owner, idx, transport, heap) = (v(0), v(1), v(2), v(3));
            let pre = env.objs.borrow_mut().alloc(OBJ_SIZE, "adapter channel");
            env.pending_alloc = Some((pre, OBJ_SIZE));
            let mut g = env.objs.borrow_mut().take(TABLE);
            let r = env.with_obj(lent, pre, |e, arena| {
                ad::open_slot(e, &mut g, arena, pre, owner, idx, transport, heap)
            });
            env.objs.borrow_mut().put(TABLE, g);
            if env.pending_alloc.take().is_some() {
                env.objs.borrow_mut().free(pre);
            }
            r
        }
        0x021E_EAD0 => obj(env, lent, args, |_, o, _| {
            ad::obj_reset(o);
            0
        }),
        0x021E_EAE0 => {
            ad::heap_free(env, v(0));
            0
        }
        0x021E_EAE8 => {
            let g = env.read_vec(lent, TABLE, 0x14);
            ad::comm_reset_if_flag(env, &g);
            0
        }
        0x021E_EB00 => {
            let (code, size) = (v(1), v(3));
            let data = if size == 0 {
                Vec::new()
            } else {
                Bufs::new(args).get(env, lent, 2, size as usize)
            };
            // the command stream (message 8: the server's command frames) for the test's checks
            if code == 8 && env.cmd_frames.last() != Some(&data) {
                env.cmd_frames.push(data.clone());
            }
            obj(env, lent, args, |_, o, _| {
                ad::queue_message(o, code as u16, &data, size)
            });
            0
        }
        0x021E_EB2C => {
            let r3 = v(3);
            obj(env, lent, args, |e, o, _| ad::poll(e, o, r3))
        }
        0x021E_EB64 => {
            let want = !matches!(args.get(1), Some(Arg::V(0)) | None);
            let mut out = [0u8; 4];
            let r = obj(env, lent, args, |_, o, _| {
                ad::reply_block(o, want.then_some(&mut out[..]))
            });
            if want {
                Bufs::new(args).put(env, lent, 1, &out);
            }
            r
        }
        0x021E_EB70 => obj(env, lent, args, |e, o, _| {
            ad::obj_close(e, o);
            0
        }),
        0x021E_EB84 => {
            let r3 = v(3);
            obj(env, lent, args, |e, o, _| ad::send_step(e, o, r3))
        }
        0x021E_EBDC => obj(env, lent, args, |_, o, _| {
            ad::block_reset(o);
            0
        }),
        0x021E_EBA4 => obj(env, lent, args, |e, o, _| ad::recv_step(e, o)),
        0x021E_EBE8 => obj(env, lent, args, |e, o, a| ad::first_byte(e, a, o)),
        0x021E_EC20 => {
            let mut out = [0u8; 4];
            let r = obj(env, lent, args, |e, o, a| ad::block_data(e, a, o, &mut out));
            Bufs::new(args).put(env, lent, 1, &out);
            r
        }
        0x021E_EC4C => {
            let (p, n) = (v(1), v(2));
            obj(env, lent, args, |e, o, _| ad::post_reply(e, o, p, n))
        }
        _ => return None,
    })
}
