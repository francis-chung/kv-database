use std::collections::HashMap;

type Link = Option<usize>;

pub struct LRUCache<K, V> {
    arena: Vec<Node<K, V>>,
    key_to_pos: HashMap<K, usize>, 
    cache: HashMap<K, V>,
    free_indices: Vec<usize>, 
    head: usize,
    tail: usize,
    capacity: usize
}


impl<K, V> LRUCache<K, V> 
where 
    K: Eq + std::hash::Hash + Clone, 
    V: Clone, 
{
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        
        let mut h = Node::new(None, None);
        let mut t = Node::new(None, None);
        h.next = Some(1);
        t.prev = Some(0);
        let mut arena = Vec::<Node<K, V>>::new();
        arena.push(h);
        arena.push(t);

        LRUCache {
            arena, 
            key_to_pos: HashMap::new(), 
            cache: HashMap::new(), 
            free_indices: Vec::new(),
            head: 0, 
            tail: 1, 
            capacity: cap
        }
    }

    fn add_node(&mut self, cur: usize) {
        let temp = self.arena[self.head].next.unwrap();
        self.arena[cur].prev = Some(self.head);
        self.arena[cur].next = Some(temp);
        self.arena[self.head].next = Some(cur);
        self.arena[temp].prev = Some(cur);
    }

    fn remove_node(&mut self, cur: usize) {
        let org_prev = self.arena[cur].prev.unwrap();
        let org_next = self.arena[cur].next.unwrap();
        self.arena[org_prev].next = Some(org_next);
        self.arena[org_next].prev = Some(org_prev);
    }

    fn move_to_head(&mut self, cur: usize) {
        self.remove_node(cur);
        self.add_node(cur);
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        match self.cache.get(key) {
            Some(value) => {
                let value = value.clone();
                let pos = self.key_to_pos[&key];
                self.move_to_head(pos);
                Some(value)
            }
            None => None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        match self.cache.get(&key) {
            Some(_) => {
                let p = self.key_to_pos[&key];
                self.arena[p].val = Some(value.clone());
                self.cache.insert(key, value);
                self.move_to_head(p);
            }
            None => {
                if self.cache.len() == self.capacity {
                    let last_node = self.arena[self.tail].prev.unwrap();
                    let last_key = self.arena[last_node].key.clone().unwrap();
                    self.free_indices.push(self.key_to_pos[&last_key]);
                    self.cache.remove(&last_key);
                    self.key_to_pos.remove(&last_key);
                    self.remove_node(last_node);
                }
                match self.free_indices.len() {
                    x if x == 0 => {
                        let pos = self.arena.len();
                        let cur = Node::new(Some(key.clone()), Some(value.clone()));
                        self.arena.push(cur);
                        self.key_to_pos.insert(key.clone(), pos);
                        self.add_node(pos);
                    }
                    _ => {
                        let pos = self.free_indices.pop().unwrap();
                        let cur = Node::new(Some(key.clone()), Some(value.clone()));
                        self.arena[pos] = cur;
                        self.key_to_pos.insert(key.clone(), pos);
                        self.add_node(pos);
                    }
                }
                self.cache.insert(key, value);
            }
        }
    }

    pub fn del(&mut self, key: &K) {
        if let Some(pos) = self.key_to_pos.remove(key) {
            self.free_indices.push(pos);
            self.cache.remove(key);
            self.remove_node(pos);
        }
    }
}

struct Node<K, V> {
    key: Option<K>, 
    val: Option<V>, 
    prev: Link, 
    next: Link
}

impl<K, V> Node<K, V>
where 
    K: Eq + std::hash::Hash + Clone, 
    V: Clone,
{
    pub fn new(k: Option<K>, v: Option<V>) -> Self {
        Node {
            key: k, 
            val: v, 
            prev: None, 
            next: None
        }
    }
}