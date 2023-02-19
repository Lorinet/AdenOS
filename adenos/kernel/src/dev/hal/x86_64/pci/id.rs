use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref CLASSES: BTreeMap<u8, &'static str> = BTreeMap::from([
        (0x00, "Unclassified Device"),
        (0x01, "Mass storage controller"),
        (0x02, "Network controller"),
        (0x03, "Display controller"),
        (0x04, "Multimedia controller"),
        (0x05, "Memory controller"),
        (0x06, "Bridge"),
        (0x07, "Communication controller"),
        (0x08, "Generic system peripheral"),
        (0x09, "Input device controller"),
        (0x0A, "Docking station"),
        (0x0B, "Processor"),
        (0x0C, "Serial bus controller"),
        (0x0D, "Wireless controller"),
        (0x0E, "Intelligent controller"),
        (0x0F, "Satellite communications controller"),
        (0x10, "Encryption controller"),
        (0x11, "Signal processing controller"),
        (0x12, "Processing accelerator"),
        (0x13, "Non-essential instrumentation"),
        (0x40, "Coprocessor"),
        (0xFF, "Unassigned class"),
    ]);

    pub static ref SUBCLASSES: BTreeMap<u8, BTreeMap<u8, &'static str>> = BTreeMap::from([
        (0x00, BTreeMap::from([
            (0x00, "Non-VGA unclassified device"),
            (0x01, "VGA compatible unclassified device"),
            (0x05, "Image coprocessor"),
        ])),
        (0x01, BTreeMap::from([
            (0x00, "SCSI storage controller"),
            (0x01, "IDE interface"),
            (0x02, "Floppy disk controller"),
            (0x03, "IPI bus controller"),
            (0x04, "RAID bus controller"),
            (0x05, "ATA controller"),
            (0x06, "SATA controller"),
            (0x07, "Serial Attached SCSI controller"),
            (0x08, "Non-volatile memory controller"),
            (0x80, "Mass storage controller"),
        ])),
        (0x02, BTreeMap::from([
            (0x00, "Ethernet controller"),
            (0x01, "Token ring network controller"),
            (0x02, "FDDI network controller"),
            (0x03, "ATM network controller"),
            (0x04, "ISDN controller"),
            (0x05, "WorldFip controller"),
            (0x06, "PICMG controller"),
            (0x07, "Infiniband controller"),
            (0x08, "Fabric controller"),
            (0x80, "Network controller"),
        ])),
        (0x03, BTreeMap::from([
            (0x00, "VGA compatible controller"),
            (0x01, "XGA compatible controller"),
            (0x02, "3D controller"),
            (0x80, "Display controller"),
        ])),
        (0x04, BTreeMap::from([
            (0x00, "Multimedia video controller"),
            (0x01, "Multimedia audio controller"),
            (0x02, "Computer telephony device"),
            (0x03, "Audio device"),
            (0x80, "Multimedia controller"),
        ])),
        (0x05, BTreeMap::from([
            (0x00, "RAM memory"),
            (0x01, "Flash memory"),
            (0x02, "CXL memory"),
            (0x80, "Memory controller"),
        ])),
        (0x06, BTreeMap::from([
            (0x00, "Host bridge"),
            (0x01, "ISA bridge"),
            (0x02, "EISA bridge"),
            (0x03, "MicroChannel bridge"),
            (0x04, "PCI bridge"),
            (0x05, "PCMCIA bridge"),
            (0x06, "NuBus bridge"),
            (0x07, "CardBus bridge"),
            (0x08, "RACEWay bridge"),
            (0x09, "Semi-transparent PCI-to-PCI bridge"),
            (0x0A, "InfiniBand-to-PCI bridge"),
            (0x80, "Bridge"),
        ])),
        (0x07, BTreeMap::from([
            (0x00, "Serial controller"),
            (0x01, "Parallel controller"),
            (0x02, "Multiport serial controller"),
            (0x03, "Modem"),
            (0x04, "GPIB controller"),
            (0x05, "Smart card controller"),
            (0x80, "Communication controller"),
        ])),
        (0x08, BTreeMap::from([
            (0x00, "PIC"),
            (0x01, "DMA controller"),
            (0x02, "Timer"),
            (0x03, "RTC"),
            (0x04, "PCI Hot-plug controller"),
            (0x05, "SD host controller"),
            (0x06, "IOMMU"),
            (0x80, "System peripheral"),
            (0x99, "Timing card"),
        ])),
        (0x09, BTreeMap::from([
            (0x00, "Keyboard controller"),
            (0x01, "Digitizer pen"),
            (0x02, "Mouse controller"),
            (0x03, "Scanner controller"),
            (0x04, "Gameport controller"),
            (0x80, "Input device controller"),
        ])),
        (0x0A, BTreeMap::from([
            (0x00, "Generic docking station"),
            (0x80, "Docking station"),
        ])),
        (0x0B, BTreeMap::from([
            (0x00, "386"),
            (0x01, "486"),
            (0x02, "Pentium"),
            (0x03, ""),
            (0x04, ""),
            (0x10, "Alpha"),
            (0x20, "PowerPC"),
            (0x30, "MIPS"),
            (0x40, "Co-processor"),
        ])),
        (0x0C, BTreeMap::from([
            (0x00, "FireWire"),
            (0x01, "ACCESS bus"),
            (0x02, "SSA"),
            (0x03, "USB controller"),
            (0x04, "Fibre Channel"),
            (0x05, "SMBus"),
            (0x06, "InfiniBand"),
            (0x07, "IPMI interface"),
            (0x08, "SERCOS interface"),
            (0x09, "CAN bus"),
            (0x80, "Serial bus controller"),
        ])),
        (0x0D, BTreeMap::from([
            (0x00, "IRDA controller"),
            (0x01, "Consumer IR controller"),
            (0x02, "RF controller"),
            (0x03, "Bluetooth"),
            (0x04, "Broadband"),
            (0x05, "802.1a controller"),
            (0x06, "802.1b controller"),
            (0x80, "Wireless controller"),
        ])),
        (0x0E, BTreeMap::from([
            (0x00, "I2O"),
        ])),
        (0x0F, BTreeMap::from([
            (0x01, "Satellite TV controller"),
            (0x02, "Satellite audio communication controller"),
            (0x03, "Satellite voice communication controller"),
            (0x04, "Satellite data communication controller"),
        ])),
        (0x10, BTreeMap::from([
            (0x00, "Network and computing encryption device"),
            (0x10, "Entertainment encryption device"),
            (0x80, "Encryption controller"),
        ])),
        (0x11, BTreeMap::from([
            (0x00, "DPIO module"),
            (0x01, "Performance counters"),
            (0x10, "Communication synchronizer"),
            (0x20, "Signal processing management"),
            (0x80, "Signal processing controller"),
        ])),
        (0x12, BTreeMap::from([
            (0x00, "Processing accelerator"),
            (0x01, "SNIA Smart Data Accelerator Interface (SDXI) controller"),
        ])),
        (0x13, BTreeMap::new()),
        (0x40, BTreeMap::new()),
        (0xFF, BTreeMap::new()),
    ]);

    pub static ref VENDOR_NAMES: BTreeMap<u16, &'static str> = BTreeMap::from([
        (0x8086, "Intel"),
        (0x1234, "QEMU"),
    ]);
}

pub fn get_class_name(class: u8) -> &'static str {
    CLASSES.get(&class).or(Some(&"Unknown")).unwrap()
}

pub fn get_subclass_name(class: u8, subclass: u8) -> &'static str {
    match SUBCLASSES.get(&class) {
        Some(map) => map.get(&subclass).or(Some(&"Unknown")).unwrap(),
        None => "Unknown",
    }
}

pub fn get_vendor_name(vendor: u16) -> &'static str {
    VENDOR_NAMES.get(&vendor).or(Some(&"Unknown")).unwrap()
}