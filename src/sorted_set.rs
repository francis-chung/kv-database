use std::collections::HashMap;

pub struct SkipList<K, V> 
{
    nodes: Vec<Node<K, V>>, 
    head: Vec<Option<usize>>, 
    head_span: Vec<usize>, 
    free_list: Vec<usize>, 
    pub key_to_pos: HashMap<K, usize>, // REMOVE pub (USED FOR DEBUGGING)
    max_level: usize, 
    level: usize
}

impl<K, V> SkipList<K, V>
where 
    K: Ord + std::hash::Hash + Clone, 
    V: Ord + Clone
{
    pub fn new(max_level: usize) -> Self {
        assert!(max_level > 0);
        
        Self {
            nodes: Vec::new(), 
            head: vec![None; 1], 
            head_span: vec![0; 1], 
            free_list: Vec::new(), 
            key_to_pos: HashMap::new(), 
            max_level, 
            level: 0
        }
    }

    pub fn is_empty(&self) -> bool {
        self.key_to_pos.is_empty()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let pos = *self.key_to_pos.get(key)?;
        Some(&self.nodes[pos].score)
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(&pos) = self.key_to_pos.get(&key) {
            if self.nodes[pos].score == value {
                return;
            }
            self.remove(&key);
        }
        let mut to_update: Vec<Option<usize>> = vec![None; self.max_level];
        let mut rank: Vec<usize> = vec![0; self.max_level];
        let mut index: Option<usize> = None;
        let mut steps: usize = 0;
        for i in (0..=self.level).rev() {
            let mut next = match index {
                Some(index_pos) => self.nodes[index_pos].forward[i], 
                None => self.head[i]
            };
            while let Some(next_val) = next && (
                self.nodes[next_val].score < value || 
                (self.nodes[next_val].score == value && self.nodes[next_val].key < key)
            ) {
                steps += if let Some(inner) = index {
                    self.nodes[inner].span[i]
                } else {
                    self.head_span[i]
                };
                index = Some(next_val);
                next = self.nodes[next_val].forward[i];
            }
            to_update[i] = index;
            rank[i] = steps;
        }
        let mut new_level: usize = 0;
        while rand::random() && new_level < self.max_level - 1 {
            new_level += 1
        }
        while self.level < new_level {
            self.level += 1;
            self.head.push(None);
            self.head_span.push(self.key_to_pos.len());
        }
        let pos = if let Some(result) = self.free_list.pop() {
            self.nodes[result] = Node::new(key.clone(), value, new_level + 1);
            result
        } else {
            self.nodes.push(Node::new(key.clone(), value, new_level + 1));
            self.nodes.len() - 1
        };
        self.key_to_pos.insert(key, pos);
        for i in 0..=new_level {
            match to_update[i] {
                Some(prev_index) => {
                    self.nodes[pos].forward[i] = self.nodes[prev_index].forward[i];
                    self.nodes[prev_index].forward[i] = Some(pos);
                    self.nodes[pos].span[i] = self.nodes[prev_index].span[i] - (rank[0] - rank[i]);
                    self.nodes[prev_index].span[i] = (rank[0] - rank[i]) + 1;
                }
                None => {
                    self.nodes[pos].forward[i] = self.head[i];
                    self.head[i] = Some(pos);
                    self.nodes[pos].span[i] = self.head_span[i] - (rank[0] - rank[i]);
                    self.head_span[i] = (rank[0] - rank[i]) + 1;
                }
            }
        }
        for i in (new_level + 1)..=self.level {
            match to_update[i] {
                Some(prev_index) => self.nodes[prev_index].span[i] += 1, 
                None => self.head_span[i] += 1
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let &pos = self.key_to_pos.get(key)?;
        let score = self.nodes[pos].score.clone();
        let height = self.nodes[pos].forward.len();
        let mut index: Option<usize> = None;
        for i in (0..=self.level).rev() {
            let mut next = match index {
                Some(index_pos) => self.nodes[index_pos].forward[i], 
                None => self.head[i]
            };
            while let Some(next_val) = next && (
                self.nodes[next_val].score < score || 
                (self.nodes[next_val].score == score && self.nodes[next_val].key < *key)
            ) {
                index = Some(next_val);
                next = self.nodes[next_val].forward[i];
            }
            if i < height {
                match index {
                    Some(prev_pos) => {
                        if self.nodes[prev_pos].forward[i] == Some(pos) {
                            self.nodes[prev_pos].forward[i] = self.nodes[pos].forward[i];
                            self.nodes[prev_pos].span[i] += self.nodes[pos].span[i];
                            self.nodes[prev_pos].span[i] -= 1;
                        }
                    }
                    None => {
                        if self.head[i] == Some(pos) {
                            self.head[i] = self.nodes[pos].forward[i];
                            self.head_span[i] += self.nodes[pos].span[i];
                            self.head_span[i] -= 1;
                        }
                    }
                }
            } else {
                match index {
                    Some(prev_pos) => self.nodes[prev_pos].span[i] -= 1, 
                    None => self.head_span[i] -= 1
                }
            }
        }
        while self.level > 0 && self.head[self.level].is_none() {
            self.level -= 1;
            self.head.pop();
            self.head_span.pop();
        }
        self.free_list.push(pos);
        self.key_to_pos.remove(key);
        Some(score)
    }

    pub fn range(&self, from: usize, to: usize) -> Vec<(usize, &K)> {
        let mut current: Option<usize> = None;
        let mut count: usize = 0;
        for i in (0..=self.level).rev() {
            let mut next = match current {
                Some(current_pos) => self.nodes[current_pos].forward[i],
                None => self.head[i]
            };
            while let Some(next_val) = next {
                let dist = if let Some(inner) = current {
                    self.nodes[inner].span[i]
                } else {
                    self.head_span[i]
                };
                if count + dist >= from {
                    break;
                }
                count += dist;
                current = next;
                next = self.nodes[next_val].forward[i];
            }
        }
        let mut list: Vec<(usize, &K)> = Vec::new();
        for i in 0..=(to - from) {
            current = match current {
                Some(current_pos) => self.nodes[current_pos].forward[0], 
                None => self.head[0]
            };
            list.push((from + i, &self.nodes[current.unwrap()].key));
        }
        list
    }
}

struct Node<K, V> 
{
    key: K, 
    score: V, 
    forward: Vec<Option<usize>>, 
    span: Vec<usize>
}

impl<K, V> Node<K, V>
where
    K: Clone,
    V: Clone
{
    fn new(key: K, score: V, height: usize) -> Self {
        Node {
            key, 
            score, 
            forward: vec![None; height], 
            span: vec![0; height]
        }
    }
}