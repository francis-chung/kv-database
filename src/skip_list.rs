use std::collections::HashMap;

pub struct SkipList<K, V> 
{
    nodes: Vec<Node<K, V>>, 
    head: Vec<Option<usize>>, 
    free_list: Vec<usize>, 
    key_to_pos: HashMap<K, usize>, 
    max_level: usize, 
    level: usize
}

impl<K, V> SkipList<K, V>
where 
    K: Ord + std::hash::Hash + Clone, 
    V: Ord + Clone
{
    pub fn new(max_level: usize) -> Self {
        SkipList {
            nodes: Vec::new(), 
            head: Vec::new(), 
            free_list: Vec::new(), 
            key_to_pos: HashMap::new(), 
            max_level, 
            level: 0
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let pos = *self.key_to_pos.get(key)?;
        Some(&self.nodes[pos].score)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut to_update: Vec<usize> = vec![0; self.max_level];
        let mut t: bool = true;
        let mut index: usize = 0;
        for i in (0..=self.level).rev() {
            let mut next = match t {
                true => self.head[i],
                false => self.nodes[index].forward[i]
            };
            while let Some(next_val) = next && self.nodes[next_val].score > value {
                t = false;
                index = next_val;
                next = self.nodes[index].forward[i];
            }
            to_update[i] = match t {
                true => usize::MAX, 
                false => index
            };
        }
        let mut new_level: usize = 0;
        while rand::random() && new_level < self.max_level - 1 {
            new_level += 1
        }
        while self.level < new_level {
            self.level += 1;
            self.head.push(None);
            to_update[self.level] = usize::MAX;
        }
        let pos = match self.free_list.len() {
            x if x == 0 => {
                let result = self.nodes.len();
                self.nodes.push(Node::new(key, value, self.max_level));
                result
            }
            _ => {
                let result = self.free_list.pop().unwrap();
                self.nodes[result] = Node::new(key, value, self.max_level);
                result
            }
        };
        for i in (0..=new_level) {
            let current = to_update[i];
            match current {
                usize::MAX => {
                    self.nodes[pos].forward[i] = self.head[i];
                    self.head[i] = Some(pos);
                }
                prev_index => {
                    self.nodes[pos].forward[i] = self.nodes[prev_index].forward[i];
                    self.nodes[prev_index].forward[i] = Some(pos);
                }
            }
        }
    }
}

struct Node<K, V> 
{
    key: K, 
    score: V, 
    forward: Vec<Option<usize>>
}

impl<K, V> Node<K, V>
where
    K: Ord + std::hash::Hash + Clone,
    V: Ord + Clone
{
    fn new(key: K, score: V, max_level: usize) -> Self {
        Node {
            key, 
            score, 
            forward: vec![None; max_level]
        }
    }
}