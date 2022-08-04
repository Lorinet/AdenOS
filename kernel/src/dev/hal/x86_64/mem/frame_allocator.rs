use crate::*;
use bootloader::bootinfo;
use x86_64::{ PhysAddr, structures::paging::{ PhysFrame } };

static mut BOOT_INFO_MEMORY_MAP: Option<&'static bootinfo::MemoryMap> = None;
static mut NEXT_FRAME: usize = 0;

pub unsafe fn init(memory_map: &'static bootinfo::MemoryMap) {
    BOOT_INFO_MEMORY_MAP = Some(memory_map);
}

fn usable_frames() -> impl Iterator<Item = PhysFrame> {
    // get usable regions from memory map
    let regions = unsafe { BOOT_INFO_MEMORY_MAP.as_ref().unwrap().iter() };
    let usable_regions = regions
        .filter(|r| r.region_type == bootinfo::MemoryRegionType::Usable);
    // map each region to its address range
    let addr_ranges = usable_regions
        .map(|r| r.range.start_addr()..r.range.end_addr());
    // transform to an iterator of frame start addresses
    let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
    // create `PhysFrame` types from the start addresses
    frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}

pub unsafe fn allocate_frame() -> Option<PhysFrame> {
    let frame = usable_frames().nth(NEXT_FRAME);
    NEXT_FRAME += 1;
    frame
}
