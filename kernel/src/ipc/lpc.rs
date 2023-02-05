use crate::{*, namespace::Handle};
use alloc::collections::BTreeMap;

pub struct SyncLPCServer {
    connection_handle: Handle,
    client_channels: BTreeMap<u32, Handle>,

}