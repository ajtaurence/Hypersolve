/// A trait for types that can be converted to or from an index.
/// Every index in [0-NUM_INDICES) must represent a unique and valid value.
pub trait Index {
    /// Number of indices for this type
    const NUM_INDICES: u64;

    /// Converts the index into the corresponding value
    ///
    /// # Panics
    /// Panics if the index is out of the valid range
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
    #[allow(unused)]
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

        result
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

macro_rules! impl_index {
    ($type:ty) => {
        impl Index for $type {
            const NUM_INDICES: u64 = Self::MAX as u64 + 1;

            fn from_index(index: u64) -> Self {
                assert!(index < Self::NUM_INDICES);
                index as Self
            }

            fn to_index(self) -> u64 {
                self as u64
            }
        }
    };
}

impl Index for bool {
    const NUM_INDICES: u64 = 2;

    fn from_index(index: u64) -> Self {
        match index {
            0 => false,
            1 => true,
            _ => panic!("index {} out of bounds", index),
        }
    }

    fn to_index(self) -> u64 {
        self as u64
    }
}

for_each!(impl_index!(u8, u16, u32));

impl<T, const N: usize> Index for [T; N]
where
    T: Index + Clone,
{
    const NUM_INDICES: u64 = T::NUM_INDICES.pow(N as u32);

    fn from_index(mut index: u64) -> Self {
        // create an unitialized array
        let mut result: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        // fill the array
        for entry in result.iter_mut() {
            *entry = std::mem::MaybeUninit::new(T::from_index(index % T::NUM_INDICES));
            index /= T::NUM_INDICES;
        }

        // transmute the type of the array now that it is initialized
        unsafe { std::ptr::read(result.as_ptr() as *const [T; N]) }
    }

    fn to_index(self) -> u64 {
        self.iter()
            .enumerate()
            .map(|(i, value)| value.clone().to_index() * T::NUM_INDICES.pow(i as u32))
            .sum()
    }
}
