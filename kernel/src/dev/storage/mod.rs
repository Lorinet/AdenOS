mod ahci;
pub use ahci::AHCI;
pub use ahci::drive::AHCIDrive;

mod nvme;
use alloc::vec::Vec;
pub use nvme::NVME;
pub use nvme::drive::NVMEDrive;

use crate::dev::*;

pub trait Drive: Device + RandomRead + RandomWrite {
    fn capacity(&mut self) -> usize;
    fn sector_size(&self) -> usize;
    fn read_sector(&mut self, sector: usize) -> Result<Vec<u8>, Error>;
    fn read_sectors(&mut self, start_sector: usize, count: usize) -> Result<Vec<u8>, Error>;
    fn write_sector(&mut self, sector: usize, buf: &mut [u8]) -> Result<(), Error>;
    fn write_sectors(&mut self, start_sector: usize, buf: &mut [u8]) -> Result<(), Error>;
}
