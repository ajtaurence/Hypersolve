use lazy_static::{__Deref, lazy_static};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Add, DerefMut},
};

use crate::{
    node_cube::node::Node,
    phases::{Phase, Phase1, Phase2, Phase3},
};

lazy_static! {
    pub static ref PHASE1_PRUNING_TABLE: ArrayPruningTable<Phase1> = gen_pruning_table(4);
    pub static ref PHASE2_PRUNING_TABLE: HashMapPruningTable<Phase2> = gen_pruning_table(4);
    pub static ref PHASE3_PRUNING_TABLE: ArrayPruningTable<Phase3> = gen_pruning_table(12);
}

pub trait PruningTable {
    type Phase: Phase;
    /// Creates an empty pruning table for the given phase
    fn new(max_depth: u8) -> Self;

    /// Set the depth of the node
    fn set_depth(&mut self, node: <<Self as PruningTable>::Phase as Phase>::Node, depth: u8);

    /// Gets a lower bound on the depth of the node
    fn get_depth(&self, node: <<Self as PruningTable>::Phase as Phase>::Node) -> u8;

    /// Gets the depth and returns whether the depth is exact or bounded
    fn get_exact_or_bounded_depth(
        &self,
        node: <<Self as PruningTable>::Phase as Phase>::Node,
    ) -> ExactOrBound<u8> {
        let depth = self.get_depth(node);
        match depth < self.get_max_depth() {
            true => ExactOrBound::Exact(depth),
            false => ExactOrBound::LowerBound(depth),
        }
    }

    /// Gets the max depth if all states were found before the maximum depth
    fn get_max_depth(&self) -> u8;

    /// Updates the max depth if all states were found before the maximum depth
    fn update_max_depth(&mut self, new_max_depth: u8);

    /// Anything that needs to be done to the pruning table after it is done being generated
    fn finalize(&mut self);
}

/// Represents an exact value or a lower bound value
pub enum ExactOrBound<T> {
    Exact(T),
    LowerBound(T),
}

impl<T> ExactOrBound<T> {
    pub fn into_inner(self) -> T {
        use ExactOrBound::*;
        match self {
            Exact(value) => value,
            LowerBound(value) => value,
        }
    }

    pub fn is_exact(&self) -> bool {
        use ExactOrBound::*;
        match self {
            Exact(_) => true,
            LowerBound(_) => false,
        }
    }

    pub fn is_bound(&self) -> bool {
        use ExactOrBound::*;
        match self {
            Exact(_) => false,
            LowerBound(_) => true,
        }
    }
}

impl<T: Add<Output = T>> Add for ExactOrBound<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use ExactOrBound::*;
        match (self, rhs) {
            (Exact(a), Exact(b)) => Exact(a + b),
            (Exact(a), LowerBound(b)) => LowerBound(a + b),
            (LowerBound(a), Exact(b)) => LowerBound(a + b),
            (LowerBound(a), LowerBound(b)) => LowerBound(a + b),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashMapPruningTable<T: Phase> {
    pub data: HashMap<usize, u8>,
    pub max_depth: u8,
    phantom: PhantomData<T>,
}

impl<T: Phase> PruningTable for HashMapPruningTable<T> {
    type Phase = T;
    // TODO: Maybe add initialization capacity?
    fn new(max_depth: u8) -> Self {
        HashMapPruningTable {
            data: HashMap::<usize, u8>::new(),
            max_depth,
            phantom: PhantomData,
        }
    }

    fn get_depth(&self, node: T::Node) -> u8 {
        match self.data.get(&node.get_index()) {
            Some(&depth) => depth,
            None => self.max_depth + 1,
        }
    }

    fn set_depth(&mut self, node: T::Node, depth: u8) {
        self.data.insert(node.get_index(), depth);
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth;
    }

    fn get_max_depth(&self) -> u8 {
        self.max_depth
    }

    fn finalize(&mut self) {
        self.data.shrink_to_fit()
    }
}

pub struct ArrayPruningTable<T: Phase> {
    data: Box<[u8]>,
    max_depth: u8,
    phantom: PhantomData<T>,
}

impl<T: Phase> __Deref for ArrayPruningTable<T> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Phase> DerefMut for ArrayPruningTable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T: Phase> PruningTable for ArrayPruningTable<T> {
    type Phase = T;

    fn new(max_depth: u8) -> Self {
        Self {
            data: vec![max_depth + 1; T::N_STATES]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            max_depth,
            phantom: PhantomData,
        }
    }

    fn set_depth(&mut self, node: T::Node, depth: u8) {
        self[node.get_index()] = depth;
    }

    fn get_depth(&self, node: T::Node) -> u8 {
        self[node.get_index()]
    }

    fn get_max_depth(&self) -> u8 {
        self.max_depth
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth
    }

    fn finalize(&mut self) {}
}

struct DepthQueue<T> {
    pub depth: u8,
    pop_from_first: bool,
    queue1: Vec<T>,
    queue2: Vec<T>,
    progress: indicatif::ProgressBar,
}

impl<T> DepthQueue<T> {
    pub fn new() -> Self {
        DepthQueue {
            depth: 0,
            pop_from_first: true,
            queue1: Vec::new(),
            queue2: Vec::new(),
            progress: indicatif::ProgressBar::hidden(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue1.is_empty() && self.queue2.is_empty()
    }

    pub fn len(&self) -> usize {
        match self.pop_from_first {
            false => self.queue1.len(),
            true => self.queue2.len(),
        }
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
            self.progress.inc(1);
            return queue.pop();
        }

        if self.is_empty() {
            self.progress.finish();
            return None;
        }

        self.progress.finish();
        self.progress = indicatif::ProgressBar::new(self.len() as u64)
            .with_message(format!("Searching depth {:?}", self.depth));
        self.progress.set_style(
            indicatif::ProgressStyle::with_template(
                "{msg}: {percent}% of {human_len} nodes {bar:40} {eta}",
            )
            .unwrap(),
        );
        self.pop_from_first = !self.pop_from_first;
        self.depth += 1;
        return self.pop();
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

pub fn gen_pruning_table<P: PruningTable<Phase = T>, T: Phase>(max_depth: u8) -> P {
    let mut pruning_table = P::new(max_depth);

    let mut queue = DepthQueue::<T::Node>::new();

    let goal = T::Node::goal();
    pruning_table.set_depth(<<P as PruningTable>::Phase as Phase>::Node::goal(), 0);
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
fn test_phase1_pruning_table() {
    let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase1>(2);
    // Should have found 166 nodes
    assert_eq!(pruning_table.data.len(), 166);
}

#[test]
fn test_phase2_pruning_table() {
    let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase2>(2);
    // Should have found 152 nodes
    assert_eq!(pruning_table.data.len(), 152);
}

#[test]
fn test_phase3_pruning_table() {
    let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase3>(2);
    // Should have found 70 nodes
    assert_eq!(pruning_table.data.len(), 70);
}
