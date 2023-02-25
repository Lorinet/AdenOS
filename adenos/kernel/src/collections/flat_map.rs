use crate::*;
use core::ops::*;
use alloc::{vec, vec::Vec};

#[derive(Debug)]
pub struct FlatMap<V> {
    memory: Vec<Option<V>>,
    first_free: Option<u32>,
}

impl<V> FlatMap<V> {
    pub const fn new() -> FlatMap<V> {
        FlatMap {
            memory: Vec::new(),
            first_free: None,
        }
    }

    pub fn insert(&mut self, key: u32, value: V) -> &mut V {
        if self.memory.len() <= key as usize {
            for _ in 0..=key as usize - self.memory.len() {
                self.memory.push(None);
            }
        }
        self.memory[key as usize].insert(value)
    }

    pub fn remove(&mut self, key: u32) -> Option<V> {
        if self.memory.len() > key as usize {
            if if let Some(n) = self.first_free {
                if n > key {
                    true
                } else {
                    false
                }
            } else {
                true
            } {
                // if first empty entry is after what we freed, replace index
                let _ = self.first_free.insert(key);
            }
            self.memory[key as usize].take()
        } else {
            None
        }
    }

    pub fn get(&self, key: u32) -> Option<&V> {
        if self.memory.len() > key as usize {
            self.memory[key as usize].as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: u32) -> Option<&mut V> {
        if self.memory.len() > key as usize {
            self.memory[key as usize].as_mut()
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.memory.clear();
    }

    pub fn len(&self) -> usize {
        self.memory.len()
    }

    pub fn push(&mut self, value: V) {
        self.memory.push(Some(value))
    }

    pub fn insert_where_you_can(&mut self, value: V) -> u32 {
        if let Some(idx) = self.first_free {
            self.insert(idx, value);
            for n in idx as usize + 1..self.memory.len() {
                if let None = self.memory[n] {
                    // if we find an empty spot...
                    let _ = self.first_free.insert(n as u32);
                    return idx;
                }
            }
            // if we don't, we won't
            self.first_free.take();
            return idx;
        } else {
            // if everything full, just append
            self.push(value);
            return self.memory.len() as u32 - 1;
        }
    }
}

impl<V> Index<u32> for FlatMap<V> {
    type Output = V;
    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<V> IndexMut<u32> for FlatMap<V> {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).unwrap()
    }
}