use std::collections::HashMap;

type Link = Option<usize>;

pub struct LRUCache {
    arena: Vec<Node>,
    key_to_pos: HashMap<i32, usize>, 
    cache: HashMap<i32, i32>,
    head: usize,
    tail: usize,
    capacity: usize
}


impl LRUCache {
    fn new(cap: usize) -> Self {
        let mut h = Node::new(-1, -1, 0);
        let mut t = Node::new(-1, -1, 1);
        h.next = Some(1);
        t.prev = Some(0);
        let mut arena = Vec::<Node>::new();
        arena.push(h);
        arena.push(t);

        LRUCache {
            arena, 
            key_to_pos: HashMap::new(), 
            cache: HashMap::new(), 
            head: 0, 
            tail: 1, 
            capacity: cap
        }
    }

    fn addNode(&mut self, cur: usize) {
        let temp = self.arena[self.head].next.unwrap();
        self.arena[cur].prev = Some(self.head);
        self.arena[cur].next = Some(temp);
        self.arena[self.head].next = Some(cur);
        self.arena[temp].prev = Some(cur);
    }

    fn removeNode(&mut self, cur: usize) {
        let org_prev = self.arena[cur].prev.unwrap();
        let org_next = self.arena[cur].next.unwrap();
        self.arena[org_prev].next = Some(org_next);
        self.arena[org_next].prev = Some(org_prev);
    }

    fn moveToHead(&mut self, cur: usize) {
        self.removeNode(cur);
        self.addNode(cur);
    }
    
    fn get(&mut self, key: i32) -> i32 {
        match self.cache.get(&key) {
            Some(&value) => {
                let pos = self.key_to_pos[&key];
                self.moveToHead(pos);
                value
            }
            None => -1
        }
    }
    
    fn put(&mut self, key: i32, value: i32) {
        match self.cache.get(&key) {
            Some(_) => {
                let p = self.key_to_pos[&key];
                self.arena[p].val = value;
                self.cache.insert(key, value);
                self.moveToHead(p);
            }
            None => {
                if self.cache.len() == self.capacity {
                    let last_node = self.arena[self.tail].prev.unwrap();
                    self.cache.remove(&self.arena[last_node].key);
                    self.key_to_pos.remove(&self.arena[last_node].key);
                    self.removeNode(last_node);
                }
                let pos = self.arena.len();
                let cur = Node::new(key, value, pos);
                self.arena.push(cur);
                self.cache.insert(key, value);
                self.key_to_pos.insert(key, pos);
                self.addNode(pos);
            }
        }
    }
}

struct Node {
    key: i32, 
    val: i32, 
    pos: usize, 
    prev: Link, 
    next: Link
}

impl Node {
    pub fn new(k: i32, v: i32, p: usize) -> Self {
        Node {
            key: k, 
            val: v, 
            pos: p, 
            prev: None, 
            next: None
        }
    }
}