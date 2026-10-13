//! `battle_net.s` (0x021D3F04-0x021D4D6C): the client's replay cursor helpers (a 0x10-byte object
//! at `c + 0x18`; written exactly from the disassembly, they are bit / halfword stores) and the
//! comm layer (global work pointer 0x021F3B88), which a local battle never creates: models of the
//! "no comm work" paths only. Out of scope beyond that (D9).

use super::{val, Bufs, Env};
use crate::server_flow::env::{Arg, Lent};

fn gap(addr: u32) -> ! {
    panic!("B16 gap: battle_net {addr:#010x} is only modelled for local battles")
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    // cursor helpers: argument 0 = the cursor
    if (0x021D_3F04..0x021D_3FDC).contains(&addr) {
        let v1 = val(args, 1);
        return Some(Bufs::new(args).with(env, lent, 0, 0x10, |_, k| {
            let h = |k: &[u8], o: usize| u16::from_le_bytes([k[o], k[o + 1]]) as u32;
            let sh = |k: &mut [u8], o: usize, v: u32| {
                k[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes())
            };
            match addr {
                0x021D_3F04 => {
                    k[2] &= !(8 | 2 | 4 | 1 | 0x10 | 0x20);
                    k[1] = 0;
                    k[0] = 0;
                    for o in [6, 8, 0xA, 0xC, 4] {
                        sh(k, o, 0);
                    }
                    0
                }
                0x021D_3F48 => {
                    sh(k, 0xA, v1);
                    0
                }
                0x021D_3F4C => (k[2] as u32 >> 2) & 1,
                0x021D_3F54 => {
                    k[2] |= 8;
                    0
                }
                0x021D_3F60 => k[1] as u32,
                0x021D_3F64 => {
                    sh(k, 8, v1);
                    k[2] |= 1;
                    sh(k, 0xC, 0);
                    k[2] &= !(2 | 4);
                    0
                }
                0x021D_3F88 => {
                    k[0] = 0;
                    k[2] &= !1;
                    0
                }
                0x021D_3F98 => u32::from(h(k, 0xC) == h(k, 8)),
                0x021D_3FA8 => h(k, 8),
                0x021D_3FAC => {
                    if (k[2] >> 1) & 1 != 0 {
                        0
                    } else {
                        k[2] |= 0x10;
                        1
                    }
                }
                0x021D_3FC4 => {
                    k[2] &= !0x10;
                    0
                }
                0x021D_3FD0 => {
                    k[2] |= 0x20;
                    0
                }
                _ => gap(addr),
            }
        }));
    }
    Some(match addr {
        // 021d3fdc(c, cursor): the replay cursor driver; it only acts on a replaying client
        // (mode `c[0x1A7] == 2`)
        0x021D_3FDC => {
            let c = super::client::client_addr_of(env, lent, args);
            if env.read_vec(lent, c + 0x1A7, 1)[0] == 2 {
                gap(addr);
            }
            0
        }
        // 021d4150(comm, ..): creates the comm work 0x021F3B88 only for a comm handle; local: 0
        0x021D_4150 => {
            if val(args, 0) != 0 {
                gap(addr);
            }
            0
        }
        // 021d4230(): no comm work -> 1 (ready)
        0x021D_4230 => 1,
        // 021d4254(): frees the comm work if any
        0x021D_4254 => 0,
        // 021d4270(comm): only called with a comm handle
        0x021D_4270 => gap(addr),
        _ => return None,
    })
}
