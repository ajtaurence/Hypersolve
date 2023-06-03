use rkyv::Archive;
use rkyv::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Add, Deref, DerefMut},
};

use crate::{node_cube::node::Node, phases::Phase};

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

pub trait ArchivedPruningTable {
    type Phase: Phase;

    /// Gets a lower bound on the depth of the node
    fn get_depth(&self, node: <<Self as ArchivedPruningTable>::Phase as Phase>::Node) -> u8;

    /// Gets the depth and returns whether the depth is exact or bounded
    fn get_exact_or_bounded_depth(
        &self,
        node: <<Self as ArchivedPruningTable>::Phase as Phase>::Node,
    ) -> ExactOrBound<u8> {
        let depth = self.get_depth(node);
        match depth < self.get_max_depth() {
            true => ExactOrBound::Exact(depth),
            false => ExactOrBound::LowerBound(depth),
        }
    }

    /// Gets the max depth if all states were found before the maximum depth
    fn get_max_depth(&self) -> u8;
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

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct HashMapPruningTable<T: Phase> {
    pub data: HashMap<u64, u8>,
    pub max_depth: u8,
    phantom: PhantomData<T>,
}

impl<T: Phase> PruningTable for HashMapPruningTable<T> {
    type Phase = T;
    // TODO: Maybe add initialization capacity?
    fn new(max_depth: u8) -> Self {
        HashMapPruningTable {
            data: HashMap::<u64, u8>::new(),
            max_depth,
            phantom: PhantomData,
        }
    }

    fn get_depth(&self, node: T::Node) -> u8 {
        match self.data.get(&(node.get_index() as u64)) {
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

impl<T: Phase> ArchivedPruningTable for ArchivedHashMapPruningTable<T> {
    type Phase = T;
    fn get_depth(&self, node: <<Self as ArchivedPruningTable>::Phase as Phase>::Node) -> u8 {
        match self.data.get(&(node.get_index() as u64)) {
            Some(&depth) => depth,
            None => self.max_depth + 1,
        }
    }

    fn get_max_depth(&self) -> u8 {
        self.max_depth
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
pub struct ArrayPruningTable<T: Phase> {
    data: Box<[u8]>,
    max_depth: u8,
    phantom: PhantomData<T>,
}

impl<T: Phase> Deref for ArrayPruningTable<T> {
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
        self[node.get_index() as usize] = depth;
    }

    fn get_depth(&self, node: T::Node) -> u8 {
        self[node.get_index() as usize]
    }

    fn get_max_depth(&self) -> u8 {
        self.max_depth
    }

    fn update_max_depth(&mut self, new_max_depth: u8) {
        self.max_depth = new_max_depth
    }

    fn finalize(&mut self) {}
}

impl<T: Phase> ArchivedPruningTable for ArchivedArrayPruningTable<T> {
    type Phase = T;
    fn get_depth(&self, node: <<Self as ArchivedPruningTable>::Phase as Phase>::Node) -> u8 {
        self.data[node.get_index() as usize]
    }

    fn get_max_depth(&self) -> u8 {
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

// pub fn explore<T: Phase>() -> bit_vec::BitVec {
//     use bit_vec::BitVec;
//     let mut bit_array = BitVec::from_elem(T::N_STATES, false);

//     let mut queue = DepthQueue::<T::Node>::new();

//     let goal = T::Node::goal();
//     queue.push(goal);

//     let mut num_count: u64 = 0;
//     // forward search mode
//     loop {
//         if let Some(node) = queue.pop() {
//             for new_node in node.connected().into_iter() {
//                 if !bit_array[new_node.get_index()] {
//                     bit_array.set(new_node.get_index(), true);

//                     if queue.depth < 8 {
//                         queue.push(new_node);
//                     } else {
//                         num_count += 1;
//                     }
//                 }
//             }
//         } else {
//             break;
//         }
//     }
//     println!("Depth: {}: {} nodes", queue.depth, num_count);

//     // reverse search mode
//     let mut depth = queue.depth;
//     loop {
//         depth += 1;
//         let mpb = indicatif::MultiProgress::new();

//         let progress = indicatif::ProgressBar::new(T::N_STATES as u64)
//             .with_message(format!("Searching depth {:?}", depth));
//         progress.set_style(
//             indicatif::ProgressStyle::with_template(
//                 "{msg}: {percent}% of {human_len} nodes {bar:40} {eta}",
//             )
//             .unwrap(),
//         );
//         let progress = mpb.add(progress);

//         let count = indicatif::ProgressBar::new(T::N_STATES as u64);
//         count.set_style(
//             indicatif::ProgressStyle::with_template(
//                 "{percent}% of {human_pos}/{human_len} nodes {bar:40} {eta}",
//             )
//             .unwrap(),
//         );

//         let count = mpb.add(count);

//         let mut flip_count: u64 = 0;

//         let last_bit_array = bit_array.clone();

//         for (i, bit) in last_bit_array.iter().enumerate() {
//             progress.inc(1);
//             if bit {
//                 continue;
//             }

//             let node = T::Node::from_index(i, None);

//             for new_node in node.connected() {
//                 if last_bit_array[new_node.get_index()] {
//                     flip_count += 1;
//                     count.inc(1);
//                     bit_array.set(i, true);
//                     break;
//                 }
//             }
//         }
//         progress.abandon();
//         count.abandon();
//         println!("States found at depth {}: {}", depth, flip_count);

//         if bit_array.all() {
//             return bit_array;
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase1_pruning_table() {
        use crate::phases::Phase1;
        let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase1>(2);
        // Should have found 166 nodes
        assert_eq!(pruning_table.data.len(), 166);
    }

    #[test]
    fn test_phase2_pruning_table() {
        use crate::phases::Phase2;
        let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase2>(2);
        // Should have found 152 nodes
        assert_eq!(pruning_table.data.len(), 152);
    }

    #[test]
    fn test_phase3_pruning_table() {
        use crate::phases::Phase3;
        let pruning_table = gen_pruning_table::<HashMapPruningTable<_>, Phase3>(2);
        // Should have found 70 nodes
        assert_eq!(pruning_table.data.len(), 70);
    }
}
