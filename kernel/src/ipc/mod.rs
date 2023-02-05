use crate::*;
use crate::exec::scheduler;
use crate::namespace::{Resource, ResourceType, Handle};
use infinity::io::*;
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use infinity::ipc::Endpoint;
use modular_bitfield::private::checks::True;
use ringbuffer::{AllocRingBuffer, RingBufferWrite, RingBufferRead, RingBuffer, RingBufferExt};
use alloc::sync::Arc;
use spin::Mutex;
use core::cell::RefCell;
use alloc::string::{String, ToString};

pub mod lpc;

pub struct Message {
    pub from: u32,
    pub bytes: Vec<u8>,
}

impl Message {
    pub fn new(bytes: &[u8]) -> Message {
        Message {
            from: scheduler::current_process(),
            bytes: bytes.to_vec(),
        }
    }
}

pub struct MessageQueue {
    name: String,
    owner: u32,
    endpoint: Endpoint,
    queue: Arc<Mutex<AllocRingBuffer<Message>>>,
}

impl MessageQueue {
    pub fn new(name: String, owner: u32, endpoint: Endpoint, capacity: usize) -> MessageQueue {
        MessageQueue {
            name,
            owner,
            endpoint,
            queue: Arc::new(Mutex::new(AllocRingBuffer::with_capacity(capacity))),
        }
    }

    pub fn send(&self, message: Message) {
        self.queue.lock().push(message);
    }

    pub fn receive(&self) -> Result<Message, Error> {
        if scheduler::current_process() != self.owner {
            return Err(Error::Permissions);
        }
        if let Some(mesg) = self.queue.lock().dequeue() {
            Ok(mesg)
        } else {
            Err(Error::NoData)
        }
    }

    pub fn peek_len(&self) -> Result<usize, Error> {
        if scheduler::current_process() != self.owner {
            return Err(Error::Permissions);
        }
        if let Some(mesg) = self.queue.lock().peek() {
            Ok(mesg.bytes.len())
        } else {
            Err(Error::NoData)
        }
    }

    pub fn available(&self) -> usize {
        self.queue.lock().len()
    }
}

impl Read for MessageQueue {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let pk = self.peek_len()?;
        if pk > buf.len() {
            Err(Error::BufferTooSmall)
        } else {
            buf[..pk].copy_from_slice(self.receive()?.bytes.as_slice());
            Ok(buf.len())
        }
    }
}

impl Write for MessageQueue {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.send(Message::new(buf));
        Ok(buf.len())
    }
}

impl Resource for MessageQueue {
    fn is_open(&self) -> bool {
        if let Endpoint::Process(pid) = self.endpoint {
            if scheduler::current_process() != pid {
                // reserved for a single process
                return true;
            }
        }
        false
    }

    fn set_open_state(&mut self, open: bool) {
        if scheduler::current_process() == self.owner {
            if !open {
                // destroy channel if owner drops handle
                namespace::drop_resource_parts(self.resource_path());
                return;
            }
        }
    }

    fn resource_path(&self) -> Vec<alloc::string::String> {
        vec![String::from("Processes"), self.owner.to_string(), String::from("MessageQueues"), self.name.clone()]
    }

    fn unwrap(&mut self) -> namespace::ResourceType {
        ResourceType::MessageQueue(self)
    }
}