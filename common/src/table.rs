use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Table<T> {
    pub items: Vec<T>,
}

impl<T: Eq + Clone> Table<T> {
    #[inline(always)]
    pub fn insert(&mut self, item: T) {
        if !self.items.contains(&item) {
            self.items.push(item)
        }
    }
    #[inline(always)]
    pub fn get_index(&self, item: T) -> Option<usize> {
        self.items.iter().enumerate().position(|x| x.1 == &item)
    }
    #[inline(always)]
    pub fn has(&self, item: &T) -> bool {
        self.items.contains(item)
    }
    #[inline(always)]
    pub fn retreive(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.items.len()
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    #[inline(always)]
    pub fn clear(&mut self) {
        self.items.clear()
    }
    #[inline(always)]
    pub fn extend(&mut self, othertable: Table<T>) {
        for items in othertable.items.iter() {
            self.insert(items.clone())
        }
    }
}

impl<T: std::fmt::Debug> fmt::Debug for Table<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table").field("Items", &self.items).finish()
    }
}

pub fn new<T>() -> Table<T> {
    Table { items: vec![] }
}
