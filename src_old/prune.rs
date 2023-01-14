use std::collections::HashMap;

use crate::node::Node;
use crate::phase1::N_PHASE1_STATES;
use crate::phase2::N_PHASE2_STATES;
use crate::phase3::N_PHASE3_STATES;
use crate::utils::PhaseEnum;

pub trait PruningTable {
    // Creates an empty pruning table for the given phase
    fn new(phase: PhaseEnum, max_depth: u8) -> Self;

    // Set the depth of the node
    fn set_depth(&mut self, node: &impl Node, depth: u8);

    // Gets the depth of the node
    fn get_depth(&self, node: &impl Node) -> u8;

    // Updates the max depth if all states were found before the maximum depth
    fn update_max_depth(&mut self, new_max_depth: u8);
}

pub struct HashMapPruningTable {
    data: HashMap<usize, u8>,
    max_depth: u8,
}

impl PruningTable for HashMapPruningTable {
    // Maybe add initialization capacity?
    fn new(_phase: PhaseEnum, max_depth: u8) -> Self {
        HashMapPruningTable {
            data: HashMap::<usize, u8>::new(),
            max_depth,
        }
    }

    #[inline]
    fn get_depth(&self, node: &impl Node) -> u8 {
        match self.data.get(&node.get_index()) {
            Some(depth) => *depth,
            None => self.max_depth + 1,
        }
    }

    #[inline]
    fn set_depth(&mut self, node: &impl Node, depth: u8) {
        self.data.insert(node.get_index(), depth);
    }

    #[inline]
    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth;
    }
}

pub type ArrayPruningTable = Box<[u8]>;

impl PruningTable for ArrayPruningTable {
    fn new(phase: PhaseEnum, max_depth: u8) -> Self {
        // Pointer size on some machines may be too small to even allocate the required memory.
        match phase {
            PhaseEnum::Phase1 => vec![max_depth + 1; N_PHASE1_STATES].into_boxed_slice(),
            PhaseEnum::Phase2 => vec![max_depth + 1; N_PHASE2_STATES].into_boxed_slice(),
            PhaseEnum::Phase3 => vec![max_depth + 1; N_PHASE3_STATES].into_boxed_slice(),
        }
    }

    #[inline]
    fn set_depth(&mut self, node: &impl Node, depth: u8) {
        self[node.get_index()] = depth;
    }

    #[inline]
    fn get_depth(&self, node: &impl Node) -> u8 {
        self[node.get_index()]
    }

    #[inline(always)]
    fn update_max_depth(&mut self, _new_max_depth: u8) {}
}

struct DepthQueue<T> {
    depth: u8,
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

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.queue1.is_empty() && self.queue2.is_empty()
    }

    #[inline]
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
        return self.pop();
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self.pop_from_first {
            true => self.queue1.len(),
            false => self.queue2.len(),
        }
    }

    #[inline]
    pub fn depth(&self) -> u8 {
        self.depth
    }
}

pub fn gen_pruning_table<P: PruningTable, N: Node>(
    phase: PhaseEnum,
    max_depth: u8,
) -> impl PruningTable {
    let mut pruning_table = P::new(phase, max_depth);

    let mut queue = DepthQueue::<N>::new();

    let goal = N::goal();
    pruning_table.set_depth(&N::goal(), 0);
    queue.push(goal);

    while queue.depth > max_depth {
        if let Some(node) = queue.pop() {
            for new_node in node.connected().into_iter() {
                if pruning_table.get_depth(&new_node) > queue.depth() {
                    pruning_table.set_depth(&new_node, queue.depth());
                    if queue.depth <= max_depth {
                        queue.push(new_node);
                    }
                }
            }
        } else {
            pruning_table.update_max_depth(queue.depth());
            break;
        }
    }

    return pruning_table;
}
