use std::collections::HashMap;

use crate::lru_cache::LRUCache;
use crate::sorted_set::SkipList;

const LRU_CAPACITY: usize = 50;

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

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            cache: LRUCache::<K, V>::new(LRU_CAPACITY),
            hits: 0,
            misses: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
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

    pub fn get_multiple(&mut self, keys: &[K]) -> Vec<Option<V>> {
        keys.iter().map(|key| self.get(key)).collect()
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

    pub fn insert_multiple(&mut self, pairs: Vec<(K, V)>) {
        self.map.extend(pairs);
    }

    // FnOnce: a function executed exactly once in this update function
    pub fn update<F>(&mut self, key: &K, f: F) -> bool
    where
        F: FnOnce(&mut V),
    {
        if let Some(value) = self.map.get_mut(key) {
            self.hits += 1;
            f(value);
            true
        } else {
            self.misses += 1;
            false
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.cache.del(key);
        self.map.remove(key)
    }

    pub fn clear(&mut self) {
        self.map.clear()
    }

    // impl... return type shows characteristics of type without
    // defining exactly what it is (especially when true
    // type signature is verbose)
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.map.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.map.iter()
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
