//! A minimal ARMv5T Thumb interpreter for the unported overlay-95 routines (test host model).
//!
//! Overlay 95 (linked at 0x06898020) is not ported apart from its effect handlers; the flow
//! reaches ~120 of its routines through `bx pc` veneers (mon lists, the position table at
//! `work + 0x1AB8`, the move-history ring at `work + 0x1C`, move / item / ability property
//! tables). Instead of guessing their behaviour, the integration runs their original Thumb code
//! here, over the object table: code and read-only data come from the ROM images, everything else
//! from [`super::objects::Objects`] (objects a port holds must have been lent, as for any callee),
//! the stack is a scratch object. A branch out of overlay 95 (`bl`/`blx`/veneer) is a call
//! through [`Env::call`] with r0-r3 and four stack words; the answer goes to r0. This is a test
//! host only (D1: nothing of it ships).

use super::Env;
use crate::server_flow::env::{Arg, FlowEnv};

pub const OVY95_END: u32 = 0x0689_E7E0;
/// The overlay-95 BSS (side-effect layers at its start, 0x1C0 bytes).
pub const OVY95_BSS: u32 = OVY95_END;
pub const OVY95_BSS_SIZE: u32 = 0x2000;
/// The interpreter stack (an object).
pub const STACK_BASE: u32 = 0x02F8_0000;
pub const STACK_SIZE: u32 = 0x4000;
/// Return-address sentinel.
const RET: u32 = 0x02FF_FFF0;

struct Cpu {
    /// The module the routine runs in (0 overlay 95, 1 ARM9 main, 2 overlay 93, 3 overlay 96):
    /// code there is interpreted, anything else is called out.
    module: u8,
    r: [u32; 16],
    n: bool,
    z: bool,
    c: bool,
    v: bool,
}

/// Whether `a` is code of the module the routine runs in.
fn local(c: &Cpu, a: u32) -> bool {
    match c.module {
        0 => in_ov95(a),
        1 => (super::rom::MAIN_BASE..0x020A_A7CC).contains(&a),
        3 => in_ov96(a),
        _ => (super::rom::OVY93_BASE..0x021F_3B20).contains(&a),
    }
}

/// Overlay-96 code (the AI; its data starts at the VM init record 0x0218A574).
pub fn in_ov96(a: u32) -> bool {
    (super::rom::OVY96_BASE..0x0218_A574).contains(&a)
}

fn in_ov95(a: u32) -> bool {
    (super::rom::OVY95_BASE..OVY95_END).contains(&a)
}

impl Env {
    fn m_read(&mut self, a: u32, n: usize) -> u32 {
        let mut b = [0u8; 4];
        if ((0x0200_0000..0x0240_0000).contains(&a) || in_ov95(a)) && self.rom_readable(a) {
            for (i, x) in b.iter_mut().enumerate().take(n) {
                *x = self.rom.r8(a + i as u32) as u8;
            }
            return u32::from_le_bytes(b);
        }
        self.objs.borrow_mut().read(a, &mut b[..n]);
        u32::from_le_bytes(b)
    }

    /// Code and read-only data the interpreter reads from the ROM images: overlay 95 (file part),
    /// overlay 93 code and rodata up to its data, ARM9 main code / rodata.
    fn rom_readable(&self, a: u32) -> bool {
        in_ov95(a)
            || (super::rom::OVY93_BASE..0x021F_3B20).contains(&a)
            || (super::rom::OVY96_BASE..0x0218_A7C0).contains(&a)
            || (super::rom::MAIN_BASE..0x020A_A7CC).contains(&a)
    }

    fn m_write(&mut self, a: u32, v: u32, n: usize) {
        assert!(
            !self.rom_readable(a),
            "B16: overlay-95 code writes ROM at {a:#x}"
        );
        self.objs.borrow_mut().write(a, &v.to_le_bytes()[..n]);
    }

    /// Runs the Thumb routine at `addr` with `args` (r0-r3, then stack words); returns r0.
    pub fn run_thumb(&mut self, addr: u32, args: &[u32]) -> u32 {
        if self.objs.borrow().find_block(STACK_BASE, 1).is_none() {
            self.objs
                .borrow_mut()
                .map(STACK_BASE, vec![0; STACK_SIZE as usize], "thumb stack");
        }
        let mut cpu = Cpu {
            module: if in_ov95(addr) {
                0
            } else if (super::rom::MAIN_BASE..0x020A_A7CC).contains(&addr) {
                1
            } else if in_ov96(addr) {
                3
            } else {
                2
            },
            r: [0; 16],
            n: false,
            z: false,
            c: false,
            v: false,
        };
        let depth_sp = self.thumb_sp.unwrap_or(STACK_BASE + STACK_SIZE - 0x40);
        cpu.r[13] = depth_sp;
        for (i, &a) in args.iter().enumerate().take(4) {
            cpu.r[i] = a;
        }
        if args.len() > 4 {
            let extra = &args[4..];
            cpu.r[13] -= 4 * extra.len() as u32;
            for (k, &w) in extra.iter().enumerate() {
                let sp = cpu.r[13];
                self.m_write(sp + 4 * k as u32, w, 4);
            }
        }
        cpu.r[14] = RET | 1;
        cpu.r[15] = addr & !1;
        let mut steps = 0u64;
        loop {
            let pc = cpu.r[15];
            if pc == RET {
                break;
            }
            steps += 1;
            assert!(
                steps < 5_000_000,
                "B16: overlay-95 routine {addr:#x} does not return"
            );
            if !local(&cpu, pc) {
                // a tail call out of the overlay (b / bx to a ported or host routine)
                let saved = self.thumb_sp.replace(cpu.r[13]);
                let r0 = self.call_out(pc, &cpu);
                self.thumb_sp = saved;
                cpu.r[0] = r0;
                cpu.r[15] = cpu.r[14] & !1;
                continue;
            }
            let op = self.m_read(pc, 2);
            cpu.r[15] = pc + 2;
            self.step(&mut cpu, pc, op as u16);
        }
        cpu.r[0]
    }

    /// A call out of the overlay: r0-r3 and four stack words as values.
    fn call_out(&mut self, target: u32, cpu: &Cpu) -> u32 {
        let target = self.resolve_veneer(target);
        if in_ov95(target) {
            let sp = cpu.r[13];
            let mut a = vec![cpu.r[0], cpu.r[1], cpu.r[2], cpu.r[3]];
            for k in 0..4 {
                a.push(self.m_read(sp + 4 * k, 4));
            }
            return self.run_thumb(target, &a);
        }
        let sp = cpu.r[13];
        let mut words = vec![cpu.r[0], cpu.r[1], cpu.r[2], cpu.r[3]];
        for k in 0..4 {
            words.push(self.m_read(sp + 4 * k, 4));
        }
        let mut args: Vec<Arg<'_>> = words.into_iter().map(Arg::V).collect();
        self.call(target, &mut args)
    }

    /// The target of an ARM veneer (`ldr ip, [pc]; bx ip; .word T` or `ldr pc, [pc, #-4]; .word
    /// T`) or a Thumb `bx pc` veneer; else `a`.
    pub fn resolve_veneer(&mut self, a: u32) -> u32 {
        let a = a & !1;
        if !self.rom_readable(a) {
            return a;
        }
        let w0 = self.rom.r32(a);
        if w0 == 0xE59F_C000 && self.rom.r32(a + 4) == 0xE12F_FF1C {
            return self.rom.r32(a + 8) & !1;
        }
        if w0 == 0xE51F_F004 {
            return self.rom.r32(a + 4) & !1;
        }
        if w0 == 0x46C0_4778
            && self.rom.r32(a + 4) == 0xE59F_C000
            && self.rom.r32(a + 8) == 0xE12F_FF1C
        {
            return self.rom.r32(a + 12) & !1;
        }
        a
    }

    fn step(&mut self, c: &mut Cpu, pc: u32, op: u16) {
        let op = op as u32;
        let rd = (op & 7) as usize;
        let rs = ((op >> 3) & 7) as usize;
        let rn = ((op >> 6) & 7) as usize;
        let imm5 = (op >> 6) & 0x1F;
        macro_rules! nz {
            ($v:expr) => {{
                let v: u32 = $v;
                c.n = v >> 31 != 0;
                c.z = v == 0;
                v
            }};
        }
        fn add_flags(c: &mut Cpu, a: u32, b: u32, carry: u32) -> u32 {
            let r = (a as u64) + (b as u64) + carry as u64;
            let res = r as u32;
            c.c = r >> 32 != 0;
            c.v = ((a ^ res) & (b ^ res)) >> 31 != 0;
            c.n = res >> 31 != 0;
            c.z = res == 0;
            res
        }
        fn sub_flags(c: &mut Cpu, a: u32, b: u32, carry: u32) -> u32 {
            add_flags(c, a, !b, carry)
        }
        match op >> 11 {
            0 => {
                // LSL imm
                let v = c.r[rs];
                let r = if imm5 == 0 {
                    v
                } else {
                    c.c = (v >> (32 - imm5)) & 1 != 0;
                    v << imm5
                };
                c.r[rd] = nz!(r);
            }
            1 => {
                let v = c.r[rs];
                let sh = if imm5 == 0 { 32 } else { imm5 };
                c.c = if sh == 32 {
                    v >> 31 != 0
                } else {
                    (v >> (sh - 1)) & 1 != 0
                };
                let r = if sh == 32 { 0 } else { v >> sh };
                c.r[rd] = nz!(r);
            }
            2 => {
                let v = c.r[rs];
                let sh = if imm5 == 0 { 32 } else { imm5 };
                c.c = if sh == 32 {
                    v >> 31 != 0
                } else {
                    (v >> (sh - 1)) & 1 != 0
                };
                let r = if sh == 32 {
                    ((v as i32) >> 31) as u32
                } else {
                    ((v as i32) >> sh) as u32
                };
                c.r[rd] = nz!(r);
            }
            3 => {
                let a = c.r[rs];
                let b = if op & 0x400 != 0 { rn as u32 } else { c.r[rn] };
                c.r[rd] = if op & 0x200 != 0 {
                    sub_flags(c, a, b, 1)
                } else {
                    add_flags(c, a, b, 0)
                };
            }
            4..=7 => {
                let r = ((op >> 8) & 7) as usize;
                let imm = op & 0xFF;
                match (op >> 11) & 3 {
                    0 => c.r[r] = nz!(imm),
                    1 => {
                        sub_flags(c, c.r[r], imm, 1);
                    }
                    2 => c.r[r] = add_flags(c, c.r[r], imm, 0),
                    _ => c.r[r] = sub_flags(c, c.r[r], imm, 1),
                }
            }
            8 => {
                if op & 0x400 == 0 {
                    // ALU
                    let a = c.r[rd];
                    let b = c.r[rs];
                    match (op >> 6) & 0xF {
                        0 => c.r[rd] = nz!(a & b),
                        1 => c.r[rd] = nz!(a ^ b),
                        2 => {
                            let sh = b & 0xFF;
                            let r = if sh == 0 {
                                a
                            } else if sh < 32 {
                                c.c = (a >> (32 - sh)) & 1 != 0;
                                a << sh
                            } else {
                                c.c = sh == 32 && a & 1 != 0;
                                0
                            };
                            c.r[rd] = nz!(r);
                        }
                        3 => {
                            let sh = b & 0xFF;
                            let r = if sh == 0 {
                                a
                            } else if sh < 32 {
                                c.c = (a >> (sh - 1)) & 1 != 0;
                                a >> sh
                            } else {
                                c.c = sh == 32 && a >> 31 != 0;
                                0
                            };
                            c.r[rd] = nz!(r);
                        }
                        4 => {
                            let sh = b & 0xFF;
                            let r = if sh == 0 {
                                a
                            } else if sh < 32 {
                                c.c = (a >> (sh - 1)) & 1 != 0;
                                ((a as i32) >> sh) as u32
                            } else {
                                c.c = a >> 31 != 0;
                                ((a as i32) >> 31) as u32
                            };
                            c.r[rd] = nz!(r);
                        }
                        5 => c.r[rd] = add_flags(c, a, b, c.c as u32),
                        6 => c.r[rd] = sub_flags(c, a, b, c.c as u32),
                        7 => {
                            let sh = b & 0xFF;
                            let r = if sh & 0x1F == 0 {
                                if sh != 0 {
                                    c.c = a >> 31 != 0;
                                }
                                a
                            } else {
                                let s = sh & 0x1F;
                                c.c = (a >> (s - 1)) & 1 != 0;
                                a.rotate_right(s)
                            };
                            c.r[rd] = nz!(r);
                        }
                        8 => {
                            nz!(a & b);
                        }
                        9 => c.r[rd] = sub_flags(c, 0, b, 1),
                        10 => {
                            sub_flags(c, a, b, 1);
                        }
                        11 => {
                            add_flags(c, a, b, 0);
                        }
                        12 => c.r[rd] = nz!(a | b),
                        13 => c.r[rd] = nz!(a.wrapping_mul(b)),
                        14 => c.r[rd] = nz!(a & !b),
                        _ => c.r[rd] = nz!(!b),
                    }
                } else {
                    // hi register operations / BX / BLX
                    let h1 = ((op >> 7) & 1) as usize;
                    let h2 = ((op >> 6) & 1) as usize;
                    let d = rd + 8 * h1;
                    let s = rs + 8 * h2;
                    let sv = if s == 15 { (pc + 4) & !1 } else { c.r[s] };
                    match (op >> 8) & 3 {
                        0 => {
                            let dv = if d == 15 { pc + 4 } else { c.r[d] };
                            let r = dv.wrapping_add(sv);
                            if d == 15 {
                                c.r[15] = r & !1;
                            } else {
                                c.r[d] = r;
                            }
                        }
                        1 => {
                            let dv = if d == 15 { pc + 4 } else { c.r[d] };
                            sub_flags(c, dv, sv, 1);
                        }
                        2 => {
                            if d == 15 {
                                c.r[15] = sv & !1;
                            } else {
                                c.r[d] = sv;
                            }
                        }
                        _ => {
                            let link = h1 != 0;
                            let target = sv;
                            if link {
                                c.r[14] = (pc + 2) | 1;
                            }
                            self.branch(c, target, link);
                        }
                    }
                }
            }
            9 => {
                let r = ((op >> 8) & 7) as usize;
                let a = ((pc + 4) & !3) + 4 * (op & 0xFF);
                c.r[r] = self.m_read(a, 4);
            }
            10 | 11 => {
                let a = c.r[rs].wrapping_add(c.r[rn]);
                match (op >> 9) & 7 {
                    0 => self.m_write(a, c.r[rd], 4),
                    1 => self.m_write(a, c.r[rd], 2),
                    2 => self.m_write(a, c.r[rd], 1),
                    3 => c.r[rd] = self.m_read(a, 1) as u8 as i8 as i32 as u32,
                    4 => c.r[rd] = self.m_read(a, 4),
                    5 => c.r[rd] = self.m_read(a, 2),
                    6 => c.r[rd] = self.m_read(a, 1),
                    _ => c.r[rd] = self.m_read(a, 2) as u16 as i16 as i32 as u32,
                }
            }
            12 => self.m_write(c.r[rs].wrapping_add(imm5 * 4), c.r[rd], 4),
            13 => c.r[rd] = self.m_read(c.r[rs].wrapping_add(imm5 * 4), 4),
            14 => self.m_write(c.r[rs].wrapping_add(imm5), c.r[rd], 1),
            15 => c.r[rd] = self.m_read(c.r[rs].wrapping_add(imm5), 1),
            16 => self.m_write(c.r[rs].wrapping_add(imm5 * 2), c.r[rd], 2),
            17 => c.r[rd] = self.m_read(c.r[rs].wrapping_add(imm5 * 2), 2),
            18 => {
                let r = ((op >> 8) & 7) as usize;
                self.m_write(c.r[13] + 4 * (op & 0xFF), c.r[r], 4);
            }
            19 => {
                let r = ((op >> 8) & 7) as usize;
                c.r[r] = self.m_read(c.r[13] + 4 * (op & 0xFF), 4);
            }
            20 => {
                let r = ((op >> 8) & 7) as usize;
                c.r[r] = ((pc + 4) & !3) + 4 * (op & 0xFF);
            }
            21 => {
                let r = ((op >> 8) & 7) as usize;
                c.r[r] = c.r[13] + 4 * (op & 0xFF);
            }
            22 | 23 => match (op >> 8) & 0xF {
                0 => {
                    let imm = 4 * (op & 0x7F);
                    c.r[13] = if op & 0x80 != 0 {
                        c.r[13] - imm
                    } else {
                        c.r[13] + imm
                    };
                }
                4 | 5 => {
                    // PUSH {rlist, lr?}
                    let mut list: Vec<usize> = (0..8).filter(|i| op & (1 << i) != 0).collect();
                    if op & 0x100 != 0 {
                        list.push(14);
                    }
                    c.r[13] -= 4 * list.len() as u32;
                    let mut a = c.r[13];
                    for i in list {
                        self.m_write(a, c.r[i], 4);
                        a += 4;
                    }
                }
                12 | 13 => {
                    let mut a = c.r[13];
                    for i in 0..8 {
                        if op & (1 << i) != 0 {
                            c.r[i] = self.m_read(a, 4);
                            a += 4;
                        }
                    }
                    if op & 0x100 != 0 {
                        let v = self.m_read(a, 4);
                        a += 4;
                        c.r[13] = a;
                        // ARMv5T: pop {pc} interworks
                        c.r[15] = v & !1;
                        return;
                    }
                    c.r[13] = a;
                }
                _ => panic!("B16: thumb op {op:#06x} at {pc:#x} not supported"),
            },
            24 => {
                let rb = ((op >> 8) & 7) as usize;
                let mut a = c.r[rb];
                for i in 0..8 {
                    if op & (1 << i) != 0 {
                        self.m_write(a, c.r[i], 4);
                        a += 4;
                    }
                }
                c.r[rb] = a;
            }
            25 => {
                let rb = ((op >> 8) & 7) as usize;
                let mut a = c.r[rb];
                for i in 0..8 {
                    if op & (1 << i) != 0 {
                        c.r[i] = self.m_read(a, 4);
                        a += 4;
                    }
                }
                if op & (1 << rb) == 0 {
                    c.r[rb] = a;
                }
            }
            26 | 27 => {
                let cond = (op >> 8) & 0xF;
                if cond == 0xF {
                    panic!("B16: swi in overlay 95 at {pc:#x}");
                }
                let take = match cond {
                    0 => c.z,
                    1 => !c.z,
                    2 => c.c,
                    3 => !c.c,
                    4 => c.n,
                    5 => !c.n,
                    6 => c.v,
                    7 => !c.v,
                    8 => c.c && !c.z,
                    9 => !c.c || c.z,
                    10 => c.n == c.v,
                    11 => c.n != c.v,
                    12 => !c.z && c.n == c.v,
                    13 => c.z || c.n != c.v,
                    _ => panic!("B16: thumb cond {cond} at {pc:#x}"),
                };
                if take {
                    let off = ((op & 0xFF) as u8 as i8 as i32) * 2;
                    c.r[15] = (pc as i32 + 4 + off) as u32;
                }
            }
            28 => {
                let off = (((op & 0x7FF) << 21) as i32 >> 20) as u32;
                c.r[15] = pc.wrapping_add(4).wrapping_add(off);
            }
            30 => {
                // BL / BLX prefix
                let off = (((op & 0x7FF) << 21) as i32 >> 9) as u32;
                c.r[14] = pc.wrapping_add(4).wrapping_add(off);
            }
            31 | 29 => {
                let target = c.r[14].wrapping_add((op & 0x7FF) << 1);
                c.r[14] = (pc + 2) | 1;
                let target = if op >> 11 == 29 {
                    target & !3
                } else {
                    target | 1
                };
                self.branch(c, target, true);
            }
            _ => panic!("B16: thumb op {op:#06x} at {pc:#x} not supported"),
        }
    }

    /// A branch (with link when `link`) to `target`: inside overlay 95 (Thumb) it continues there;
    /// anything else is a call out (a veneer, ARM or another overlay).
    fn branch(&mut self, c: &mut Cpu, target: u32, link: bool) {
        if target & !1 == RET {
            c.r[15] = RET;
            return;
        }
        let t = self.resolve_veneer(target);
        // in overlay 93 only the interpreted glue and the constant handler-table getters run here;
        // a call to anything else there is a call of its port
        let ov93_ok =
            c.module != 2 || !link || super::dispatch::OV93_THUMB.contains(&(t & !1)) || {
                let g = t & !1;
                self.rom.r16(g) >> 8 == 0x21 && self.rom.r16(g + 2) == 0x6001
            };
        if local(c, t) && t == target & !1 && target & 1 != 0 && ov93_ok {
            c.r[15] = t & !1;
            return;
        }
        // 64-bit runtime helpers (r0:r1 results)
        if let Some(r) = long_helper(t & !1, &c.r) {
            c.r[0] = r as u32;
            c.r[1] = (r >> 32) as u32;
            c.r[15] = c.r[14] & !1;
            return;
        }
        let saved = self.thumb_sp.replace(c.r[13]);
        let r0 = self.call_out(t, c);
        self.thumb_sp = saved;
        c.r[0] = r0;
        c.r[15] = c.r[14] & !1;
    }
}

/// The ARM9 64-bit runtime helpers (`_ll_mul` 0x0209C054), computed directly.
fn long_helper(a: u32, r: &[u32; 16]) -> Option<u64> {
    let x = u64::from(r[0]) | (u64::from(r[1]) << 32);
    let y = u64::from(r[2]) | (u64::from(r[3]) << 32);
    match a {
        0x0209_C054 => Some(x.wrapping_mul(y)),
        _ => None,
    }
}
