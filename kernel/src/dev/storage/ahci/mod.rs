use crate::{*, dev::hal::pci};

pub struct AHCI {
    pci_device: &'static pci::PCIDeviceHeader,
}

impl AHCI {
    pub fn new(pci_device: &'static pci::PCIDeviceHeader) -> AHCI {
        AHCI {
            pci_device
        }
    }
}

impl dev::Device for AHCI {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn device_name(&self) -> &str {
        "Storage/AHCI"
    }
}