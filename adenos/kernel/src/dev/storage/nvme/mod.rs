use crate::{*, dev::hal::{pci, mem}};
use {dev::*, namespace::*};
use alloc::{vec, vec::Vec, string::String};
use modular_bitfield::{bitfield, specifiers::*};

mod queue;
pub mod drive;

#[repr(u8)]
enum OpCode {
    CreateIOSubmissionQueue = 0x01,
    CreateIOCompletionQueue = 0x05,
    Identify = 0x06,
    NamespaceAttachment = 0x15,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Capabilities {
    maximum_queue_entries_supported: u16,
    contiguous_queues_required: bool,
    arbitration_mechanism_supported: B2,
    _reserved_0: B5,
    TO: u8,
    doorbell_stride: B4,
    nvm_subsystem_reset_supported: bool,
    command_sets_supported: u8,
    boot_partition_supported: bool,
    _reserved_1: B2,
    memory_page_size_minimum: B4,
    memory_page_size_maximum: B4,
    _reserved_2: u8,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct ControlRegisters {
    capabilities: u64,
    version: u32,
    interrupt_mask_set: u32,
    interrupt_mask_clear: u32,
    controller_configuration: u32,
    _reserved_0: u32,
    controller_status: u32,
    _reserved_1: u32,
    admin_queue_attributes: u32,
    admin_submission_queue: u64,
    admin_completion_queue: u64,
}

#[bitfield]
#[repr(u32)]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ControllerConfiguration {
    enabled: bool,
    _reserved_0: B3,
    command_set_selected: B3,
    memory_page_size: B4,
    arbitration_mechanism_supported: B3,
    shutdown_notification: B2,
    io_submission_queue_size: B4,
    io_completion_queue_size: B4,
    _reserved_1: u8,
}

#[bitfield]
#[repr(u32)]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ControllerStatus {
    ready: bool,
    controller_fatal_state: bool,
    shutdown_status: B2,
    nvm_subsystem_reset_occurred: bool,
    processing_paused: bool,
    _reserved: B26,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct SubmissionQueueEntry {
    opcode: u8,
    fused_operation: B2,
    _reserved_0: B4,
    prp_sgl: B2,
    command_id: u16,
    nsid: u32,
    _reserved_1: u64,
    metadata_pointer: u64,
    data_pointer_0: u64,
    data_pointer_1: u64,
    command_specific_0: u32,
    command_specific_1: u32,
    command_specific_2: u32,
    command_specific_3: u32,
    command_specific_4: u32,
    command_specific_5: u32,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct CompletionQueueEntry {
    command_specific: u32,
    _reserved: u32,
    submission_queue_head_pointer: u16,
    submission_queue_id: u16,
    command_id: u16,
    phase: bool,
    status: B15,
}


#[derive(Debug)]
pub struct NVME {
    pci_device_header: &'static pci::PCIHeaderType0,
    mbar: &'static mut ControlRegisters,
    doorbell_stride: usize,
    admin_queue: queue::NVMEQueue,
}

impl NVME {
    pub fn new(pci_device_header: &'static pci::PCIDeviceHeader) -> NVME {
        let head0: &pci::PCIHeaderType0 = pci_device_header.into();
        let mbar = pci::bar_to_struct_64::<ControlRegisters>(head0.bar_0 & 0xFFFFFFF0, head0.bar_1);
        
        let caps = pci::bar_to_struct_64::<Capabilities>(head0.bar_0 & 0xFFFFFFF0, head0.bar_1);
        let doorbell_stride = (4 << caps.doorbell_stride()) as usize; // a value of 0 is 4 bytes stride with no padding

        NVME {
            pci_device_header: head0,
            mbar,
            doorbell_stride,
            admin_queue: queue::NVMEQueue::new_uninit(),
        }
    }
}

impl Device for NVME {
    fn init_device(&mut self) -> Result<(), Error> {
        const reg_offset_controller_configuration: u64 = 0x14;
        const reg_offset_controller_status: u64 = 0x1C;
        const shutdown_status_normal: u8 = 0b00;
        const shutdown_status_completed: u8 = 0b10;
        const shutdown_notification_abrupt_shutdown: u8 = 0b10;

        let config = unsafe { (((self.mbar as *const _ as u64) + reg_offset_controller_configuration) as *mut ControllerConfiguration).as_mut().unwrap() };
        let status = unsafe { (((self.mbar as *const _ as u64) + reg_offset_controller_status) as *mut ControllerStatus).as_mut().unwrap() };
        //serial_println!("{:#x?}\n{:#x?}", config, status);

        if status.ready() || status.controller_fatal_state() {
            // shutdown controller
            if status.shutdown_status() == shutdown_status_normal || status.controller_fatal_state() {
                let set = config.clone().with_shutdown_notification(shutdown_notification_abrupt_shutdown);
                *config = set;
                for _ in 0..1000000 {}
                if status.shutdown_status() != shutdown_status_completed {
                    return Err(Error::InitFailure);
                }
            }

            // reset controller
            let set = config.clone().with_enabled(false);
            *config = set;
            for _ in 0..1000000 {}
            if status.ready() {
                for _ in 0..10000000 {}
                if status.ready() {
                    return Err(Error::InitFailure)
                }
            }
        }

        // set up admin queue
        unsafe {
            let frame_admin_sub_queue = mem::FRAME_ALLOCATOR.allocate_frame();
            let frame_admin_sub_queue_virt = frame_admin_sub_queue + mem::PHYSICAL_MEMORY_OFFSET;
            let clr = frame_admin_sub_queue_virt as *mut u8;
            for i in 0..0x1000 {
                *clr.offset(i) = 0;
            }

            let frame_admin_com_queue = mem::FRAME_ALLOCATOR.allocate_frame();
            let frame_admin_com_queue_virt = frame_admin_com_queue + mem::PHYSICAL_MEMORY_OFFSET;
            let clr = frame_admin_com_queue_virt as *mut u8;
            for i in 0..0x1000 {
                *clr.offset(i) = 0;
            }

            const admin_submission_queue_size: isize = 8;
            const admin_completion_queue_size: isize = 8;

            let admin_sub_queue = frame_admin_sub_queue_virt as *mut SubmissionQueueEntry;
            let admin_com_queue = frame_admin_com_queue_virt as *mut CompletionQueueEntry;
            self.admin_queue = queue::NVMEQueue::new(self.mbar as *mut ControlRegisters, self.doorbell_stride, 0, admin_sub_queue, admin_com_queue, admin_submission_queue_size, admin_completion_queue_size);
            
            let admin_sub_que_phys = frame_admin_sub_queue;
            let admin_com_que_phys = frame_admin_com_queue;
            let mut admin_queue_attrib = 0;
            admin_queue_attrib |= (0xfff & admin_completion_queue_size as u32) << 16;
            admin_queue_attrib |= 0xfff & admin_submission_queue_size as u32;

            let set = self.mbar.clone()
            .with_admin_submission_queue(admin_sub_que_phys)
            .with_admin_completion_queue(admin_com_que_phys)
            .with_admin_queue_attributes(admin_queue_attrib);
            *self.mbar = set;

            let admin_sub_doorbell_off = (self.mbar as *const _ as usize) + 0x1000;
            let admin_sub_doorbell_ptr = admin_sub_doorbell_off as *mut u16;
            *admin_sub_doorbell_ptr = 7;

            let admin_com_doorbell_off = (self.mbar as *const _ as usize) + 0x1000 + self.doorbell_stride;
            let admin_com_doorbell_ptr = admin_com_doorbell_off as *mut u16;
            *admin_com_doorbell_ptr = 0;
        }

        // configure and enable controller
        const arbitration_mechanism_round_robin: u8 = 0x00;
        const command_set_nvme: u8 = 0x00;
        const memory_page_size_4kb: u8 = 0x00;
        const shutdown_notification_none: u8 = 0x00;

        let set = config.clone()
        .with_arbitration_mechanism_supported(arbitration_mechanism_round_robin)
        .with_command_set_selected(command_set_nvme)
        .with_memory_page_size(memory_page_size_4kb)
        .with_shutdown_notification(shutdown_notification_none)
        .with_enabled(true);
        *config = set;

        for _ in 0..1000000 {}
        if !status.ready() {
            for _ in 0..10000000 {}
            if !status.ready() {
                return Err(Error::InitFailure)
            }
        }

        // attach all namespaces
        unsafe {
            let prp = mem::FRAME_ALLOCATOR.allocate_frame();
            self.admin_queue.admin_identify(prp, 0xffffffff, 0, 0x01, 0);
        }

        for _ in 0..10000000 {}

        //serial_println!("{:#x?}", self.admin_queue.dequeue());
        //self.admin_queue.print_all();
        serial_println!("{:#x}", self.pci_device_header.capabilities_pointer);
        let mut cap = pci::MSIXCapability::from_address(self.pci_device_header as *const _ as u64 + self.pci_device_header.capabilities_pointer as u64);
        while cap.capability_id() != 0x11 {
            cap = pci::MSIXCapability::from_address(self.pci_device_header as *const _ as u64 + cap.next_pointer() as u64);
        }
        cap.init(self.pci_device_header as *const _ as u64);
        //serial_println!("{:#x} {:#x} {:#x} {:#x} {} {} {:#x} {:#x} {:#x} {:#x}", cap.capability_id(), cap.next_pointer(), cap.table_size(), cap._reserved(), cap.function_mask(), cap.enable(), cap.bir(), cap.table_offset(), cap.pending_bit_bir(), cap.pending_bit_offset());

        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Storage"), String::from("NVME")]
    }

    fn unwrap(&mut self) -> DeviceClass {
        DeviceClass::Other
    }
}