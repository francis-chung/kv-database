use std::{
    collections::HashMap, 
    sync::{Mutex, atomic::{AtomicUsize, Ordering}}
};

use crate::lru_cache::LRUCache;

const LRU_CAPACITY: usize = 50;

pub struct HashMapWrapper<K, V> {
    map: HashMap<K, V>,
    cache: Mutex<LRUCache<K, V>>,
    hits: AtomicUsize,
    misses: AtomicUsize,
}

impl<K, V> HashMapWrapper<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            cache: Mutex::new(LRUCache::<K, V>::new(LRU_CAPACITY)),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            cache: Mutex::new(LRUCache::<K, V>::new(LRU_CAPACITY)),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<V> {
        { // if key in LRU cache, allows for faster access
            let mut cache = self.cache.lock().unwrap();
            if let Some(value) = cache.get(key) {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(value);
            }
        }
        match self.map.get(key) {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                let mut cache = self.cache.lock().unwrap();
                cache.put(key.clone(), value.clone());
                Some(value.clone())
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn get_multiple(&self, keys: &[K]) -> Vec<Option<V>> {
        keys.iter().map(|key| self.get(key)).collect()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(_) = cache.get(key) {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }
        match self.map.get(key) {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                let mut cache = self.cache.lock().unwrap();
                cache.put(key.clone(), value.clone());
                true
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let result = self.map.insert(key.clone(), value.clone());
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, value);
        result
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
            self.hits.fetch_add(1, Ordering::Relaxed);
            f(value);
            true
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            false
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().unwrap();
        cache.del(key);
        drop(cache);
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
