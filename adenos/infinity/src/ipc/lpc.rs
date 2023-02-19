use crate::*;
use os::*;
use alloc::collections::BTreeMap;

pub struct LPCServer {
    connection_handle: Option<u32>,
    client_channels: BTreeMap<u32, u32>,
    functions: BTreeMap<u32, fn(*const u8) -> i64>,
}

impl LPCServer {
    pub fn new() -> LPCServer {
        LPCServer {
            connection_handle: None,
            client_channels: BTreeMap::new(),
            functions: BTreeMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        let _ = self.connection_handle.insert(os::create_message_queue("lpc_connect", 0)?);
        Ok(())
    }

    pub fn register_function(&mut self, id: u32, function: fn(*const u8) -> i64) {
        self.functions.insert(id, function);
    }
}