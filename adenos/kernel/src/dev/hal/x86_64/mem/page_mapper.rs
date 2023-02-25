use crate::*;
use super::FRAME_ALLOCATOR;
use super::PHYSICAL_MEMORY_OFFSET;
use x86_64::{structures::paging::{page_table::{PageTableEntry, PageTableFlags}, PageTable, PageTableIndex}, {registers::control::Cr3, VirtAddr}, PhysAddr, {instructions::tlb}};

#[inline(always)]
pub fn align(addr: u64) -> u64 {
    (addr / 0x1000) * 0x1000
}

pub fn unmap_userspace_page_tables(page_table_addr: u64) {
    let page_table = addr_to_page_table(page_table_addr);
    let l3_page_table = addr_to_page_table(page_table[0].addr().as_u64());
    let l2_page_table = addr_to_page_table(l3_page_table[1].addr().as_u64());
    for (_, ent) in l2_page_table.iter_mut().enumerate() {
        if !ent.is_unused() {
            let l1_page_table = addr_to_page_table(ent.addr().as_u64());
            for (_, ent) in l1_page_table.iter_mut().enumerate() {
                if !ent.is_unused() {
                    free_frame(ent.addr().as_u64());
                    ent.set_unused();
                }
            }
            free_frame(ent.addr().as_u64());
            ent.set_unused();
        }
    }
    free_frame(l3_page_table[1].addr().as_u64());
    free_frame(page_table[0].addr().as_u64());
    free_frame(page_table_addr);
    tlb::flush_all();
}

pub fn get_l1_entry(addr: u64) -> Option<&'static mut PageTableEntry> {
    let (frame, _) = Cr3::read();
    let mut frame = frame.start_address().as_u64();
    let virt_addr = VirtAddr::new(addr as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    let table_virt_addr = frame + unsafe { PHYSICAL_MEMORY_OFFSET };
    let table = unsafe { &mut *(table_virt_addr as *mut PageTable) };
    let mut ent = &mut table[table_indexes[0]];
    for i in 0..4 {
        let table_virt_addr = frame + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &mut *(table_virt_addr as *mut PageTable) };
        ent = &mut table[table_indexes[i]];
        frame = if ent.is_unused() {
            return None
        } else {
            align(ent.addr().as_u64())
        };
    }
    Some(ent)
}

pub fn get_l4_table() -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let frame = frame.start_address().as_u64() + unsafe { PHYSICAL_MEMORY_OFFSET };
    unsafe {
        (frame as *mut PageTable).as_mut().unwrap()
    }
}

pub fn set_write_combining(buffer: *const u8, size: usize) {
    set_flags(buffer, size, PageTableFlags::WRITE_THROUGH | PageTableFlags::NO_CACHE | PageTableFlags::HUGE_PAGE)
}

pub fn set_uncacheable(buffer: *const u8, size: usize) {
    set_flags(buffer, size, PageTableFlags::NO_CACHE)
}

pub fn set_flags(buffer: *const u8, size: usize, add_flags: PageTableFlags) {
    for addr in ((buffer as u64)..((buffer as u64) + size as u64)).step_by(0x1000) {
        let ent = get_l1_entry(align(addr)).unwrap();
        let flags = ent.flags();
        ent.set_flags(flags | add_flags); // PA7
    }
}

pub fn copy_over_kernel_tables_but_not_userspace_ones() -> &'static mut PageTable {
    let kernel_page_table = unsafe { addr_to_page_table(super::KERNEL_PAGE_TABLE) };
    let user_page_table = unsafe { new_l4_table() };
    // copy L4 entries but not first one!
    let mut l4_entries = kernel_page_table.iter().enumerate();
    l4_entries.next();
    for (i, ent) in l4_entries {
        if !ent.is_unused() {
            user_page_table[i] = ent.clone();
        }
    }
    // get kernel L3 table at L4 entry 0
    let kernel_l3_table = addr_to_page_table(kernel_page_table[0].addr().as_u64());
    // create new L3 table at L4 entry 0
    let user_l3_table = unsafe { table_entry_new_table(user_page_table, PageTableIndex::new(0), Some(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE)).unwrap() };
    // set L3 table index 0 to kernel L3 table index 0 BUT in a new table
    unsafe { table_entry(user_l3_table, PageTableIndex::new(0), kernel_l3_table[0].addr().as_u64(), None, None) };
    /*serial_println!("\nKernel page table: {:x}\nUser page table: {:x}\nKernel L4 entry 0: {:x}\nUser L4 entry 0: {:x}\nKernel L3 entry 0: {:x}\nUser L3 entry 0: {:x}\n",
    kernel_page_table_addr, user_page_table_addr,
    kernel_page_table[0].addr().as_u64(), user_l4_addr,
    kernel_l3_table[0].addr().as_u64(), user_l3_table[0].addr().as_u64());*/
    user_page_table
}

pub unsafe fn table_entry_new_table(page_table: &mut PageTable, entry: PageTableIndex, flags: Option<PageTableFlags>) -> Option<&mut PageTable> {
    if page_table[entry].is_unused() {
        page_table[entry].set_addr(PhysAddr::new(new_frame_zeroed()),
        flags.unwrap_or(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL));
    }
    let virt_table = page_table[entry].frame().unwrap().start_address().as_u64() + PHYSICAL_MEMORY_OFFSET;
    tlb::flush(VirtAddr::new(virt_table));
    Some(&mut *(virt_table as *mut PageTable))
}

pub unsafe fn table_entry(page_table: &mut PageTable, entry: PageTableIndex, frame: u64, flags: Option<PageTableFlags>, flush_addr: Option<u64>) -> Option<u64> {
    if page_table[entry].is_unused() {
        page_table[entry].set_addr(PhysAddr::new(frame), flags.unwrap_or(PageTableFlags::WRITABLE | PageTableFlags::GLOBAL | PageTableFlags::PRESENT));
        if let Some(flush_addr) = flush_addr {
            tlb::flush(VirtAddr::new(flush_addr));
        }
    }
    if let Ok(frm) = page_table[entry].frame() {
        Some(frm.start_address().as_u64())
    } else {
        None
    }
}

pub unsafe fn new_l4_table() -> &'static mut PageTable {
    addr_to_page_table(new_frame_zeroed())
}

pub fn map_l1_table(l4_table: &mut PageTable, page: u64, flags: Option<PageTableFlags>) -> Option<&mut PageTable> {
    let virt_addr = VirtAddr::new(page);
    let l3_table = unsafe { table_entry_new_table(l4_table, virt_addr.p4_index(), flags).or_else(|| return None).unwrap() };
    let l2_table = unsafe { table_entry_new_table(l3_table, virt_addr.p3_index(), flags).or_else(|| return None).unwrap() };
    let l1_table = unsafe { table_entry_new_table(l2_table, virt_addr.p2_index(), flags).or_else(|| return None).unwrap() };
    Some(l1_table)
}

pub fn map_addr(l4_table: &mut PageTable, virt_addr: u64, phys_addr: u64, flags: Option<PageTableFlags>) {
    let page_table = map_l1_table(l4_table, align(virt_addr), flags).unwrap();
    unsafe { table_entry(page_table, VirtAddr::new(virt_addr).p1_index(), (phys_addr / 0x1000) * 0x1000, flags, Some(virt_addr)); }
}

pub fn unmap_addr(page_table: &mut PageTable, virt_addr: u64) {
    let mut frame = page_table as *const _ as u64 - unsafe { PHYSICAL_MEMORY_OFFSET };
    let virt_addr = VirtAddr::new(virt_addr as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        let table_virt_addr = frame + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &mut *(table_virt_addr as *mut PageTable) };
        let ent = &mut table[table_indexes[i]];
        frame = if ent.is_unused() {
            return;
        } else {
            if i == 3 {
                ent.set_unused();
                return;
            } else {
                align(ent.addr().as_u64())
            }
        };
    }
}

pub fn map_page_to_frame(l4_table: &mut PageTable,page: u64, frame: u64, flags: Option<PageTableFlags>) {
    let page_table = map_l1_table(l4_table, page, flags).unwrap();
    unsafe { table_entry(page_table, VirtAddr::new(page).p1_index(), frame, flags, Some(page)); }
}

pub fn new_frame_zeroed() -> u64 {
    unsafe {
        let frm_phys = FRAME_ALLOCATOR.allocate_frame();
        let frm_clr = (frm_phys + PHYSICAL_MEMORY_OFFSET) as *mut u8;
        for i in 0..0x1000 {
            *frm_clr.offset(i) = 0;
        }
        frm_phys
    }
}

pub fn free_frame(frame: u64) {
    unsafe { super::FRAME_ALLOCATOR.free_frame(frame) };
}

pub fn translate_addr(virt_addr: usize) -> Option<u64> {
    let (frame, _) = Cr3::read();
    let mut frame = frame.start_address().as_u64();
    let virt_addr = VirtAddr::new(virt_addr as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        let table_virt_addr = frame + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &*(table_virt_addr as *const PageTable) };
        let ent = &table[table_indexes[i]];
        frame = if ent.is_unused() {
            return None
        } else {
            align(ent.addr().as_u64())
        };
    }
    Some(frame + u64::from(virt_addr.page_offset()))
}

pub fn translate_addr_using_table(page_table: &PageTable, virt_addr: usize) -> Option<u64> {
    let mut frame = page_table as *const _ as u64 - unsafe { PHYSICAL_MEMORY_OFFSET };
    let virt_addr = VirtAddr::new(virt_addr as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        let table_virt_addr = frame + unsafe { PHYSICAL_MEMORY_OFFSET };
        let table = unsafe { &*(table_virt_addr as *const PageTable) };
        let ent = &table[table_indexes[i]];
        frame = if ent.is_unused() {
            return None
        } else {
            align(ent.addr().as_u64())
        };
    }
    Some(frame + u64::from(virt_addr.page_offset()))
}

pub unsafe fn show_which_page_tables(address: usize) {
    let virt_addr = VirtAddr::new(address as u64);
    let table_indexes = [
        virt_addr.p4_index(), virt_addr.p3_index(), virt_addr.p2_index(), virt_addr.p1_index()
    ];
    for i in 0..4 {
        serial_println!("LEVEL {}: {}", 4 - i, u16::from(table_indexes[i]));
    }
}

pub fn addr_to_page_table(addr: u64) -> &'static mut PageTable {
    unsafe {
        ((addr + PHYSICAL_MEMORY_OFFSET) as *mut PageTable).as_mut().unwrap()
    }
}