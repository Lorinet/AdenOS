use crate::*;
use bitflags::bitflags;

pub trait Handle {
    fn id(&self) -> usize;
    fn owner(&self) -> usize;
}

bitflags! {
    pub struct Permissions: usize {
        const READ = 0b0000000000000001;
        const WRITE = 0b0000000000000010;
    }
}