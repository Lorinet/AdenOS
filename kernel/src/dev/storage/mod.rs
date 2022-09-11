mod ahci;
pub use ahci::AHCI;
pub use ahci::drive::AHCIDrive;

pub trait Drive {
    fn capacity(&mut self) -> usize;
}