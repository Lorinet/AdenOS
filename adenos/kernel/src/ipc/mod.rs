use crate::*;
use crate::exec::scheduler;
use dev::*;
use namespace::*;
use alloc::boxed::Box;
use alloc::{vec, vec::Vec};
use modular_bitfield::private::checks::True;
use ringbuffer::{AllocRingBuffer, RingBufferWrite, RingBufferRead, RingBuffer, RingBufferExt};
use alloc::sync::Arc;
use spin::Mutex;
use core::cell::RefCell;
use alloc::string::{String, ToString};

pub struct MessageQueue {
    owner: u32,
    endpoint: Endpoint,
    queue: Arc<Mutex<AllocRingBuffer<Message>>>,
}

impl MessageQueue {
    pub fn new(owner: u32, endpoint: Endpoint, capacity: usize) -> MessageQueue {
        MessageQueue {
            owner,
            endpoint,
            queue: Arc::new(Mutex::new(AllocRingBuffer::with_capacity(capacity))),
        }
    }
}

impl MessageTransport for MessageQueue {
    fn send(&self, message: Message) -> Result<(), Error> {
        if let Endpoint::Process(pid) = self.endpoint {
            if pid != message.from {
                return Err(Error::Permissions)
            }
        }
        self.queue.lock().push(message);
        Ok(())
    }

    fn receive(&self) -> Result<Message, Error> {
        panic!("Fuck you");
       /*if scheduler::current_thread() != self.owner {
            return Err(Error::Permissions);
        }
        if let Some(mesg) = self.queue.lock().dequeue() {
            Ok(mesg)
        } else {
            Err(Error::NoData)
        }*/
    }

    fn peek_len(&self) -> Result<usize, Error> {
        panic!("Fuck you");
        /*if scheduler::current_thread() != self.owner {
            return Err(Error::Permissions);
        }
        if let Some(mesg) = self.queue.lock().peek() {
            Ok(mesg.bytes.len())
        } else {
            Err(Error::NoData)
        }*/
    }

    fn available(&self) -> usize {
        self.queue.lock().len()
    }

    fn owner(&self) -> u32 {
        self.owner
    }

    fn endpoint(&self) -> Endpoint {
        self.endpoint
    }
}


pub trait MessageTransport {
    fn send(&self, message: Message) -> Result<(), Error>;
    fn receive(&self) -> Result<Message, Error>;
    fn peek_len(&self) -> Result<usize, Error>;
    fn available(&self) -> usize;
    fn owner(&self) -> u32;
    fn endpoint(&self) -> Endpoint;
}

pub struct Message {
    pub from: u32,
    pub bytes: Vec<u8>,
}

impl Message {
    pub fn new(bytes: &[u8]) -> Message {
        Message {
            from: syscall::_get_process_id() as u32,
            bytes: bytes.to_vec(),
        }
    }
}

pub struct MessageChannel {
    name: String,
    channel: Box<dyn MessageTransport>,
}

impl MessageChannel {
    pub fn new(name: String, channel: Box<dyn MessageTransport>) -> MessageChannel {
        MessageChannel {
            name,
            channel,
        }
    }
}


impl MessageTransport for MessageChannel {
    fn available(&self) -> usize {
        self.channel.available()
    }

    fn endpoint(&self) -> Endpoint {
        self.channel.endpoint()
    }

    fn owner(&self) -> u32 {
        self.channel.owner()
    }

    fn peek_len(&self) -> Result<usize, Error> {
        self.channel.peek_len()
    }

    fn send(&self, message: Message) -> Result<(), Error> {
        self.channel.send(message)
    }

    fn receive(&self) -> Result<Message, Error> {
        self.channel.receive()
    }
}

impl Read for MessageChannel {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let pk = self.channel.peek_len()?;
        if pk > buf.len() {
            Err(Error::BufferTooSmall)
        } else {
            buf[..pk].copy_from_slice(self.channel.receive()?.bytes.as_slice());
            Ok(buf.len())
        }
    }
}

impl Write for MessageChannel {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.channel.send(Message::new(buf));
        Ok(buf.len())
    }
}

impl Resource for MessageChannel {
    fn is_open(&self) -> bool {
        if let Endpoint::Process(pid) = self.channel.endpoint() {
            if syscall::_get_process_id() as u32 != pid {
                // reserved for a single process
                return true;
            }
        }
        false
    }

    fn set_open_state(&mut self, open: bool) {
        if syscall::_get_process_id() as u32 == self.channel.owner() {
            if !open {
                // destroy channel if owner drops handle
                namespace::drop_resource_parts(self.resource_path());
                return;
            }
        }
    }

    fn resource_path(&self) -> Vec<alloc::string::String> {
        vec![String::from("Processes"), self.channel.owner().to_string(), String::from("MessageChannels"), self.name.clone()]
    }

    fn unwrap(&mut self) -> namespace::ResourceType {
        ResourceType::MessageChannel(self)
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Endpoint {
    Any = 0,
    Process(u32),
}

impl Into<u32> for Endpoint {
    fn into(self) -> u32 {
        match self {
            Endpoint::Any => 0,
            Endpoint::Process(pid) => pid,
        }
    }
}

impl From<u32> for Endpoint {
    fn from(val: u32) -> Self {
        if val == 0 {
            Endpoint::Any
        } else {
            Endpoint::Process(val as u32)
        }
    }
}

pub struct BidirectionalChannel {
    receiver: u32,
    sender: u32,
}

impl BidirectionalChannel {
    pub fn new(receiver: u32, sender: u32) -> BidirectionalChannel {
        BidirectionalChannel {
            receiver,
            sender,
        }
    }
}

impl Read for BidirectionalChannel {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Error::from_code_to_usize(syscall::_read(self.receiver as usize, buf.as_mut_ptr(), buf.len()) as i64)
    }
}

impl Write for BidirectionalChannel {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        Error::from_code_to_usize(syscall::_write(self.sender as usize, buf.as_ptr(), buf.len()) as i64)
    }
}