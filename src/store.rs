use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct HashMapWrapper<K, V> {
    map: HashMap<K, V>, 
    hits: AtomicUsize, 
    misses: AtomicUsize
}

impl<K, V> HashMapWrapper<K, V> 
where 
    K: Eq + std::hash::Hash
{
    pub fn new() -> Self {
        Self { 
            map: HashMap::new(), 
            hits: AtomicUsize::new(0), 
            misses: AtomicUsize::new(0)
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity), 
            hits: AtomicUsize::new(0), 
            misses: AtomicUsize::new(0)
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        match self.map.get(key) {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    pub fn get_multiple(&self, keys: &[K]) -> Vec<Option<&V>> {
        keys.iter()
            .map(|key| self.get(key))
            .collect()
    }

    pub fn contains_key(&self, key: &K) -> bool {
        match self.map.contains_key(key) {
            true => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                true
            }
            false => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                false
            }            
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }

    pub fn insert_multiple(&mut self, pairs: Vec<(K, V)>) {
        self.map.extend(pairs);
    }

    // FnOnce: a function executed exactly once in this update function
    pub fn update<F>(&mut self, key: &K, f: F) -> bool
    where F: FnOnce(&mut V) {
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

impl <K, V> Default for HashMapWrapper<K, V>
where 
    K: Eq + std::hash::Hash
{
    fn default() -> Self { Self::new() }
}
