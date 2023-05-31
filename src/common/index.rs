/// A trait for types that can be converted to or from an index.
/// Every index in [0-NUM_INDICES) must represent a unique and valid value.
pub trait Index {
    /// Number of indices for this type
    const NUM_INDICES: u64;

    /// Converts the index into the corresponding value
    fn from_index(index: u64) -> Self;

    /// Converts the value into its index
    fn to_index(self) -> u64;

    /// Iterates over all values in order of increasing index
    fn iter_in_order() -> IndexIterator<Self>
    where
        Self: Sized,
    {
        IndexIterator::new(Self::NUM_INDICES)
    }
}

/// An iterator over values which implement the Index trait
pub struct IndexIterator<T: Index> {
    index: u64,
    num_indices: u64,
    phantom: std::marker::PhantomData<T>,
}

impl<T: Index> IndexIterator<T> {
    fn new(num_indices: u64) -> Self {
        Self {
            index: 0,
            num_indices,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Index> Iterator for IndexIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.num_indices {
            return None;
        }

        let result = Some(T::from_index(self.index));
        self.index += 1;

        return result;
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = (self.num_indices - self.index) as usize;

        (size, Some(size))
    }
}

impl<T: Index> ExactSizeIterator for IndexIterator<T> {
    fn len(&self) -> usize {
        (self.num_indices - self.index) as usize
    }
}
