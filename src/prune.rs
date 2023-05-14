use std::collections::HashMap;

use crate::node_cube::node::Node;

pub trait PruningTable<T: Node> {
    /// Creates an empty pruning table for the given phase
    fn new(max_depth: u8) -> Self;

    /// Set the depth of the node
    fn set_depth(&mut self, node: T, depth: u8);

    /// Gets the depth of the node
    fn get_depth(&self, node: T) -> u8;

    /// Updates the max depth if all states were found before the maximum depth
    fn update_max_depth(&mut self, new_max_depth: u8);

    /// Anything that needs to be done to the pruning table after it is done being generated
    fn finalize(&mut self);
}

#[derive(Debug)]
pub struct HashMapPruningTable {
    pub data: HashMap<usize, u8>,
    pub max_depth: u8,
}

impl<T: Node> PruningTable<T> for HashMapPruningTable {
    // Maybe add initialization capacity?
    fn new(max_depth: u8) -> Self {
        HashMapPruningTable {
            data: HashMap::<usize, u8>::new(),
            max_depth,
        }
    }

    fn get_depth(&self, node: T) -> u8 {
        match self.data.get(&node.get_index()) {
            Some(&depth) => depth,
            None => self.max_depth + 1,
        }
    }

    fn set_depth(&mut self, node: T, depth: u8) {
        self.data.insert(node.get_index(), depth);
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth;
    }

    fn finalize(&mut self) {
        self.data.shrink_to_fit()
    }
}

pub type ArrayPruningTable = Box<[u8]>;

impl<T: Node> PruningTable<T> for ArrayPruningTable {
    fn new(max_depth: u8) -> Self {
        vec![max_depth + 1; T::N_STATES].into_boxed_slice()
    }

    fn set_depth(&mut self, node: T, depth: u8) {
        self[node.get_index()] = depth;
    }

    fn get_depth(&self, node: T) -> u8 {
        self[node.get_index()]
    }

    fn update_max_depth(&mut self, _new_max_depth: u8) {}

    fn finalize(&mut self) {}
}

// TODO: check that behavior is correct
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
        println!("depth: {}", self.depth);
        return self.pop();
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

pub fn gen_pruning_table<P: PruningTable<N>, N: Node>(max_depth: u8) -> P {
    let mut pruning_table = P::new(max_depth);

    let mut queue = DepthQueue::<N>::new();

    let goal = N::goal();
    pruning_table.set_depth(N::goal(), 0);
    queue.push(goal);

    loop {
        if let Some(node) = queue.pop() {
            for new_node in node.connected().into_iter() {
                if pruning_table.get_depth(new_node) > queue.depth() {
                    pruning_table.set_depth(new_node, queue.depth());
                    if queue.depth < max_depth {
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

    return pruning_table;
}

#[test]
fn test_phase1_pruning_table_gen() {
    use crate::node_cube::node::Phase1Node;
    let pruning_table = gen_pruning_table::<HashMapPruningTable, Phase1Node>(2);
    assert_eq!(pruning_table.data.len(), 166);
}

#[test]
fn test_phase3_pruning_table_gen() {
    use crate::node_cube::node::Phase3Node;
    let pruning_table = gen_pruning_table::<HashMapPruningTable, Phase3Node>(2);
    assert_eq!(pruning_table.data.len(), 70);
}
