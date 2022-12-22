use core::fmt::Display;

use alloc::{vec, vec::Vec};
use crate::*;
pub struct Tree<K, V>
where
    K: PartialEq + Clone + Display,
{
    subtrees: Vec<Tree<K, V>>,
    key: K,
    value: Option<V>,
}

impl<K, V> Tree<K, V>
where
    K: PartialEq + Clone + Display,
{
    pub const fn new(root_key: K, root_value: Option<V>) -> Tree<K, V> {
        Tree {
            subtrees: Vec::new(),
            key: root_key,
            value: root_value,
        }
    }

    pub fn get_subtree(&mut self, key: K) -> Option<&mut Self> {
        for tree in &mut self.subtrees {
            if tree.key == key {
                return Some(tree);
            }
        }
        None
    }

    pub fn get_node_by_path(&mut self, path: Vec<K>) -> Option<&mut Self> {
        let mut subtree = Some(self);
        for node in path {
            subtree = subtree.unwrap().get_subtree(node);
            if let None = subtree {
                return None;
            }
        }
        subtree
    }

    pub fn insert_node_by_path(&mut self, path: Vec<K>, value: Option<V>) {
        let mut subtree = Some(unsafe { (self as *const _ as usize as *mut Tree<K, V>).as_mut().unwrap() });
        for node in &path[..path.len() - 1] {
            if let Some(subtree) = &mut subtree {
                if let None = subtree.get_subtree(node.clone()) {
                    subtree.insert_subtree(node.clone(), None);
                }
            }
            subtree = subtree.unwrap().get_subtree(node.clone());
        }
        if let Some(subtree) = subtree {
            subtree.insert_subtree(path.iter().last().cloned().unwrap(), value);
        }
    }

    pub fn value(&mut self) -> Option<&mut V> {
        self.value.as_mut()
    }

    pub fn insert_subtree(&mut self, key: K, value: Option<V>) {
        self.subtrees.push(Tree::new(key.clone(), value));
    }

    pub fn iter_mut_bf(&mut self) -> IterMutBF<K, V> {
        IterMutBF {
            stack: vec![self],
            next_stack: vec![],
        }
    }
}

pub struct IterMutBF<'a, K, V>
where
    K: PartialEq + Clone + Display,
{
    stack: Vec<&'a mut Tree<K, V>>,
    next_stack: Vec<&'a mut Tree<K, V>>,
}

impl<'a, K, V> Iterator for IterMutBF<'a, K, V>
where
    K: PartialEq + Clone + Display,
{
    type Item = (&'a mut K, Option<&'a mut V>);

    fn next(&mut self) -> Option<Self::Item> {
        let tree = if let Some(tree) = self.stack.pop() {
            tree
        } else if let Some(tree) = self.next_stack.pop() {
            core::mem::swap(&mut self.stack, &mut self.next_stack);
            tree
        } else {
            return None;
        };

        self.next_stack.extend(tree.subtrees.iter_mut());


        Some((&mut tree.key, tree.value.as_mut()))
    }
}
