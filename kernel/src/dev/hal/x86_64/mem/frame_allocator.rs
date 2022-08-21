use crate::*;
use dev::hal::mem;
use bootloader::boot_info;
use alloc::{vec, vec::Vec};
use x86_64::{ PhysAddr, structures::paging::{ PhysFrame } };

pub struct BitmapFrameAllocator {
    bitmap: *mut u8,
    free_pages: usize,
    used_pages: usize,
    number_of_pages: usize,
    next_free: usize,
    first_usable_address: u64,
}

impl BitmapFrameAllocator {
    pub const fn new_uninit() -> BitmapFrameAllocator {
        BitmapFrameAllocator {
            bitmap: 0 as *mut u8,
            free_pages: 0,
            used_pages: 0,
            number_of_pages: 0,
            next_free: 0,
            first_usable_address: 0,
        }
    }

    pub fn new() -> BitmapFrameAllocator {
        let available_pages = unsafe { mem::FREE_MEMORY / 0x1000 };
        let mut new = BitmapFrameAllocator {
            bitmap: 0 as *mut u8,
            free_pages: available_pages,
            used_pages: 0,
            number_of_pages: available_pages,
            next_free: 0,
            first_usable_address: 0,
        };
        new.allocate_bitmap(available_pages);
        new
    }

    fn allocate_bitmap(&mut self, size: usize) {
        let bytes = (size / 8) as u64;
        let frames_needed = (bytes / 0x1000 + 1) as u64;

        let regions = unsafe { mem::BOOT_MEMORY_MAP.unwrap().iter() };
        let usable_regions = regions
        .filter(|r| r.kind == boot_info::MemoryRegionKind::Usable);

        let mut alloc_first_phys = 0;
        let mut alloc_last_phys = 0;
        let mut found_mem = false;

        for reg in usable_regions {
            if reg.end - reg.start >= frames_needed {
                alloc_first_phys = reg.start;
                alloc_last_phys = ((alloc_first_phys + bytes) / 0x1000 + 1) * 0x1000;
                found_mem = true;
                break;
            }
        }
        if !found_mem {
            panic!("NOT_ENOUGH_MEMORY");
        }
        //serial_println!("Bitmap size: {}\nStart address: {:x}\nEnd address: {:x}", alloc_last_phys - alloc_first_phys, alloc_first_phys, alloc_last_phys);
        // clear bitmap memory
        let (alloc_first_virt, alloc_last_virt) = (alloc_first_phys + unsafe { mem::PHYSICAL_MEMORY_OFFSET }, alloc_last_phys + unsafe { mem::PHYSICAL_MEMORY_OFFSET });
        for addr in alloc_first_virt..alloc_last_virt {
            unsafe { *(addr as *mut u8) = 0; }
        }
        self.bitmap = alloc_first_virt as *mut u8;
        // reserve bitmap frames
        let bit_index_first = Self::address_to_bit_index(alloc_first_phys);
        let bit_index_last = Self::address_to_bit_index(alloc_last_phys);
        for bit in bit_index_first..bit_index_last {
            self.reserve_frame(bit);
        }
        self.first_usable_address = alloc_last_phys + 0x1000;
        self.next_free = bit_index_last + 1;
        // reserve reserved frames
        let _ = unsafe { mem::BOOT_MEMORY_MAP.unwrap().iter() }
        .filter(|r| r.kind != boot_info::MemoryRegionKind::Usable)
        .map(|reg| reg.start..reg.end)
        .flat_map(|reg| reg.step_by(0x1000))
        .map(|reg| self.reserve_frame(reg as usize));
    }

    #[inline(always)]
    pub fn bitmap_get(&self, index: usize) -> bool {
        unsafe {
            (((*self.bitmap.offset((index / 8) as isize)) << (index % 8)) >> 7) & 1 == 1
        }
    }

    #[inline(always)]
    pub fn bitmap_set(&mut self, index: usize) {
        unsafe {
            (*(self.bitmap.offset((index / 8) as isize))) |= 0b10000000 >> (index % 8)
        }
    }

    #[inline(always)]
    pub fn bitmap_clear(&mut self, index: usize) {
        unsafe {
            (*(self.bitmap.offset((index / 8) as isize))) &= !(0b10000000 >> (index % 8));
        }
    }

    pub fn lock_all(&mut self) {
        self.next_free += 1;
        self.first_usable_address = Self::bit_index_to_address(self.next_free);
        self.next_free += 1;
    }

    #[inline(always)]
    fn address_to_bit_index(address: u64) -> usize {
        (address / 0x1000) as usize
    }

    #[inline(always)]
    fn bit_index_to_address(index: usize) -> u64 {
        (index * 0x1000) as u64
    }

    pub fn reserve_frame(&mut self, frame_index: usize) {
        self.bitmap_set(frame_index);
        self.free_pages -= 1;
        self.used_pages += 1;
    }

    pub fn allocate_frame(&mut self) -> u64 {
        let frame = Self::bit_index_to_address(self.next_free);
        self.bitmap_set(self.next_free);
        self.free_pages -= 1;
        self.used_pages += 1;
        while {
            self.next_free += 1;
            if self.next_free >= self.number_of_pages {
                self.next_free = Self::address_to_bit_index(self.first_usable_address);
            }
            self.bitmap_get(self.next_free)
        } {}
        frame
    }

    pub fn free_frame(&mut self, frame: u64) {
        if frame <= self.first_usable_address {
            return;
        }
        let bit = Self::address_to_bit_index(frame);
        if !self.bitmap_get(bit) {
            return;
        }
        self.bitmap_clear(bit);
        self.free_pages += 1;
        self.used_pages -= 1;
        if self.next_free > bit {
            self.next_free = bit;
        }
    }

    pub fn get_free_pages(&self) -> usize {
        self.free_pages
    }

    pub fn get_used_pages(&self) -> usize {
        self.used_pages
    }

    pub fn get_pages(&self) -> usize {
        self.free_pages + self.used_pages
    }
}