use rkyv::{Archive, Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use super::*;

/// A trait for objects that can act as pruning tables, storing the lower bound
/// on the distance from solved for any node
pub(crate) trait PruningTable<N: Node> {
    /// Creates an empty pruning table for the given phase
    fn new(max_depth: u8) -> Self;

    /// Set the depth of the node
    fn set_depth(&mut self, node: N, depth: u8);

    /// Gets a lower bound on the depth of the node
    fn get_depth_bound(&self, node: N) -> u8;

    /// Gets the depth to which values are certain
    fn get_pruning_depth(&self) -> u8;

    /// Updates the max depth if all states were found before the maximum depth
    fn update_max_depth(&mut self, new_max_depth: u8);

    /// Anything that needs to be done to the pruning table after it is done being generated
    fn finalize(&mut self);

    /// Generates the pruning table to the desired depth
    fn generate(depth: u8) -> Self
    where
        Self: Sized,
    {
        // Create the pruning table
        let mut pruning_table = Self::new(depth);

        // Create the queue
        let mut queue = DepthQueue::<N>::new();

        // Set the goal node and push it to the queue
        let goal = N::goal();
        pruning_table.set_depth(goal, 0);
        queue.push(goal);

        loop {
            if let Some(node) = queue.pop() {
                for new_node in node.connected::<NodeAxisFilterIterator<N>>() {
                    if pruning_table.get_depth_bound(new_node) > queue.depth() {
                        pruning_table.set_depth(new_node, queue.depth());
                        if queue.depth < depth {
                            queue.push(new_node);
                        }
                    }
                }
            } else {
                pruning_table.update_max_depth(queue.depth());
                break;
            }
        }

        pruning_table.finalize();

        pruning_table
    }
}

/// Like a regular pruning table but is read only and typically generated by rkyv for zero copy deserialization
pub(crate) trait ArchivedPruningTable<N: Node> {
    /// Gets a lower bound on the depth of the node
    fn get_depth_bound(&self, node: N) -> u8;

    /// Gets the depth to which values are certain
    fn get_pruning_depth(&self) -> u8;
}

/// A pruning table backed by a hashmap for storing a selection of nodes in phases where the
/// set of all nodes is too large to store them all.
#[derive(Archive, Serialize, Deserialize)]
pub(crate) struct HashMapPruningTable<N: Node> {
    pub data: HashMap<N::Index, u8>,
    pub max_depth: u8,
}

impl<N: Node> PruningTable<N> for HashMapPruningTable<N> {
    fn new(max_depth: u8) -> Self {
        HashMapPruningTable {
            data: HashMap::new(),
            max_depth,
        }
    }

    fn get_depth_bound(&self, node: N) -> u8 {
        match self.data.get(&{ node.get_index() }) {
            Some(&depth) => depth,
            None => self.max_depth + 1,
        }
    }

    fn set_depth(&mut self, node: N, depth: u8) {
        self.data.insert(node.get_index(), depth);
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth;
    }

    fn get_pruning_depth(&self) -> u8 {
        self.max_depth
    }

    fn finalize(&mut self) {
        self.data.shrink_to_fit()
    }
}

impl<N: Node> ArchivedPruningTable<N> for ArchivedHashMapPruningTable<N>
where
    <N::Index as Archive>::Archived: Hash + std::cmp::Eq,
{
    fn get_depth_bound(&self, node: N) -> u8 {
        match self.data.get(&{ node.get_index() }) {
            Some(&depth) => depth,
            None => self.max_depth + 1,
        }
    }

    fn get_pruning_depth(&self) -> u8 {
        self.max_depth
    }
}

/// A pruning table backed by an array where each state has a unique index to store its distance
#[derive(Debug, Archive, Serialize, Deserialize)]
pub(crate) struct ArrayPruningTable<N: Node> {
    data: Box<[u8]>,
    max_depth: u8,
    phantom: PhantomData<N>,
}

impl<N: Node> Deref for ArrayPruningTable<N> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<N: Node> DerefMut for ArrayPruningTable<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<N: Node> PruningTable<N> for ArrayPruningTable<N> {
    fn new(max_depth: u8) -> Self {
        Self {
            data: vec![max_depth + 1; N::N_STATES].into_boxed_slice(),
            max_depth,
            phantom: PhantomData,
        }
    }

    fn set_depth(&mut self, node: N, depth: u8) {
        self[node.get_index().into() as usize] = depth;
    }

    fn get_depth_bound(&self, node: N) -> u8 {
        self[node.get_index().into() as usize]
    }

    fn get_pruning_depth(&self) -> u8 {
        self.max_depth
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth
    }

    fn finalize(&mut self) {}
}

impl<N: Node> ArchivedPruningTable<N> for ArchivedArrayPruningTable<N> {
    fn get_depth_bound(&self, node: N) -> u8 {
        self.data[node.get_index().into() as usize]
    }

    fn get_pruning_depth(&self) -> u8 {
        self.max_depth
    }
}

struct DepthQueue<T> {
    pub depth: u8,
    pop_from_first: bool,
    queue1: Vec<T>,
    queue2: Vec<T>,
}

impl<T> DepthQueue<T> {
    pub fn new() -> Self {
        DepthQueue {
            depth: 0,
            pop_from_first: true,
            queue1: Vec::new(),
            queue2: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue1.is_empty() && self.queue2.is_empty()
    }

    pub fn push(&mut self, value: T) {
        match self.pop_from_first {
            false => self.queue1.push(value),
            true => self.queue2.push(value),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        let queue = if self.pop_from_first {
            &mut self.queue1
        } else {
            &mut self.queue2
        };

        if !queue.is_empty() {
            return queue.pop();
        }

        if self.is_empty() {
            return None;
        }

        self.pop_from_first = !self.pop_from_first;
        self.depth += 1;
        self.pop()
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase1_pruning_table() {
        let pruning_table = HashMapPruningTable::<Phase1Node>::generate(2);
        // Should have found 166 nodes
        assert_eq!(pruning_table.data.len(), 166);
    }

    #[test]
    fn test_phase2_pruning_table() {
        let pruning_table = HashMapPruningTable::<Phase2Node>::generate(2);
        // Should have found 152 nodes
        assert_eq!(pruning_table.data.len(), 152);
    }

    #[test]
    fn test_phase3_pruning_table() {
        let pruning_table = HashMapPruningTable::<Phase3Node>::generate(2);
        // Should have found 70 nodes
        assert_eq!(pruning_table.data.len(), 70);
    }
}
