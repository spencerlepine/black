//! Call arguments with buffers: what a port or hook passes by pointer to a stack local of its own
//! (`CArg::InBuf` / `OutBuf`, the black-script hooks' `&[u8; N]` / `&mut [u8; N]`) lives on the
//! interpreter stack for the call, as the original's local would.

use super::Env;

enum Item {
    V(u32),
    /// Bytes copied to the scratch area before the call (and read back after it when `out`).
    B {
        bytes: Vec<u8>,
        out: bool,
    },
}

/// The arguments of one call (in register / stack order).
#[derive(Default)]
pub struct Args {
    items: Vec<Item>,
    outs: Vec<Vec<u8>>,
    /// `items` index of each out buffer, by out key.
    out_idx: Vec<usize>,
}

impl Args {
    pub fn new() -> Self {
        Args::default()
    }

    /// A value argument.
    pub fn v(&mut self, v: u32) {
        self.items.push(Item::V(v));
    }

    /// A pointer to a copy of `b` (the callee reads it).
    pub fn inb(&mut self, b: &[u8]) {
        self.items.push(Item::B {
            bytes: b.to_vec(),
            out: false,
        });
    }

    /// A pointer to a copy of the words `w`.
    pub fn inw(&mut self, w: &[u32]) {
        let b: Vec<u8> = w.iter().flat_map(|x| x.to_le_bytes()).collect();
        self.inb(&b);
    }

    /// A pointer to `n` zeroed bytes the callee writes; returns the key of [`Args::out_bytes`].
    pub fn outb(&mut self, n: usize) -> usize {
        self.inoutb(&vec![0; n])
    }

    /// A pointer to a copy of `b` that the callee may change; returns the key of [`Args::out_bytes`].
    pub fn inoutb(&mut self, b: &[u8]) -> usize {
        self.out_idx.push(self.items.len());
        self.items.push(Item::B {
            bytes: b.to_vec(),
            out: true,
        });
        self.out_idx.len() - 1
    }

    /// The bytes of out buffer `k` after the call.
    pub fn out_bytes(&self, k: usize) -> Vec<u8> {
        self.outs[k].clone()
    }

    /// Out buffer `k` as words.
    pub fn out_words(&self, k: usize, w: &mut [u32]) {
        for (i, x) in w.iter_mut().enumerate() {
            *x = u32::from_le_bytes(self.outs[k][4 * i..4 * i + 4].try_into().unwrap());
        }
    }
}

impl Env {
    /// Calls `addr` with `a` (buffers placed on the interpreter stack below the current frame);
    /// returns r0 | r1 << 32.
    pub fn call_args(&mut self, addr: u32, a: &mut Args) -> u64 {
        let saved = self.sp;
        let mut words = Vec::with_capacity(a.items.len());
        let mut placed = Vec::new();
        for it in &a.items {
            match it {
                Item::V(v) => words.push(*v),
                Item::B { bytes, out } => {
                    let n = (bytes.len() as u32 + 7) & !7;
                    self.sp -= n.max(8);
                    let p = self.sp;
                    self.write_bytes(p, bytes);
                    words.push(p);
                    if *out {
                        placed.push((p, bytes.len()));
                    }
                }
            }
        }
        let r = self.call_words(addr, &words);
        a.outs = placed
            .into_iter()
            .map(|(p, n)| self.read_bytes(p, n))
            .collect();
        self.sp = saved;
        r
    }
}
