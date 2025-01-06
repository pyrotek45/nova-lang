use std::{borrow::Borrow, fmt};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Table<T> {
    pub items: Vec<T>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Table { items: vec![] }
    }
}
impl<T> Table<T> {
    pub fn insert(&mut self, item: T)
    where
        T: PartialEq,
    {
        if self.has(&item) {
            return;
        }
        self.items.push(item)
    }
    pub fn get_index<K>(&self, item: &K) -> Option<usize>
    where
        T: Borrow<K>,
        K: PartialEq + ?Sized,
    {
        self.items.iter().position(|x| x.borrow() == item.borrow())
    }
    pub fn has<K>(&self, item: &K) -> bool
    where
        T: Borrow<K>,
        K: PartialEq,
    {
        self.items.iter().any(|x| x.borrow() == item)
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
    pub fn remove<K>(&mut self, item: &K)
    where
        T: Borrow<K>,
        K: PartialEq,
    {
        self.items.remove(self.get_index(item).unwrap());
    }
}

impl<T: std::fmt::Debug> fmt::Debug for Table<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Table").field("Items", &self.items).finish()
    }
}
