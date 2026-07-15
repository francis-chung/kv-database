use std::collections::HashMap;
use ordered_float::OrderedFloat;

use crate::sorted_set::SkipList;

pub struct SortedSetStore {
    sets: HashMap<String, SkipList<String, OrderedFloat<f64>>>
}

impl SortedSetStore {
    pub fn zadd(&mut self, key: &str, member: String, score: f64) -> bool {
        let list = self.sets
            .entry(key.to_string())
            .or_insert_with(|| SkipList::new(16));
        let is_new = list.get(&member).is_none();
        list.insert(member, OrderedFloat(score));
        is_new
    }

    // OPTIMIZE: allocates String just for referenced &String lookup every op
    pub fn zscore(&self, key: &str, member: &str) -> Option<f64> {
        self.sets.get(key)?
            .get(&member.to_string())
            .map(|score| score.0)
    }

    pub fn zrem(&mut self, key: &str, member: &str) -> Option<f64> {
        let list = self.sets.get_mut(key)?;
        let result = list
            .remove(&member.to_string())
            .map(|score| score.0);
        if list.is_empty() {
            self.sets.remove(key);
        }
        result
    }
}