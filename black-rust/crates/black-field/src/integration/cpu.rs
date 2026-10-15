//! An ARMv5TE interpreter (ARM and Thumb) for the unported original code the F12 field session
//! runs (test host only; D1: nothing of it ships).
//!
//! The interpreter knows nothing about the game: memory, the call hooks (ported functions and host
//! models are entered at their address instead of being run) and the software interrupts are the
//! [`Bus`]'s. Supported: every ARMv5TE data-processing, multiply (incl. the long and the DSP
//! halfword forms), load / store (word, byte, halfword, signed, doubleword, multiple, swap), branch
//! (B / BL / BX / BLX, interworking loads), CLZ, MRS / MSR (flags and the I / F bits only), coprocessor
//! moves (CP15: ignored, reads 0), PLD (no-op) and SWI (to the bus). Saturating Q-arithmetic and mode
//! changes panic (none of the field code reached them).

/// The CPU state.
#[derive(Clone, Debug, Default)]
pub struct Cpu {
    pub r: [u32; 16],
    pub n: bool,
    pub z: bool,
    pub c: bool,
    pub v: bool,
    pub q: bool,
    /// The CPSR I / F bits and mode (kept for MRS / MSR, no effect).
    pub ctl: u32,
    pub thumb: bool,
}

impl Cpu {
    pub fn cpsr(&self) -> u32 {
        (self.n as u32) << 31
            | (self.z as u32) << 30
            | (self.c as u32) << 29
            | (self.v as u32) << 28
            | (self.q as u32) << 27
            | (self.thumb as u32) << 5
            | (self.ctl & 0xDF)
    }

    fn set_cpsr(&mut self, v: u32, mask: u32) {
        if mask & 0xFF00_0000 != 0 {
            self.n = v >> 31 != 0;
            self.z = (v >> 30) & 1 != 0;
            self.c = (v >> 29) & 1 != 0;
            self.v = (v >> 28) & 1 != 0;
            self.q = (v >> 27) & 1 != 0;
        }
        if mask & 0xFF != 0 {
            self.ctl = (self.ctl & !0xDF) | (v & 0xDF);
        }
    }

    fn cond(&self, c: u32) -> bool {
        match c {
            0 => self.z,
            1 => !self.z,
            2 => self.c,
            3 => !self.c,
            4 => self.n,
            5 => !self.n,
            6 => self.v,
            7 => !self.v,
            8 => self.c && !self.z,
            9 => !self.c || self.z,
            10 => self.n == self.v,
            11 => self.n != self.v,
            12 => !self.z && self.n == self.v,
            13 => self.z || self.n != self.v,
            _ => true,
        }
    }

    fn nz(&mut self, v: u32) -> u32 {
        self.n = v >> 31 != 0;
        self.z = v == 0;
        v
    }

    fn add(&mut self, a: u32, b: u32, carry: bool, set: bool) -> u32 {
        let r = a as u64 + b as u64 + carry as u64;
        let res = r as u32;
        if set {
            self.c = r >> 32 != 0;
            self.v = ((a ^ res) & (b ^ res)) >> 31 != 0;
            self.nz(res);
        }
        res
    }

    fn sub(&mut self, a: u32, b: u32, carry: bool, set: bool) -> u32 {
        self.add(a, !b, carry, set)
    }

    /// Branch to `t` with interworking (bit 0 selects Thumb).
    pub fn bx(&mut self, t: u32) {
        self.thumb = t & 1 != 0;
        self.r[15] = if self.thumb { t & !1 } else { t & !3 };
    }
}

/// What the interpreter runs on.
pub trait Bus {
    fn rd(&mut self, a: u32, n: u32) -> u32;
    fn wr(&mut self, a: u32, v: u32, n: u32);
    /// Whether an instruction fetch at `pc` enters a hooked function (port or host model).
    fn hooked(&self, pc: u32) -> bool;
    /// Run the hooked function at `cpu.r[15]` (arguments in r0-r3 and on the stack) and return to
    /// `lr` (results in r0 / r1).
    fn hook(&mut self, cpu: &mut Cpu);
    /// A software interrupt (BIOS call) `n`.
    fn swi(&mut self, cpu: &mut Cpu, n: u32);
    /// The instruction about to run (diagnostics).
    fn at(&mut self, pc: u32, lr: u32, sp: u32);
    /// Abort with `msg` and the bus's diagnostics.
    fn fail(&mut self, msg: &str) -> !;
}

/// Shift `v` by type `t` (0 LSL, 1 LSR, 2 ASR, 3 ROR) and `amt`; `imm` = immediate shift encoding
/// (amount 0 means 32 for LSR / ASR, RRX for ROR). Returns (value, carry out).
fn shift(v: u32, t: u32, amt: u32, imm: bool, c: bool) -> (u32, bool) {
    match t {
        0 => match amt {
            0 => (v, c),
            1..=31 => (v << amt, (v >> (32 - amt)) & 1 != 0),
            32 => (0, v & 1 != 0),
            _ => (0, false),
        },
        1 => {
            let amt = if imm && amt == 0 { 32 } else { amt };
            match amt {
                0 => (v, c),
                1..=31 => (v >> amt, (v >> (amt - 1)) & 1 != 0),
                32 => (0, v >> 31 != 0),
                _ => (0, false),
            }
        }
        2 => {
            let amt = if imm && amt == 0 { 32 } else { amt };
            match amt {
                0 => (v, c),
                1..=31 => (((v as i32) >> amt) as u32, (v >> (amt - 1)) & 1 != 0),
                _ => (((v as i32) >> 31) as u32, v >> 31 != 0),
            }
        }
        _ => {
            if imm && amt == 0 {
                (((c as u32) << 31) | (v >> 1), v & 1 != 0)
            } else if amt == 0 {
                (v, c)
            } else {
                let a = amt & 31;
                if a == 0 {
                    (v, v >> 31 != 0)
                } else {
                    (v.rotate_right(a), (v >> (a - 1)) & 1 != 0)
                }
            }
        }
    }
}

/// Executes from the current `cpu` state until `pc == stop` (the return sentinel) or `max` steps.
pub fn run<B: Bus>(b: &mut B, cpu: &mut Cpu, stop: u32, max: u64) -> u64 {
    let mut steps = 0u64;
    loop {
        let pc = cpu.r[15];
        if pc & !1 == stop & !1 {
            return steps;
        }
        if b.hooked(pc) {
            b.hook(cpu);
            continue;
        }
        steps += 1;
        b.at(pc, cpu.r[14], cpu.r[13]);
        if steps >= max {
            b.fail(&format!(
                "F12: interpreted code does not return (pc {pc:#010x})"
            ));
        }
        if cpu.thumb {
            let op = b.rd(pc, 2);
            cpu.r[15] = pc + 2;
            thumb(b, cpu, pc, op);
        } else {
            let op = b.rd(pc, 4);
            cpu.r[15] = pc + 4;
            if op >> 28 == 0xF {
                arm_uncond(b, cpu, pc, op);
            } else if cpu.cond(op >> 28) {
                arm(b, cpu, pc, op);
            }
        }
    }
}

fn ldr_word<B: Bus>(b: &mut B, a: u32) -> u32 {
    let v = b.rd(a & !3, 4);
    v.rotate_right(8 * (a & 3))
}

fn arm_uncond<B: Bus>(_b: &mut B, cpu: &mut Cpu, pc: u32, op: u32) {
    if op & 0x0E00_0000 == 0x0A00_0000 {
        // BLX imm
        let off = (((op & 0x00FF_FFFF) << 8) as i32 >> 6) as u32 | ((op >> 23) & 2);
        cpu.r[14] = pc + 4;
        let t = pc.wrapping_add(8).wrapping_add(off);
        cpu.thumb = true;
        cpu.r[15] = t;
        return;
    }
    if op & 0x0D70_F000 == 0x0550_F000 {
        return; // PLD
    }
    panic!("F12: ARM op {op:#010x} at {pc:#010x} not supported");
}

fn arm<B: Bus>(b: &mut B, cpu: &mut Cpu, pc: u32, op: u32) {
    let reg = |cpu: &Cpu, i: u32| -> u32 {
        if i == 15 {
            pc + 8
        } else {
            cpu.r[i as usize]
        }
    };
    // BX / BLX reg
    if op & 0x0FFF_FFD0 == 0x012F_FF10 {
        let t = reg(cpu, op & 0xF);
        if op & 0x20 != 0 {
            cpu.r[14] = pc + 4;
        }
        cpu.bx(t);
        return;
    }
    // CLZ
    if op & 0x0FFF_0FF0 == 0x016F_0F10 {
        let v = reg(cpu, op & 0xF);
        cpu.r[((op >> 12) & 0xF) as usize] = v.leading_zeros();
        return;
    }
    match (op >> 25) & 7 {
        0 | 1 => {
            let imm = (op >> 25) & 1 != 0;
            // multiplies, swaps, extra loads / stores live in the register form with bit 7 and 4 set
            if !imm && op & 0x90 == 0x90 {
                if (op >> 5) & 3 == 0 {
                    if op & 0x0FC0_00F0 == 0x0000_0090 {
                        // MUL / MLA
                        let rd = (op >> 16) & 0xF;
                        let mut v = reg(cpu, op & 0xF).wrapping_mul(reg(cpu, (op >> 8) & 0xF));
                        if op & (1 << 21) != 0 {
                            v = v.wrapping_add(reg(cpu, (op >> 12) & 0xF));
                        }
                        cpu.r[rd as usize] = v;
                        if op & (1 << 20) != 0 {
                            cpu.nz(v);
                        }
                        return;
                    }
                    if op & 0x0F80_00F0 == 0x0080_0090 {
                        // UMULL / UMLAL / SMULL / SMLAL
                        let (rlo, rhi) = ((op >> 12) & 0xF, (op >> 16) & 0xF);
                        let (a, bb) = (reg(cpu, op & 0xF), reg(cpu, (op >> 8) & 0xF));
                        let mut v: u64 = if op & (1 << 22) != 0 {
                            ((a as i32 as i64) * (bb as i32 as i64)) as u64
                        } else {
                            (a as u64) * (bb as u64)
                        };
                        if op & (1 << 21) != 0 {
                            v = v.wrapping_add(
                                (cpu.r[rhi as usize] as u64) << 32 | cpu.r[rlo as usize] as u64,
                            );
                        }
                        cpu.r[rlo as usize] = v as u32;
                        cpu.r[rhi as usize] = (v >> 32) as u32;
                        if op & (1 << 20) != 0 {
                            cpu.n = v >> 63 != 0;
                            cpu.z = v == 0;
                        }
                        return;
                    }
                    if op & 0x0FB0_0FF0 == 0x0100_0090 {
                        // SWP / SWPB
                        let a = reg(cpu, (op >> 16) & 0xF);
                        let byte = op & (1 << 22) != 0;
                        let old = if byte { b.rd(a, 1) } else { ldr_word(b, a) };
                        let nv = reg(cpu, op & 0xF);
                        b.wr(if byte { a } else { a & !3 }, nv, if byte { 1 } else { 4 });
                        cpu.r[((op >> 12) & 0xF) as usize] = old;
                        return;
                    }
                    panic!("F12: ARM op {op:#010x} at {pc:#010x} not supported");
                }
                // LDRH / STRH / LDRSB / LDRSH / LDRD / STRD
                let p = op & (1 << 24) != 0;
                let u = op & (1 << 23) != 0;
                let i = op & (1 << 22) != 0;
                let w = op & (1 << 21) != 0;
                let l = op & (1 << 20) != 0;
                let rn = (op >> 16) & 0xF;
                let rd = (op >> 12) & 0xF;
                let off = if i {
                    ((op >> 4) & 0xF0) | (op & 0xF)
                } else {
                    reg(cpu, op & 0xF)
                };
                let base = reg(cpu, rn);
                let addr = if u {
                    base.wrapping_add(off)
                } else {
                    base.wrapping_sub(off)
                };
                let ea = if p { addr } else { base };
                let sh = (op >> 5) & 3;
                let mut wb = !p || w;
                if l {
                    let v = match sh {
                        1 => b.rd(ea & !1, 2),
                        2 => b.rd(ea, 1) as u8 as i8 as i32 as u32,
                        _ => b.rd(ea & !1, 2) as u16 as i16 as i32 as u32,
                    };
                    if wb {
                        cpu.r[rn as usize] = addr;
                        wb = false;
                    }
                    if rd == 15 {
                        cpu.bx(v);
                    } else {
                        cpu.r[rd as usize] = v;
                    }
                } else {
                    match sh {
                        1 => b.wr(ea & !1, reg(cpu, rd), 2),
                        2 => {
                            // LDRD
                            let lo = b.rd(ea & !3, 4);
                            let hi = b.rd((ea & !3) + 4, 4);
                            cpu.r[rd as usize] = lo;
                            cpu.r[rd as usize + 1] = hi;
                        }
                        _ => {
                            // STRD
                            b.wr(ea & !3, reg(cpu, rd), 4);
                            b.wr((ea & !3) + 4, reg(cpu, rd + 1), 4);
                        }
                    }
                }
                if wb {
                    cpu.r[rn as usize] = addr;
                }
                return;
            }
            // MRS / MSR
            let opc = (op >> 21) & 0xF;
            let s = op & (1 << 20) != 0;
            if !s && (8..=11).contains(&opc) {
                if op & 0x0FBF_0FFF == 0x010F_0000 {
                    let rd = (op >> 12) & 0xF;
                    cpu.r[rd as usize] = if op & (1 << 22) != 0 { 0 } else { cpu.cpsr() };
                    return;
                }
                if op & 0x0DB0_F000 == 0x0120_F000 {
                    let v = if imm {
                        (op & 0xFF).rotate_right(2 * ((op >> 8) & 0xF))
                    } else {
                        reg(cpu, op & 0xF)
                    };
                    if op & (1 << 22) == 0 {
                        let mut mask = 0;
                        if op & (1 << 19) != 0 {
                            mask |= 0xFF00_0000;
                        }
                        if op & (1 << 16) != 0 {
                            mask |= 0xFF;
                        }
                        cpu.set_cpsr(v, mask);
                    }
                    return;
                }
                if op & 0x0F90_0090 == 0x0100_0080 {
                    // SMLAxy / SMLAWy / SMULWy / SMLALxy / SMULxy
                    let rd = (op >> 16) & 0xF;
                    let (rm, rs) = (reg(cpu, op & 0xF), reg(cpu, (op >> 8) & 0xF));
                    let half = |v: u32, top: bool| -> i32 {
                        if top {
                            (v >> 16) as i16 as i32
                        } else {
                            v as i16 as i32
                        }
                    };
                    let x = op & (1 << 5) != 0;
                    let y = op & (1 << 6) != 0;
                    match opc {
                        8 => {
                            let p = half(rm, x).wrapping_mul(half(rs, y));
                            let acc = reg(cpu, (op >> 12) & 0xF) as i32;
                            let (r, ov) = p.overflowing_add(acc);
                            if ov {
                                cpu.q = true;
                            }
                            cpu.r[rd as usize] = r as u32;
                        }
                        9 => {
                            let p = (((rm as i32 as i64) * (half(rs, y) as i64)) >> 16) as i32;
                            if x {
                                cpu.r[rd as usize] = p as u32;
                            } else {
                                let acc = reg(cpu, (op >> 12) & 0xF) as i32;
                                let (r, ov) = p.overflowing_add(acc);
                                if ov {
                                    cpu.q = true;
                                }
                                cpu.r[rd as usize] = r as u32;
                            }
                        }
                        11 => {
                            cpu.r[rd as usize] = half(rm, x).wrapping_mul(half(rs, y)) as u32;
                        }
                        _ => panic!("F12: ARM op {op:#010x} at {pc:#010x} not supported"),
                    }
                    return;
                }
                panic!("F12: ARM op {op:#010x} at {pc:#010x} not supported");
            }
            // data processing
            let rn = (op >> 16) & 0xF;
            let rd = (op >> 12) & 0xF;
            let (b2, sc) = if imm {
                let rot = 2 * ((op >> 8) & 0xF);
                let v = (op & 0xFF).rotate_right(rot);
                (v, if rot == 0 { cpu.c } else { v >> 31 != 0 })
            } else {
                let rm = op & 0xF;
                let t = (op >> 5) & 3;
                if op & 0x10 != 0 {
                    let rs = cpu.r[((op >> 8) & 0xF) as usize] & 0xFF;
                    let v = if rm == 15 {
                        pc + 12
                    } else {
                        cpu.r[rm as usize]
                    };
                    shift(v, t, rs, false, cpu.c)
                } else {
                    shift(reg(cpu, rm), t, (op >> 7) & 0x1F, true, cpu.c)
                }
            };
            let a = if !imm && op & 0x10 != 0 && rn == 15 {
                pc + 12
            } else {
                reg(cpu, rn)
            };
            let logic = |cpu: &mut Cpu, v: u32| -> u32 {
                if s {
                    cpu.c = sc;
                    cpu.nz(v)
                } else {
                    v
                }
            };
            let res = match opc {
                0 => Some(logic(cpu, a & b2)),
                1 => Some(logic(cpu, a ^ b2)),
                2 => Some(cpu.sub(a, b2, true, s)),
                3 => Some(cpu.sub(b2, a, true, s)),
                4 => Some(cpu.add(a, b2, false, s)),
                5 => {
                    let c = cpu.c;
                    Some(cpu.add(a, b2, c, s))
                }
                6 => {
                    let c = cpu.c;
                    Some(cpu.sub(a, b2, c, s))
                }
                7 => {
                    let c = cpu.c;
                    Some(cpu.sub(b2, a, c, s))
                }
                8 => {
                    logic(cpu, a & b2);
                    None
                }
                9 => {
                    logic(cpu, a ^ b2);
                    None
                }
                10 => {
                    cpu.sub(a, b2, true, true);
                    None
                }
                11 => {
                    cpu.add(a, b2, false, true);
                    None
                }
                12 => Some(logic(cpu, a | b2)),
                13 => Some(logic(cpu, b2)),
                14 => Some(logic(cpu, a & !b2)),
                _ => Some(logic(cpu, !b2)),
            };
            if let Some(v) = res {
                if rd == 15 {
                    assert!(!s, "F12: ARM op {op:#010x} at {pc:#010x}: mode return");
                    cpu.r[15] = v & !3;
                } else {
                    cpu.r[rd as usize] = v;
                }
            }
        }
        2 | 3 => {
            // LDR / STR / LDRB / STRB
            if (op >> 25) & 1 != 0 && op & 0x10 != 0 {
                panic!("F12: ARM undefined op {op:#010x} at {pc:#010x}");
            }
            let p = op & (1 << 24) != 0;
            let u = op & (1 << 23) != 0;
            let byte = op & (1 << 22) != 0;
            let w = op & (1 << 21) != 0;
            let l = op & (1 << 20) != 0;
            let rn = (op >> 16) & 0xF;
            let rd = (op >> 12) & 0xF;
            let off = if (op >> 25) & 1 == 0 {
                op & 0xFFF
            } else {
                shift(
                    reg(cpu, op & 0xF),
                    (op >> 5) & 3,
                    (op >> 7) & 0x1F,
                    true,
                    cpu.c,
                )
                .0
            };
            let base = reg(cpu, rn);
            let addr = if u {
                base.wrapping_add(off)
            } else {
                base.wrapping_sub(off)
            };
            let ea = if p { addr } else { base };
            let wb = !p || w;
            if l {
                let v = if byte { b.rd(ea, 1) } else { ldr_word(b, ea) };
                if wb {
                    cpu.r[rn as usize] = addr;
                }
                if rd == 15 {
                    cpu.bx(v);
                } else {
                    cpu.r[rd as usize] = v;
                }
            } else {
                let v = if rd == 15 {
                    pc + 12
                } else {
                    cpu.r[rd as usize]
                };
                if byte {
                    b.wr(ea, v & 0xFF, 1);
                } else {
                    b.wr(ea & !3, v, 4);
                }
                if wb {
                    cpu.r[rn as usize] = addr;
                }
            }
        }
        4 => {
            // LDM / STM
            let p = op & (1 << 24) != 0;
            let u = op & (1 << 23) != 0;
            let w = op & (1 << 21) != 0;
            let l = op & (1 << 20) != 0;
            let rn = ((op >> 16) & 0xF) as usize;
            let list = op & 0xFFFF;
            let n = list.count_ones();
            let base = cpu.r[rn];
            let start = match (p, u) {
                (false, true) => base,
                (true, true) => base + 4,
                (false, false) => base - 4 * n + 4,
                (true, false) => base - 4 * n,
            };
            let end = if u { base + 4 * n } else { base - 4 * n };
            let mut a = start;
            let mut newpc = None;
            for i in 0..16 {
                if list & (1 << i) == 0 {
                    continue;
                }
                if l {
                    let v = b.rd(a & !3, 4);
                    if i == 15 {
                        newpc = Some(v);
                    } else {
                        cpu.r[i] = v;
                    }
                } else {
                    let v = if i == 15 { pc + 12 } else { cpu.r[i] };
                    b.wr(a & !3, v, 4);
                }
                a += 4;
            }
            if w && !(l && list & (1 << rn) != 0) {
                cpu.r[rn] = end;
            }
            if let Some(v) = newpc {
                cpu.bx(v);
            }
        }
        5 => {
            // B / BL
            let off = (((op & 0x00FF_FFFF) << 8) as i32 >> 6) as u32;
            if op & (1 << 24) != 0 {
                cpu.r[14] = pc + 4;
            }
            cpu.r[15] = pc.wrapping_add(8).wrapping_add(off);
        }
        6 => panic!("F12: ARM coprocessor transfer {op:#010x} at {pc:#010x}"),
        _ => {
            if op & (1 << 24) != 0 {
                cpu.r[15] = pc + 4;
                b.swi(cpu, (op >> 16) & 0xFF);
                return;
            }
            if op & 0x10 != 0 {
                // MRC / MCR (CP15 cache / TCM control): reads give 0
                if op & (1 << 20) != 0 {
                    let rd = (op >> 12) & 0xF;
                    if rd != 15 {
                        cpu.r[rd as usize] = 0;
                    }
                }
                return;
            }
            panic!("F12: ARM coprocessor op {op:#010x} at {pc:#010x}");
        }
    }
}

fn thumb<B: Bus>(b: &mut B, c: &mut Cpu, pc: u32, op: u32) {
    let rd = (op & 7) as usize;
    let rs = ((op >> 3) & 7) as usize;
    let rn = ((op >> 6) & 7) as usize;
    let imm5 = (op >> 6) & 0x1F;
    match op >> 11 {
        0..=2 => {
            let (v, carry) = shift(c.r[rs], op >> 11, imm5, true, c.c);
            c.c = carry;
            c.r[rd] = c.nz(v);
        }
        3 => {
            let a = c.r[rs];
            let bb = if op & 0x400 != 0 { rn as u32 } else { c.r[rn] };
            c.r[rd] = if op & 0x200 != 0 {
                c.sub(a, bb, true, true)
            } else {
                c.add(a, bb, false, true)
            };
        }
        4..=7 => {
            let r = ((op >> 8) & 7) as usize;
            let imm = op & 0xFF;
            match (op >> 11) & 3 {
                0 => c.r[r] = c.nz(imm),
                1 => {
                    c.sub(c.r[r], imm, true, true);
                }
                2 => c.r[r] = c.add(c.r[r], imm, false, true),
                _ => c.r[r] = c.sub(c.r[r], imm, true, true),
            }
        }
        8 => {
            if op & 0x400 == 0 {
                let a = c.r[rd];
                let bb = c.r[rs];
                match (op >> 6) & 0xF {
                    0 => c.r[rd] = c.nz(a & bb),
                    1 => c.r[rd] = c.nz(a ^ bb),
                    k @ (2 | 3 | 4 | 7) => {
                        let t = match k {
                            2 => 0,
                            3 => 1,
                            4 => 2,
                            _ => 3,
                        };
                        let (v, carry) = shift(a, t, bb & 0xFF, false, c.c);
                        c.c = carry;
                        c.r[rd] = c.nz(v);
                    }
                    5 => {
                        let cc = c.c;
                        c.r[rd] = c.add(a, bb, cc, true)
                    }
                    6 => {
                        let cc = c.c;
                        c.r[rd] = c.sub(a, bb, cc, true)
                    }
                    8 => {
                        c.nz(a & bb);
                    }
                    9 => c.r[rd] = c.sub(0, bb, true, true),
                    10 => {
                        c.sub(a, bb, true, true);
                    }
                    11 => {
                        c.add(a, bb, false, true);
                    }
                    12 => c.r[rd] = c.nz(a | bb),
                    13 => c.r[rd] = c.nz(a.wrapping_mul(bb)),
                    14 => c.r[rd] = c.nz(a & !bb),
                    _ => c.r[rd] = c.nz(!bb),
                }
            } else {
                let h1 = ((op >> 7) & 1) as usize;
                let h2 = ((op >> 6) & 1) as usize;
                let d = rd + 8 * h1;
                let s = rs + 8 * h2;
                let sv = if s == 15 { pc + 4 } else { c.r[s] };
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
                        c.sub(dv, sv, true, true);
                    }
                    2 => {
                        if d == 15 {
                            c.r[15] = sv & !1;
                        } else {
                            c.r[d] = sv;
                        }
                    }
                    _ => {
                        if h1 != 0 {
                            c.r[14] = (pc + 2) | 1;
                        }
                        c.bx(sv);
                    }
                }
            }
        }
        9 => {
            let r = ((op >> 8) & 7) as usize;
            let a = ((pc + 4) & !3) + 4 * (op & 0xFF);
            c.r[r] = b.rd(a, 4);
        }
        10 | 11 => {
            let a = c.r[rs].wrapping_add(c.r[rn]);
            match (op >> 9) & 7 {
                0 => b.wr(a & !3, c.r[rd], 4),
                1 => b.wr(a & !1, c.r[rd] & 0xFFFF, 2),
                2 => b.wr(a, c.r[rd] & 0xFF, 1),
                3 => c.r[rd] = b.rd(a, 1) as u8 as i8 as i32 as u32,
                4 => c.r[rd] = ldr_word(b, a),
                5 => c.r[rd] = b.rd(a & !1, 2),
                6 => c.r[rd] = b.rd(a, 1),
                _ => c.r[rd] = b.rd(a & !1, 2) as u16 as i16 as i32 as u32,
            }
        }
        12 => b.wr(c.r[rs].wrapping_add(imm5 * 4) & !3, c.r[rd], 4),
        13 => c.r[rd] = ldr_word(b, c.r[rs].wrapping_add(imm5 * 4)),
        14 => b.wr(c.r[rs].wrapping_add(imm5), c.r[rd] & 0xFF, 1),
        15 => c.r[rd] = b.rd(c.r[rs].wrapping_add(imm5), 1),
        16 => b.wr(c.r[rs].wrapping_add(imm5 * 2) & !1, c.r[rd] & 0xFFFF, 2),
        17 => c.r[rd] = b.rd(c.r[rs].wrapping_add(imm5 * 2) & !1, 2),
        18 => {
            let r = ((op >> 8) & 7) as usize;
            b.wr(c.r[13] + 4 * (op & 0xFF), c.r[r], 4);
        }
        19 => {
            let r = ((op >> 8) & 7) as usize;
            c.r[r] = ldr_word(b, c.r[13] + 4 * (op & 0xFF));
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
                let mut list: Vec<usize> = (0..8).filter(|i| op & (1 << i) != 0).collect();
                if op & 0x100 != 0 {
                    list.push(14);
                }
                c.r[13] -= 4 * list.len() as u32;
                let mut a = c.r[13];
                for i in list {
                    b.wr(a, c.r[i], 4);
                    a += 4;
                }
            }
            12 | 13 => {
                let mut a = c.r[13];
                for i in 0..8 {
                    if op & (1 << i) != 0 {
                        c.r[i] = b.rd(a, 4);
                        a += 4;
                    }
                }
                if op & 0x100 != 0 {
                    let v = b.rd(a, 4);
                    a += 4;
                    c.r[13] = a;
                    c.bx(v);
                    return;
                }
                c.r[13] = a;
            }
            14 => panic!("F12: thumb bkpt at {pc:#010x}"),
            _ => panic!("F12: thumb op {op:#06x} at {pc:#010x} not supported"),
        },
        24 => {
            let rb = ((op >> 8) & 7) as usize;
            let mut a = c.r[rb];
            for i in 0..8 {
                if op & (1 << i) != 0 {
                    b.wr(a & !3, c.r[i], 4);
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
                    c.r[i] = b.rd(a & !3, 4);
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
                b.swi(c, op & 0xFF);
                return;
            }
            assert!(
                cond != 0xE,
                "F12: thumb undefined op {op:#06x} at {pc:#010x}"
            );
            if c.cond(cond) {
                let off = ((op & 0xFF) as u8 as i8 as i32) * 2;
                c.r[15] = (pc as i32 + 4 + off) as u32;
            }
        }
        28 => {
            let off = (((op & 0x7FF) << 21) as i32 >> 20) as u32;
            c.r[15] = pc.wrapping_add(4).wrapping_add(off);
        }
        30 => {
            let off = (((op & 0x7FF) << 21) as i32 >> 9) as u32;
            c.r[14] = pc.wrapping_add(4).wrapping_add(off);
        }
        31 => {
            let t = c.r[14].wrapping_add((op & 0x7FF) << 1);
            c.r[14] = (pc + 2) | 1;
            c.r[15] = t & !1;
        }
        29 => {
            let t = c.r[14].wrapping_add((op & 0x7FF) << 1) & !3;
            c.r[14] = (pc + 2) | 1;
            c.thumb = false;
            c.r[15] = t;
        }
        _ => panic!("F12: thumb op {op:#06x} at {pc:#010x} not supported"),
    }
}
