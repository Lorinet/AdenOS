use crate::*;
use alloc::alloc::{GlobalAlloc, Layout};

#[cfg(feature = "slab_allocator")]
use slab_allocator_rs::LockedHeap;
#[cfg(feature = "slab_allocator")]
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[cfg(feature = "linked_list_allocator")]
use linked_list_allocator::LockedHeap;
#[cfg(feature = "linked_list_allocator")]
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();
