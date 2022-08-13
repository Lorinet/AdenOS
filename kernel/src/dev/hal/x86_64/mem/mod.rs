use core::alloc::Layout;

use crate::*;
use allocator;
use bootloader::bootinfo;
use alloc::{vec::Vec, alloc::alloc};
use x86_64::{ structures::paging::{mapper::MapToError, Page, PageTable, PhysFrame, PageTableFlags, Size4KiB}, VirtAddr, PhysAddr, registers::control::{Cr3, Cr3Flags}, instructions::tlb };

mod frame_allocator;
pub mod page_mapper;

pub static mut PHYSICAL_MEMORY_OFFSET: u64 = 0;
pub static mut BOOT_MEMORY_MAP: Option<&bootinfo::MemoryMap> = None;

pub const KERNEL_HEAP_START: usize = 0x_4444_4444_0000;
pub const KERNEL_HEAP_SIZE: usize = 64 * 4096;

pub fn init() {
    println!("Initializing kernel heap...");
    unsafe { frame_allocator::init(BOOT_MEMORY_MAP.unwrap()) };
    println!("Initialized frame allocator");
    init_heap().expect("KERNEL_HEAP_ALLOCATION_FAILED");
}

pub unsafe fn enable_page_table(page_table: &'static mut PageTable) {
    let phys_addr = page_mapper::translate_addr(page_table as *const PageTable as usize).unwrap();
    println!("L4 address CR3: {}", phys_addr);
    Cr3::write(PhysFrame::from_start_address(PhysAddr::new(phys_addr as u64)).expect("userspace page table not aligned"), Cr3Flags::all());
    tlb::flush_all();
}

unsafe fn debug_page_tables(page_table: &PageTable) {
    serial_println!("DEBUG PAGE TABLE");
    for (i, ent) in page_table.iter().enumerate() {
        if !ent.is_unused() {
            serial_println!("L4 entry {}: {:#018o}", i, ent.addr());
        }
    }
}

fn show_which_page_tables(virt_addr: u64) {
    let l4e = (virt_addr >> 39) & 0o777;
    println!("Address {:#018x} is in L3 table {}", virt_addr, l4e);
}

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(KERNEL_HEAP_START as u64);
        let heap_end = heap_start + KERNEL_HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = unsafe { frame_allocator::allocate_frame().ok_or(MapToError::FrameAllocationFailed)? };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;
        page_mapper::map_page_to_frame(page, frame, Some(flags)).or_else(|_| Err(MapToError::FrameAllocationFailed::<Size4KiB>))?;
    }

    unsafe {
        #[cfg(feature = "slab_allocator")]
        allocator::ALLOCATOR.init(KERNEL_HEAP_START, KERNEL_HEAP_SIZE);
        #[cfg(feature = "linked_list_allocator")]
        allocator::ALLOCATOR.lock().init(KERNEL_HEAP_START as *mut u8, KERNEL_HEAP_SIZE);
    }

    unsafe { 
        println!("Physical memory virtual base: {:#018x}", PHYSICAL_MEMORY_OFFSET);
        println!("Kernel heap virtual base: {:#018x}", KERNEL_HEAP_START);
        println!("Kernel heap size: {:#018x}", KERNEL_HEAP_SIZE);
    }

    Ok(())
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable
{
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}
