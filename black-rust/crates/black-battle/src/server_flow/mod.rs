//! Server flow (`btl_server_flow.c`, asm/battle_ungrouped.s, 0x021BAA94-0x021CD800).
//!
//! Split by B9 into address-contiguous areas (one module each, see MIGRATION-AGENT.md §7).
//! Each area adds its own `impl SvFlowWork` block with accessors for the fields it uses; field
//! offsets are named as they are identified. Code outside this layer (server, client, main
//! module, Pokemon container) is reached through traits, never re-implemented here.

pub mod adapter;
pub mod api;
pub mod cmd_queue;
pub mod core1;
pub mod core2;
pub mod core3;
pub mod core4;
pub mod core5;
pub mod core6;
pub mod env;
pub mod events;
pub mod handex;
pub mod hem;
pub mod server_side;

#[cfg(test)]
pub(crate) mod testkit;

/// The original's `BTL_SVFLOW_WORK`: `0x212C` bytes, allocated and cleared by the flow create
/// routine at 0x021BB968 (not ported yet).
///
/// Kept as a byte image so the Unicorn diff can compare it whole; known fields get typed
/// accessors.
#[derive(Clone)]
pub struct SvFlowWork {
    pub mem: Box<[u8; SvFlowWork::SIZE]>,
}

impl SvFlowWork {
    pub const SIZE: usize = 0x212C;

    pub fn new() -> Self {
        SvFlowWork {
            mem: Box::new([0; Self::SIZE]),
        }
    }

    pub fn u8_at(&self, ofs: usize) -> u8 {
        self.mem[ofs]
    }

    pub fn set_u8_at(&mut self, ofs: usize, v: u8) {
        self.mem[ofs] = v;
    }

    pub fn u16_at(&self, ofs: usize) -> u16 {
        u16::from_le_bytes([self.mem[ofs], self.mem[ofs + 1]])
    }

    pub fn set_u16_at(&mut self, ofs: usize, v: u16) {
        self.mem[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
    }

    pub fn u32_at(&self, ofs: usize) -> u32 {
        u32::from_le_bytes(self.mem[ofs..ofs + 4].try_into().unwrap())
    }

    pub fn set_u32_at(&mut self, ofs: usize, v: u32) {
        self.mem[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
    }
}

impl Default for SvFlowWork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_are_little_endian() {
        let mut w = SvFlowWork::new();
        w.set_u32_at(0x10, 0x1234_5678);
        assert_eq!(w.u8_at(0x10), 0x78);
        assert_eq!(w.u16_at(0x12), 0x1234);
        w.set_u16_at(0x780, 0xBEEF);
        assert_eq!(w.u32_at(0x780), 0xBEEF);
    }
}
