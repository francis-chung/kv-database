use std::collections::HashMap;

pub struct HashMapWrapper<K, V> {
    inner: HashMap<K, V>
}

impl<K, V> HashMapWrapper<K, V> 
where 
    K: Eq + std::hash::Hash
{
    pub fn new() -> Self {
        Self { inner: HashMap::new() }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        println!("Adding data into the wrapper...");
        self.inner.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }
}

fn main() {
    let mut new_map = HashMapWrapper::new();
    new_map.insert(String::from("asdf"), 30);
    match new_map.get(&String::from("asdf")) {
        Some(value) => println!("A value was found: {}.", value), 
        None => println!("No value was found.")
    }
}
