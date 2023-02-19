use crate::*;
use bootloader::boot_info;
use frame_allocator::*;
use x86_64::{ structures::paging::{Page, PageTable, PhysFrame, PageTableFlags, Size4KiB}, VirtAddr, PhysAddr, registers::control::{Cr3, Cr3Flags}, instructions::tlb };

use self::page_mapper::addr_to_page_table;

pub mod frame_allocator;
pub mod page_mapper;

pub static mut PHYSICAL_MEMORY_OFFSET: u64 = 0;
pub static mut BOOT_MEMORY_MAP: Option<&boot_info::MemoryRegions> = None;
pub static mut FREE_MEMORY: usize = 0;
pub static mut FRAME_ALLOCATOR: BitmapFrameAllocator = BitmapFrameAllocator::new_uninit();
pub static mut KERNEL_PAGE_TABLE: u64 = 0;

pub const KERNEL_HEAP_START: usize = 0x_4444_4444_0000;
pub const KERNEL_HEAP_SIZE: usize = 0x1000 * 64;

pub fn init() {
    let (pt, _) = Cr3::read();
    unsafe { KERNEL_PAGE_TABLE = pt.start_address().as_u64(); }
    unsafe { FRAME_ALLOCATOR = BitmapFrameAllocator::new() };
    init_heap();
}

pub unsafe fn enable_page_table(page_table: &'static PageTable) {
    let phys_addr = PhysAddr::new((page_table as *const PageTable as u64) - PHYSICAL_MEMORY_OFFSET);
    Cr3::write(PhysFrame::from_start_address(phys_addr).expect("userspace page table not aligned"), Cr3Flags::all());
    tlb::flush_all();
}

pub fn init_heap() {
    let page_range = {
        let heap_start = VirtAddr::new(KERNEL_HEAP_START as u64);
        let heap_end = heap_start + KERNEL_HEAP_SIZE - 1u64;
        let heap_start_page: Page<Size4KiB> = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    }.map(|frm| frm.start_address().as_u64());

    let l4_table = addr_to_page_table(unsafe { KERNEL_PAGE_TABLE });

    for page in page_range {
        let frame = unsafe { FRAME_ALLOCATOR.allocate_frame() };
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;
        page_mapper::map_page_to_frame(l4_table, page, frame, Some(flags));
    }

    unsafe { infinity::allocator::ALLOCATOR.init(KERNEL_HEAP_START, KERNEL_HEAP_SIZE); }

    // kernel heap is initialized, we can set up bitmap
    unsafe { FRAME_ALLOCATOR.lock_all() };
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable
{
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

pub unsafe fn print_page_tables() {
    let active_page_table = active_level_4_table(VirtAddr::new(PHYSICAL_MEMORY_OFFSET));
    for ent in active_page_table.iter().enumerate() {
        if !ent.1.is_unused() {
            serial_println!("L4 entry {}: {:x?}", ent.0, ent.1);
        }
    }
}

pub unsafe fn show_which_page_tables(address: usize) {
    page_mapper::show_which_page_tables(address);
}