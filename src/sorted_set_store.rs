use std::collections::HashMap;

use crate::sorted_set::SkipList;

const TREE_HEIGHT: usize = 16;

pub struct SortedSetStore<K, V> {
    sets: HashMap<K, SkipList<K, V>>
}

impl<K, V> SortedSetStore<K, V> 
where 
    K: Ord + std::hash::Hash + Clone, 
    V: Ord + Clone
{
    pub fn new() -> Self {
        Self {
            sets: HashMap::new()
        }
    }
    
    pub fn zadd(&mut self, key: &K, member: K, score: V) -> bool {
        let list = self.sets
            .entry(key.clone())
            .or_insert_with(|| SkipList::new(TREE_HEIGHT));
        let is_new = list.get(&member).is_none();
        list.insert(member, score);
        is_new
    }

    // OPTIMIZE: allocates String just for referenced &String lookup every op
    pub fn zscore(&self, key: &K, member: &K) -> Option<V> {
        self.sets.get(key)?.get(member).cloned()
    }

    pub fn zrem(&mut self, key: &K, member: &K) -> Option<V> {
        let list = self.sets.get_mut(key)?;
        let result = list.remove(member);
        if list.is_empty() {
            self.sets.remove(key);
        }
        result
    }
}