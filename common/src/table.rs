use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Table<T> {
    pub items: Vec<T>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Table { items: vec![] }
    }
}
impl<T: Eq + Clone> Table<T> {
    pub fn insert(&mut self, item: T) {
        if !self.items.contains(&item) {
            self.items.push(item)
        }
    }
    pub fn get_index(&self, item: T) -> Option<usize> {
        self.items.iter().enumerate().position(|x| x.1 == &item)
    }
    pub fn has(&self, item: &T) -> bool {
        self.items.contains(item)
    }
    pub fn retreive(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn clear(&mut self) {
        self.items.clear()
    }
    pub fn extend(&mut self, othertable: Table<T>) {
        self.items.extend(othertable.items);
    }
    pub fn remove(&mut self, item: T) {
        self.items.remove(self.get_index(item).unwrap());
    }
}

impl<T: std::fmt::Debug> fmt::Debug for Table<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table").field("Items", &self.items).finish()
    }
}
