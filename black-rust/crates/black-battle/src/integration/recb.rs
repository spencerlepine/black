//! The battle record builder (`btl_adapter.c` 0x021EE8B4-0x021EEA2F, not ported: B16 gap): the
//! 0x40-byte object at `S + 0x68` that packs the clients' replies into one message. Host model
//! written from the disassembly (UNVERIFIED; no recorded hashes).
//!
//! Layout: `[0]` used length of the data (from data + 0; the first two data bytes are the
//! header), `[1]` mask of the clients added, `[2]` number of entries, `[3]` flags (bits 0-5 a
//! kind, bit 6 the mode, bit 7 overflow), `[4..0x40]` the data: header `(code, flags)` then per
//! client `(client << 5 | words)` and the words.

use super::{val, Bufs, Env};
use crate::server_flow::env::{Arg, Lent};

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let b = val(args, 0);
    let v = |i: usize| val(args, i);
    Some(match addr {
        // 021ee8b4(b, mode): clear; mode bit -> flags bit 6
        0x021E_E8B4 => {
            let mut o = [0u8; 0x40];
            o[3] = ((v(1) & 1) << 6) as u8;
            env.write_bytes(lent, b, &o);
            0
        }
        // 021ee8d8(b, &len, tag): header (4, 0x20 | tag << 7), len = 2; returns the data
        0x021E_E8D8 => {
            let tag = v(2);
            env.write_bytes(lent, b + 4, &[4, 0x20 | ((tag & 1) << 7) as u8]);
            Bufs::new(args).put(env, lent, 1, &2u32.to_le_bytes());
            b + 4
        }
        // 021ee8f4(b, client, entries, n): appends the client's n words once
        0x021E_E8F4 => {
            let (client, src, n) = (v(1), v(2), v(3));
            let mut o = env.read_vec(lent, b, 0x40);
            if o[0] == 0 {
                o[3] = (o[3] & !0x3F) | 1;
                o[0] = 2;
            }
            let mask = (1u32 << client) as u8;
            if o[1] & mask == 0 {
                let size = 4 * n as usize;
                let len = o[0] as usize;
                let newlen = len + size + 1;
                if newlen > 0x3C {
                    o[3] |= 0x80;
                } else {
                    o[1] |= mask;
                    o[2] = o[2].wrapping_add(1);
                    o[4 + len] = ((client << 5) | n) as u8;
                    if size > 0 {
                        let d = env.read_vec(lent, src, size);
                        o[4 + len + 1..4 + len + 1 + size].copy_from_slice(&d);
                    }
                    o[0] = newlen as u8;
                }
            }
            env.write_bytes(lent, b, &o);
            0
        }
        // 021ee96c(b, code, &len): closes the message; 0 on overflow, else the data
        0x021E_E96C => {
            let mut o = env.read_vec(lent, b, 0x40);
            let f = o[3];
            if f & 0x80 != 0 {
                return Some(0);
            }
            o[4] = v(1) as u8;
            o[5] = (o[2] & 0xF) | ((f & 7) << 4) | (((f >> 6) & 1) << 7);
            env.write_bytes(lent, b, &o);
            Bufs::new(args).put(env, lent, 2, &(o[0] as u32).to_le_bytes());
            b + 4
        }
        // 021ee9a4(b, &len): the empty message (0, 0x30)
        0x021E_E9A4 => {
            env.write_bytes(lent, b + 4, &[0, 0x30]);
            Bufs::new(args).put(env, lent, 1, &2u32.to_le_bytes());
            b + 4
        }
        // 021ee9b4(b, data, size): loads a received message for reading
        0x021E_E9B4 => {
            let (src, size) = (v(1), v(2) as usize);
            let mut o = env.read_vec(lent, b, 0x40);
            let d = env.read_vec(lent, src, size);
            o[4..4 + size].copy_from_slice(&d);
            o[0] = size as u8;
            o[1] = 0;
            o[2] = 0;
            o[3] = 0;
            env.write_bytes(lent, b, &o);
            0
        }
        // 021ee9e4(b, &pos, &client, &count, entries): the next client entry; 0 at the end
        0x021E_E9E4 => {
            let o = env.read_vec(lent, b, 0x40);
            let mut bufs = Bufs::new(args);
            let mut pos =
                u32::from_le_bytes(bufs.get(env, lent, 1, 4).try_into().unwrap()) as usize;
            if pos == 0 {
                pos = 2;
            }
            if pos >= o[0] as usize {
                bufs.put(env, lent, 1, &(pos as u32).to_le_bytes());
                return Some(0);
            }
            let e = o[4 + pos];
            let count = e & 0x1F;
            bufs.put(env, lent, 2, &[(e >> 5) & 7]);
            bufs.put(env, lent, 3, &[count]);
            pos += 1;
            let size = 4 * count as usize;
            if size > 0 {
                bufs.put(env, lent, 4, &o[4 + pos..4 + pos + size]);
            }
            pos += size;
            bufs.put(env, lent, 1, &(pos as u32).to_le_bytes());
            1
        }
        _ => return None,
    })
}
