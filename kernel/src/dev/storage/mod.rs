mod ahci;
pub use ahci::AHCI;
pub use ahci::drive::AHCIDrive;

mod nvme;
pub use nvme::NVME;
pub use nvme::drive::NVMEDrive;

pub trait Drive {
    fn capacity(&mut self) -> usize;
}