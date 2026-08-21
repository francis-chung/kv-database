use std::collections::HashMap;
use ordered_float::OrderedFloat;

use crate::lru_cache::LRUCache;
use crate::sorted_set_store::SortedSetStore;

const LRU_CAPACITY: usize = 50;

pub struct Db {
    pub kv_store: HashMapWrapper<String, String>, 
    pub sorted_sets: SortedSetStore<String, OrderedFloat<f64>>
}

impl Db {
    pub fn new() -> Self {
        Self {
            kv_store: HashMapWrapper::<String, String>::new(),
            sorted_sets: SortedSetStore::<String, OrderedFloat<f64>>::new()
        }
    }
}

pub struct HashMapWrapper<K, V> {
    map: HashMap<K, V>,
    cache: LRUCache<K, V>,
    hits: usize,
    misses: usize,
}

impl<K, V> HashMapWrapper<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            cache: LRUCache::<K, V>::new(LRU_CAPACITY),
            hits: 0,
            misses: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        // if key in LRU cache, allows for faster access
        if let Some(value) = self.cache.get(key) {
            self.hits += 1;
            return Some(value);
        }
        match self.map.get(key) {
            Some(value) => {
                self.hits += 1;
                self.cache.put(key.clone(), value.clone());
                Some(value.clone())
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }

    pub fn contains_key(&mut self, key: &K) -> bool {
        if let Some(_) = self.cache.get(key) {
            self.hits += 1;
            return true;
        }
        match self.map.get(key) {
            Some(value) => {
                self.hits += 1;
                self.cache.put(key.clone(), value.clone());
                true
            }
            None => {
                self.misses += 1;
                false
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.cache.put(key.clone(), value.clone());
        self.map.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.del(key);
        self.map.remove(key)
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }
}

impl<K, V> Default for HashMapWrapper<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
