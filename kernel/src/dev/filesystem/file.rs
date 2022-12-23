use crate::*;
use handle::Handle;
use alloc::string::String;

pub struct FileHandle {
    id: usize,
    owner: usize,
    path: String,
}

impl FileHandle {
    pub fn new(id: usize, owner: usize, path: String) -> FileHandle {
        FileHandle {
            id,
            owner,
            path,
        }
    }
}

impl Handle for FileHandle {
    fn id(&self) -> usize {
        self.id
    }

    fn owner(&self) -> usize {
        self.owner
    }
}