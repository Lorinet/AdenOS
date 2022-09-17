use core::mem::size_of;
use crate::*;
#[derive(Debug)]
pub struct NVMEQueue {
    mbar: *mut u8,
    doorbell_stride: usize,
    queue_id: usize,
    submission_queue: *mut super::SubmissionQueueEntry,
    completion_queue: *mut super::CompletionQueueEntry,
    submission_queue_size: isize,
    completion_queue_size: isize,
    submission_head: isize,
    submission_tail: isize,
    completion_head: isize,
}

impl NVMEQueue {
    pub fn new_uninit() -> NVMEQueue {
        NVMEQueue {
            mbar: 0 as *mut u8,
            doorbell_stride: 0,
            queue_id: 0,
            submission_queue: 0 as *mut super::SubmissionQueueEntry,
            completion_queue: 0 as *mut super::CompletionQueueEntry,
            submission_queue_size: 0,
            completion_queue_size: 0,
            submission_head: 0,
            submission_tail: 0,
            completion_head: 0,
        }
    }

    pub fn new(mbar: *mut super::ControlRegisters, doorbell_stride: usize, queue_id: usize, submission_queue: *mut super::SubmissionQueueEntry, completion_queue: *mut super::CompletionQueueEntry, submission_queue_size: isize, completion_queue_size: isize) -> NVMEQueue {
        NVMEQueue {
            mbar: mbar as *mut u8,
            doorbell_stride,
            queue_id,
            submission_queue,
            completion_queue,
            submission_queue_size,
            completion_queue_size,
            submission_head: 0,
            submission_tail: submission_queue_size - 1,
            completion_head: 0,
        }
    }

    pub fn setup(&mut self) {
        self.submission_doorbell(0);
        self.completion_doorbell(0);
    }

    pub fn submission_doorbell(&mut self, slot: u16) {
        let offset = (self.mbar as usize) + 0x1000 + ((2 * self.queue_id) * self.doorbell_stride);
        serial_println!("{:x} {:x}", self.mbar as usize, offset);
        let pointer = offset as *mut u16;
        unsafe {
            *pointer = slot;
        }
    }

    pub fn completion_doorbell(&mut self, slot: u16) {
        let offset = (self.mbar as usize) + 0x1000 + (((2 * self.queue_id) + 1) * self.doorbell_stride);
        let pointer = offset as *mut u16;
        unsafe {
            *pointer = slot + 1;
        }
    }

    pub fn get_completion_doorbell(&self) -> u16 {
        let offset = (self.mbar as usize) + 0x1000 + (((2 * self.queue_id) + 1) * self.doorbell_stride);
        let pointer = offset as *const u16;
        unsafe { *pointer }
    }

    pub fn get_submission_doorbell(&self) -> u16 {
        let offset = (self.mbar as usize) + 0x1000 + ((2 * self.queue_id) * self.doorbell_stride);
        let pointer = offset as *const u16;
        unsafe { *pointer }
    }

    pub fn print_all(&self) {
        for i in 0..8 {
            serial_println!("{:#x?}", unsafe { *self.completion_queue.offset(i) });
        }
    }

    pub fn wait_until_completion(&mut self) {

    }

    pub fn enqueue(&mut self, entry: super::SubmissionQueueEntry) {
        let size = size_of::<super::SubmissionQueueEntry>();
        unsafe {
            *self.submission_queue.offset(self.submission_tail) = entry;
            self.submission_doorbell(self.submission_tail as u16);
        }
        self.submission_tail += 1;
        if self.submission_tail >= self.submission_queue_size {
            self.submission_tail = 0;
        }
    }

    pub fn dequeue(&mut self) -> super::CompletionQueueEntry {
        let out = unsafe { *self.completion_queue.offset(self.completion_head) };
        self.completion_head -= 1;
        if self.completion_head < 0 {
            self.completion_head = self.completion_queue_size - 1;
        }
        self.completion_doorbell(self.completion_head as u16);
        self.submission_head = out.submission_queue_head_pointer() as isize;
        serial_println!("HEAD: {} TAIL: {}", self.submission_head, self.submission_tail);
        out
    }

    pub fn admin_identify(&mut self, prp: u64, namespace_id: u32, controller_id: u16, controller_namespace_structure: u8, command_id: u16) {
        const fuse_normal: u8 = 0x00;
        const psdt_use_prp: u8 = 0x00;
        
        let command = super::SubmissionQueueEntry::new().with_command_id(command_id)
        .with_opcode(super::OpCode::Identify as u8).with_command_id(self.submission_tail as u16)
        .with_fused_operation(fuse_normal).with_prp_sgl(psdt_use_prp)
        .with_data_pointer_0(prp).with_nsid(namespace_id)
        .with_command_specific_0((((controller_id as u32) << 16) | controller_namespace_structure as u32) as u32);

        self.enqueue(command);
    }
    
}