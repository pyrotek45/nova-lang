use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug)]
struct TaggedItems<T, Tag>
where
    T: Eq + Hash + Clone,
    Tag: Eq + Hash + Clone,
{
    items: Vec<T>,                         // Original items
    unique_keys: HashMap<(T, Tag), usize>, // Map from (item, tag) → index
    tag_map: HashMap<Tag, HashSet<usize>>, // Tag → indices
}

impl<T, Tag> TaggedItems<T, Tag>
where
    T: Eq + Hash + Clone,
    Tag: Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            items: Vec::new(),
            unique_keys: HashMap::new(),
            tag_map: HashMap::new(),
        }
    }

    fn insert(&mut self, item: T, tag: Tag) -> usize {
        let key = (item.clone(), tag.clone());

        if let Some(&index) = self.unique_keys.get(&key) {
            return index;
        }

        let index = self.items.len();
        self.items.push(item.clone());
        self.unique_keys.insert(key, index);
        self.tag_map.entry(tag).or_default().insert(index);
        index
    }

    fn get_indices_by_tag(&self, tag: &Tag) -> Option<Vec<usize>> {
        self.tag_map.get(tag).map(|set| set.iter().copied().collect())
    }

    fn get_item_at_index(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    fn all_items(&self) -> &Vec<T> {
        &self.items
    }
}
