//! C: include/heap.h. The real allocator is undecompiled, so it is a trait.

/// C: `enum HeapID`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum HeapId {
    Unknown = 0,
}

/// Models `Heap_AllocDebug` / `Heap_Free`. Blocks are owned byte buffers.
pub trait Heap {
    /// C: `Heap_AllocDebug @ (undecompiled)`. Returns `None` when allocation fails.
    fn alloc_debug(
        &mut self,
        heap_id: HeapId,
        size: u32,
        unk2: bool,
        file: &'static str,
        line: u32,
    ) -> Option<Box<[u8]>>;

    /// C: `Heap_Free @ (undecompiled)`
    fn free(&mut self, block: Box<[u8]>);
}

/// Host heap: zero-filled `Box<[u8]>` allocations with simple accounting (for tests/tools).
#[derive(Debug, Default)]
pub struct HostHeap {
    pub live_blocks: usize,
    pub total_allocs: usize,
    /// (file, line) of the most recent allocation.
    pub last_site: Option<(&'static str, u32)>,
    /// When set, the next allocation fails.
    pub fail_next: bool,
}

impl Heap for HostHeap {
    fn alloc_debug(
        &mut self,
        _heap_id: HeapId,
        size: u32,
        _unk2: bool,
        file: &'static str,
        line: u32,
    ) -> Option<Box<[u8]>> {
        if self.fail_next {
            self.fail_next = false;
            return None;
        }
        self.live_blocks += 1;
        self.total_allocs += 1;
        self.last_site = Some((file, line));
        Some(vec![0u8; size as usize].into_boxed_slice())
    }

    fn free(&mut self, _block: Box<[u8]>) {
        self.live_blocks -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_heap_accounting() {
        let mut h = HostHeap::default();
        let b = h.alloc_debug(HeapId::Unknown, 16, true, "x.c", 3).unwrap();
        assert_eq!(b.len(), 16);
        assert!(b.iter().all(|&x| x == 0));
        assert_eq!(h.live_blocks, 1);
        assert_eq!(h.last_site, Some(("x.c", 3)));
        h.free(b);
        assert_eq!(h.live_blocks, 0);
        h.fail_next = true;
        assert!(h.alloc_debug(HeapId::Unknown, 1, true, "x.c", 1).is_none());
        assert!(h.alloc_debug(HeapId::Unknown, 1, true, "x.c", 1).is_some());
    }
}
