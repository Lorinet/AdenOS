use crate::{*, dev::hal::{mem, pci::{self, PCIHeaderType0}}};
use core::{mem::size_of, fmt::Display};
use modular_bitfield::{bitfield, specifiers::{B5, B4, B16, B3, B1, B22, B9, B128}};

pub mod drive;

const HBA_PORT_DEVICE_PRESENT: u8 = 0x03;
const HBA_PORT_IPM_ACTIVE: u8 = 0x01;
const HBA_PORT_TASK_FILE_ERROR: u32 = 1 << 30;
const SATA_SIGNATURE_ATAPI: u32 = 0xEB140101;
const SATA_SIGNATURE_ATA: u32 = 0x00000101;
const SATA_SIGNATURE_SEMB: u32 = 0xC33C0101;
const SATA_SIGNATURE_PM: u32 = 0x96690101;

pub enum DiskIO {
    Read,
    Write,
}

#[derive(Copy, Clone, Debug)]
pub enum AHCIError {
    PortCommunicationError,
    DiskReadError,
    DiskWriteError,
}

impl Display for AHCIError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AHCI error: {:?}", self)
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ATAStatus {
    DeviceBusy = 0x80,
    DataTransferRequested = 0x08,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
enum HBACommands {
    ST = 0x0001,
    FRE = 0x0010,
    FR = 0x4000,
    CR = 0x8000,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ATACommands {
    ReadDMAEx = 0x25,
    WriteDMAEx = 0x35,
}

#[derive(Copy, Clone, Debug)]
pub enum PortType {
    None = 0,
    SATA = 1,
    SEMB = 2,
    PM = 3,
    SATAPI = 4,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum FISType {
    RegH2D = 0x27,
    RegD2H = 0x34,
    DMAAct = 0x39,
    DMASetup = 0x41,
    Data = 0x46,
    BIST = 0x58,
    PIOSetup = 0x5F,
    DevBits = 0xA1,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct HBAPort {
    pub command_list_base_address: u64,
    pub fis_base_address: u64,
    pub interrupt_status: u32,
    pub interrupt_enable: u32,
    pub command_status: u32,
    _reserved_0: u32,
    pub task_file_data: u32,
    pub signature: u32,
    pub sata_status: u32,
    pub sata_control: u32,
    pub sata_error: u32,
    pub sata_active: u32,
    pub command_issue: u32,
    pub sata_notification: u32,
    pub fis_based_switch_control: u32,
    _reserved_1: [u32; 11],
    _vendor_specific: [u32; 4],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct HBAMemory {
    pub host_capabilities: u32,
    pub global_host_control: u32,
    pub interrupt_status: u32,
    pub port_implemented: u32,
    pub version: u32,
    pub ccc_control: u32,
    pub ccc_ports: u32,
    pub enclosure_management_location: u32,
    pub enclosure_management_control: u32,
    pub host_capabilities_extended: u32,
    pub bios_os_handoff_control: u32,
    _reserved: [u8; 0xA0-0x2C],
    _vendor_specific: [u8; 0x100-0xA0],
    pub ports: [HBAPort; 32],
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct FISRegH2D {
    fis_type: u8,
    port_multiplier: B4,
    _reserved_0: B3,
    command_control: bool,
    command: u8,
    feature_low: u8,
    lba_0: u8,
    lba_1: u8,
    lba_2: u8,
    device_register: u8,
    lba_3: u8,
    lba_4: u8,
    lba_5: u8,
    feature_high: u8,
    count: u16,
    iso_command_completion: u8,
    control: u8,
    _reserved_1: u32,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct HBAPRDTEntry {
    data_base_address: u64,
    _reserved_0: u32,
    byte_count: B22,
    _reserved_1: B9,
    interrupt_on_completion: bool,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct HBACommandTable {
    command_fis: [u8; 64],
    atapi_command: [u8; 16],
    _reserved: [u8; 48],
    prdt_entry: [HBAPRDTEntry; 1],
}

#[derive(Copy, Clone, Debug)]
pub struct Port {
    pub hba_port: *mut HBAPort,
    pub port_type: PortType,
    pub port_number: usize,
}

impl Port {
    pub fn configure(&mut self) {
        unsafe {
            self.stop_commands();

            let command_list_base_new = mem::FRAME_ALLOCATOR.allocate_frame();
            (*self.hba_port).command_list_base_address = command_list_base_new;
            let clear = (command_list_base_new + mem::PHYSICAL_MEMORY_OFFSET) as *mut u8;
            for i in 0..0x1000 {
                *clear.offset(i) = 0;
            }

            let fis_base_new = mem::FRAME_ALLOCATOR.allocate_frame();
            (*self.hba_port).fis_base_address = fis_base_new;
            let clear = (fis_base_new + mem::PHYSICAL_MEMORY_OFFSET) as *mut u8;
            for i in 0..256 {
                *clear.offset(i) = 0;
            }

            let command_header = ((*self.hba_port).command_list_base_address + mem::PHYSICAL_MEMORY_OFFSET) as *mut HBACommandHeader;

            for i in 0..32 {
                let command_table_address = mem::FRAME_ALLOCATOR.allocate_frame();
                let address = command_table_address + (i << 8);
                (*command_header.offset(i as isize)).set_command_table_descriptor_base_address(address);
                let clear = (command_table_address + mem::PHYSICAL_MEMORY_OFFSET) as *mut u8;
                for i in 0..256 {
                    *clear.offset(i) = 0;
                }
            }

            self.start_commands();
        }
    }

    pub fn start_commands(&mut self) {
        unsafe {
            while (*self.hba_port).command_status & HBACommands::CR as u32 != 0 {}
            (*self.hba_port).command_status |= HBACommands::FRE as u32;
            (*self.hba_port).command_status |= HBACommands::ST as u32;
        }
    }

    pub fn stop_commands(&mut self) {
        unsafe {
            (*self.hba_port).command_status &= !(HBACommands::FRE as u32);
            (*self.hba_port).command_status &= !(HBACommands::ST as u32);
            while ((*self.hba_port).command_status & HBACommands::CR as u32 != 0) && ((*self.hba_port).command_status & HBACommands::FR as u32 != 0) {}
        }
    }

    pub fn diskio(&mut self, operation: DiskIO, sector: u64, count: u16, buffer: *mut u8) -> Result<(), AHCIError> {
        unsafe {
            let sector_l = sector as u32;
            let sector_h = (sector >> 32) as u32;
            (*self.hba_port).interrupt_status = u32::MAX;
            let cmd_header = (((*self.hba_port).command_list_base_address + mem::PHYSICAL_MEMORY_OFFSET) as *mut HBACommandHeader).as_mut().unwrap();
            cmd_header.set_command_fis_length((size_of::<FISRegH2D>() / size_of::<u32>()) as u8);
            cmd_header.set_write(match operation {
                DiskIO::Read => false,
                DiskIO::Write => true,
            });
            cmd_header.set_physical_region_descriptor_table_length(1);

            let command_table = ((cmd_header.command_table_descriptor_base_address() + mem::PHYSICAL_MEMORY_OFFSET) as *mut HBACommandTable).as_mut().unwrap();
            let clear = (cmd_header.command_table_descriptor_base_address() + mem::PHYSICAL_MEMORY_OFFSET) as *mut u8;
            for i in 0..(size_of::<HBACommandTable>() as isize + (cmd_header.physical_region_descriptor_table_length() - 1) as isize * size_of::<HBAPRDTEntry>() as isize) {
                *clear.offset(i) = 0;
            }

            command_table.prdt_entry[0].set_data_base_address(mem::page_mapper::translate_addr(buffer as usize).unwrap());
            command_table.prdt_entry[0].set_byte_count((count << 9) as u32 - 1);
            command_table.prdt_entry[0].set_interrupt_on_completion(true);

            let cmd_fis = (command_table.command_fis.as_mut_ptr() as *mut FISRegH2D).as_mut().unwrap();
            cmd_fis.set_fis_type(FISType::RegH2D as u8);
            cmd_fis.set_command_control(true);
            cmd_fis.set_command(match operation {
                DiskIO::Read => ATACommands::ReadDMAEx,
                DiskIO::Write => ATACommands::WriteDMAEx,
            } as u8);

            let lba_l = sector_l.to_le_bytes();
            let lba_h = sector_h.to_le_bytes();
            cmd_fis.set_lba_0(lba_l[0]);
            cmd_fis.set_lba_1(lba_l[1]);
            cmd_fis.set_lba_2(lba_l[2]);
            cmd_fis.set_lba_3(lba_h[0]);
            cmd_fis.set_lba_4(lba_h[1]);
            cmd_fis.set_lba_5(lba_h[2]);

            cmd_fis.set_device_register(1 << 6);
            cmd_fis.set_count(count & 0xFF);

            let mut spin = 0;
            while ((*self.hba_port).task_file_data & (ATAStatus::DeviceBusy as u8 | ATAStatus::DataTransferRequested as u8) as u32 != 0) && spin < 1000000 {
                spin += 1;
            }

            if spin == 1000000 {
                return Err(AHCIError::PortCommunicationError);
            }

            (*self.hba_port).command_issue = 1;

            loop {
                if (*self.hba_port).command_issue == 0 {
                    break;
                }
                if (*self.hba_port).interrupt_status & HBA_PORT_TASK_FILE_ERROR != 0 {
                    return Err(match operation {
                        DiskIO::Read => AHCIError::DiskReadError,
                        DiskIO::Write => AHCIError::DiskWriteError,
                    });
                }
            }

            Ok(())
        }
    }
}

#[bitfield]
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct HBACommandHeader {
    command_fis_length: B5,
    atapi: bool,
    write: bool,
    prefetchable: bool,
    reset: bool,
    bist: bool,
    clear_busy_upon_ok: bool,
    _reserved_0: bool,
    port_multiplier_port: B4,
    physical_region_descriptor_table_length: B16,
    physical_region_descriptor_byte_count_transferred: u32,
    command_table_descriptor_base_address: u64,
    _reserved_1: B128,
}

pub struct AHCI {
    pci_device_header: &'static pci::PCIDeviceHeader,
    abar: &'static HBAMemory,
    pub port_count: usize,
    pub ports: [Option<Port>; 32],
}

impl AHCI {
    pub fn new(pci_device: &'static pci::PCIDeviceHeader) -> AHCI {
        let head0: &PCIHeaderType0 = pci_device.into();
        let abar = unsafe { ((head0.bar_5 as u64 + mem::PHYSICAL_MEMORY_OFFSET) as *const HBAMemory).as_ref().unwrap() };
        AHCI {
            pci_device_header: pci_device,
            abar,
            port_count: 0,
            ports: [None; 32],
        }
    }

    fn get_port_type(port: &HBAPort) -> PortType {
        let sata_status = port.sata_status;
        let interface_power_management: u8 = ((sata_status >> 8) & 0b111) as u8;
        let device_detection: u8 = (sata_status & 0b111) as u8;
        
        if device_detection != HBA_PORT_DEVICE_PRESENT {
            return PortType::None;
        }

        if interface_power_management != HBA_PORT_IPM_ACTIVE {
            return PortType::None;
        }

        match port.signature {
            SATA_SIGNATURE_ATA => PortType::SATA,
            SATA_SIGNATURE_ATAPI => PortType::SATAPI,
            SATA_SIGNATURE_PM => PortType::PM,
            SATA_SIGNATURE_SEMB => PortType::SEMB,
            _ => PortType::None,
        }
    }

    fn probe_ports(&mut self) {
        let ports_implemented = self.abar.port_implemented;
        for p in 0..32 {
            if ports_implemented & (1 << p) != 0 {
                let port_type = Self::get_port_type(&self.abar.ports[p]);
                if let PortType::SATA | PortType::SATAPI = port_type {
                    self.ports[self.port_count] = Some(Port {
                        hba_port: &self.abar.ports[p] as *const HBAPort as *mut HBAPort,
                        port_type,
                        port_number: p,
                    });
                    self.port_count += 1;
                }
            }
        }
    }
}

impl dev::Device for AHCI {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        self.probe_ports();
        for mut port in self.ports {
            if let Some(port) = port.as_mut() {
                port.configure();
                let drv = devices::register_device(drive::AHCIDrive::new(port.port_number));
                devices::get_device::<drive::AHCIDrive>(drv).unwrap().init_device()?;
            }
        }
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn device_name(&self) -> &str {
        "Storage/AHCI"
    }
}