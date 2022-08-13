use core::alloc::Layout;

use crate::*;
use super::frame_allocator;
use super::PHYSICAL_MEMORY_OFFSET;
use alloc::alloc;
use x86_64::{structures::paging::{PageTable, PageTableIndex, PageTableFlags, Page, PhysFrame, Size4KiB, page_table::FrameError}, {registers::control::Cr3, VirtAddr}, PhysAddr, {instructions::tlb}};

pub unsafe fn new_l4_table() -> &'static mut PageTable {
    let table = alloc::alloc_zeroed(Layout::for_value(&PageTable::new())) as *mut PageTable;
    &mut *table
}

pub fn map_l1_table(page: Page<Size4KiB>, flags: Option<PageTableFlags>) -> Result<&'static mut PageTable, FrameError> {
    let l4_table = unsafe { &mut *((Cr3::read().0.start_address().as_u64() + PHYSICAL_MEMORY_OFFSET) as *mut PageTable) };
    let virt_addr = page.start_address();
    let l3_table = unsafe { table_entry_new_table(l4_table, virt_addr.p4_index(), flags)? };
    let l2_table = unsafe { table_entry_new_table(l3_table, virt_addr.p3_index(), flags)? };
    let l1_table = unsafe { table_entry_new_table(l2_table, virt_addr.p2_index(), flags)? };
    Ok(l1_table)
}

pub fn map_addr(virt_addr: usize, phys_addr: usize, flags: Option<PageTableFlags>) -> Result<(), FrameError> {
    let virt_addr = VirtAddr::new(virt_addr as u64);
    let phys_addr = PhysAddr::new(phys_addr as u64);
    let page_table = map_l1_table(Page::containing_address(virt_addr), flags)?;
    unsafe { table_entry(page_table, virt_addr.p1_index(), PhysFrame::containing_address(phys_addr), flags, virt_addr)?; }
    Ok(())
}

pub fn map_page_to_frame(page: Page, frame: PhysFrame, flags: Option<PageTableFlags>) -> Result<(), FrameError> {
    let page_table = map_l1_table(page, flags)?;
    unsafe { table_entry(page_table, page.p1_index(), frame, flags, page.start_address())?; }
    Ok(())
}

pub fn get_flags(page: Page) {
    let (mut frame, _) = Cr3::read();
    let virt_addr = page.start_address();
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        let table_virt_addr = frame.start_address().as_u64() + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &*(table_virt_addr as *const PageTable) };
        let ent = &table[table_indexes[i]];
        frame = if ent.is_unused() {
            return;
        } else {
            PhysFrame::containing_address(ent.addr())
        };
    }
}

pub fn translate_addr(virt_addr: usize) -> Option<usize> {
    let (mut frame, _) = Cr3::read();
    let virt_addr = VirtAddr::new(virt_addr as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        let table_virt_addr = frame.start_address().as_u64() + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &*(table_virt_addr as *const PageTable) };
        let ent = &table[table_indexes[i]];
        frame = if ent.is_unused() {
            return None
        } else {
            PhysFrame::containing_address(ent.addr())
        };
    }
    Some((frame.start_address() + u64::from(virt_addr.page_offset())).as_u64() as usize)
}

pub unsafe fn table_entry_new_table(page_table: &mut PageTable, entry: PageTableIndex, flags: Option<PageTableFlags>) -> Result<&mut PageTable, FrameError> {
    let frm_phys = frame_allocator::allocate_frame().unwrap().start_address();
    let frm_clr = (frm_phys.as_u64() + PHYSICAL_MEMORY_OFFSET) as *mut u8;
    for i in 0..0x1000 {
        *frm_clr.offset(i) = 0;
    }
    if page_table[entry].is_unused() {
        page_table[entry].set_addr(frm_phys,
        flags.unwrap_or(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL));
    }
    let virt_table = page_table[entry].frame()?.start_address().as_u64() + PHYSICAL_MEMORY_OFFSET;
    tlb::flush(VirtAddr::new(virt_table));
    Ok(&mut *(virt_table as *mut PageTable))
}

pub unsafe fn table_entry(page_table: &mut PageTable, entry: PageTableIndex, frame: PhysFrame, flags: Option<PageTableFlags>, flush_addr: VirtAddr) -> Result<PhysFrame<Size4KiB>, FrameError> {
    if page_table[entry].is_unused() {
        page_table[entry].set_addr(frame.start_address(), flags.unwrap_or(PageTableFlags::WRITABLE | PageTableFlags::GLOBAL | PageTableFlags::PRESENT));
        tlb::flush(flush_addr);
    }
    page_table[entry].frame()
}