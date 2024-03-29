use super::*;

/// A fixed length vector of an arbitrary type.
///
/// Supports elementwise operations.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Vector<T, const N: usize>(pub(crate) [T; N]);

impl<T, const N: usize> std::fmt::Debug for Vector<T, N>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Vector(")?;
        for i in 0..N {
            f.write_fmt(format_args!("{:?}", self[i]))?;
            if i != N - 1 {
                f.write_str(", ")?;
            }
        }
        f.write_str(")")
    }
}

impl<T, const N: usize> IntoIterator for Vector<T, N> {
    type Item = T;
    type IntoIter = std::array::IntoIter<T, N>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T, const N: usize> std::ops::Index<usize> for Vector<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for Vector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T, const N: usize> std::iter::Sum for Vector<T, N>
where
    Self: num_traits::Zero + std::ops::AddAssign + Clone,
{
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        if let Some(mut result) = iter.next() {
            for next in iter {
                result += next;
            }
            result
        } else {
            num_traits::zero()
        }
    }
}

impl<T, const N: usize> num_traits::Zero for Vector<T, N>
where
    T: num_traits::Zero + Clone,
{
    fn zero() -> Self {
        Self::from_elem(T::zero())
    }

    fn is_zero(&self) -> bool {
        self.iter().all(|i| i.is_zero())
    }
}

impl<T, const N: usize> num_traits::One for Vector<T, N>
where
    T: num_traits::One + std::cmp::PartialEq + Clone,
{
    fn one() -> Self {
        Self::from_elem(T::one())
    }

    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self.iter().all(|i| i.is_one())
    }
}

impl<T, const N: usize> Default for Vector<T, N>
where
    T: Default + Clone,
{
    fn default() -> Self {
        Self::from_elem(T::default())
    }
}

impl<T, const N: usize> std::ops::Deref for Vector<T, N> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> std::ops::DerefMut for Vector<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::from_array(value)
    }
}

impl<T, const N: usize> From<Vector<T, N>> for [T; N] {
    fn from(value: Vector<T, N>) -> Self {
        value.into_array()
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for Vector<T, N> {
    type Error = Vec<T>;
    fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
        let result: Result<[T; N], _> = value.try_into();
        match result {
            Ok(value) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }
}

macro_rules! impl_unary_op_for_vector {
    ($optrait:ident, $opmethod:ident) => {
        impl<T, O, const N: usize> std::ops::$optrait for Vector<T, N>
        where
            T: std::ops::$optrait<Output = O>,
        {
            type Output = Vector<O, N>;
            fn $opmethod(self) -> Self::Output {
                self.map(|i| i.$opmethod())
            }
        }
    };
}
impl_unary_op_for_vector!(Neg, neg);
impl_unary_op_for_vector!(Not, not);

macro_rules! impl_binary_op_for_vector {
    ($optrait:ident, $opmethod:ident, $optraitassign:ident, $opmethodassign:ident) => {
        impl<T, P, O, const N: usize> std::ops::$optrait<Vector<P, N>> for Vector<T, N>
        where
            T: std::ops::$optrait<P, Output = O> + Clone,
            P: Clone,
        {
            type Output = Vector<O, N>;
            fn $opmethod(self, rhs: Vector<P, N>) -> Self::Output {
                Vector::from_function(|i| self[i].clone().$opmethod(rhs[i].clone()))
            }
        }
        impl<T, U, const N: usize> std::ops::$optraitassign<Vector<U, N>> for Vector<T, N>
        where
            T: std::ops::$optraitassign<U>,
            U: Clone,
        {
            fn $opmethodassign(&mut self, rhs: Vector<U, N>) {
                for i in 0..N {
                    self[i].$opmethodassign(rhs[i].clone());
                }
            }
        }
    };
}
impl_binary_op_for_vector!(Add, add, AddAssign, add_assign);
impl_binary_op_for_vector!(Sub, sub, SubAssign, sub_assign);
impl_binary_op_for_vector!(Div, div, MulAssign, mul_assign);
impl_binary_op_for_vector!(Mul, mul, DivAssign, div_assign);
impl_binary_op_for_vector!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_binary_op_for_vector!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_binary_op_for_vector!(BitXor, bitxor, BitXorAssign, bitxor_assign);
impl_binary_op_for_vector!(Shl, shl, ShlAssign, shl_assign);
impl_binary_op_for_vector!(Shr, shr, ShrAssign, shr_assign);
impl_binary_op_for_vector!(Rem, rem, RemAssign, rem_assign);

impl<T, const N: usize> Vector<T, N> {
    pub(crate) fn from_elem(elem: T) -> Self
    where
        T: Clone,
    {
        Vector::from_function(|_| elem.clone())
    }

    pub(crate) const fn from_array(slice: [T; N]) -> Self {
        Self(slice)
    }

    pub(crate) fn into_array(self) -> [T; N] {
        self.0
    }

    /// Creates a vector with elements determined as a function of the index
    pub(crate) fn from_function<F>(mut f: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        // create an unitialized array
        let mut result: [std::mem::MaybeUninit<T>; N] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        // fill the array
        for (i, entry) in result.iter_mut().enumerate() {
            *entry = std::mem::MaybeUninit::new(f(i));
        }

        // transmute the type of the array now that it is initialized
        let transmuted_result = unsafe { std::ptr::read(result.as_ptr() as *const [T; N]) };

        // return the vector
        Vector(transmuted_result)
    }

    /// Maps a function over every element
    pub(crate) fn map<F, U>(self, f: F) -> Vector<U, N>
    where
        F: FnMut(T) -> U,
    {
        Vector(self.into_array().map(f))
    }

    /// The sum of the components of this vector
    pub(crate) fn sum(self) -> T
    where
        T: std::iter::Sum,
    {
        self.0.into_iter().sum()
    }

    /// Dot product between self and another vector
    pub(crate) fn dot<P, O>(self, rhs: Vector<P, N>) -> O
    where
        T: std::ops::Mul<P, Output = O> + Clone,
        P: Clone,
        O: std::iter::Sum,
    {
        dot(self, rhs)
    }

    /// Mangnitude squared of the vector
    pub(crate) fn magnitude_squared<O>(self) -> O
    where
        T: std::ops::Mul<Output = O> + Clone,
        O: std::iter::Sum,
    {
        self.map(|i| i.clone() * i).sum()
    }

    /// Converts a vector of one type into a vector of another
    pub(crate) fn cast<P>(self) -> Vector<P, N>
    where
        P: From<T>,
    {
        self.map(|i| i.into())
    }

    /// Permutes the elements of the vector by the permutation
    pub(crate) fn permute(self, permutation: groups::Permutation<N>) -> Self
    where
        T: Clone,
    {
        let mut result = self.0.clone();

        for (i, value) in result.iter_mut().enumerate() {
            *value = self.0[permutation.into_array()[i]].clone();
        }
        Vector(result)
    }
}

/// Dot product between two vectors.
///
/// Only defined properly for real number components.
pub(crate) fn dot<T, P, O, const N: usize>(a: Vector<T, N>, b: Vector<P, N>) -> O
where
    T: std::ops::Mul<P, Output = O> + Clone,
    P: Clone,
    O: std::iter::Sum,
{
    (a * b).sum()
}

macro_rules! vector {
    ($elem:expr;$n:expr) => (
        $ crate::common::Vector::<_,$n>::from_elem($elem)
    );
    ($($x:expr),*) => ( crate::common::Vector::from_array([$($x),*]) )
}

// Cast a vector from one type of number to another
macro_rules! impl_vector_num_cast {
    ($($type:ty)+) => {};
    ($type1:ty, $type2:ty) => {
        impl<const N: usize> From<Vector<$type1, N>> for Vector<$type2, N> {
            fn from(value: Vector<$type1, N>) -> Self {
                value.map(|i| i as $type2)
            }
        }
        impl<const N: usize> From<Vector<$type2, N>> for Vector<$type1, N> {
            fn from(value: Vector<$type2, N>) -> Self {
                value.map(|i| i as $type1)
            }
        }
    };
}

// The macro that evaluates another macro on all the unique pairs of types given a list of types
macro_rules! for_all_unique_pairs {
    ($mac:ident!($x:ty)) => {
        // If there are no pairs then do nothing
    };
    ($mac:ident!($head:ty, $($tail:ty),*)) => {
        // Evaluate the macro on the first element and all subsequent elements
        $(
            $mac!($head, $tail);
        )*
        // Run the macro for the rest of the sequence
        for_all_unique_pairs!($mac!($($tail),*));
    };
}
// Implelement number type casting
for_all_unique_pairs!(impl_vector_num_cast!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize, f32, f64
));

pub(crate) type Vector3<T> = Vector<T, 3>;
pub(crate) type Vector4<T> = Vector<T, 4>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_zero_no_panic() {
        let v1 = Vector::<usize, 0>::from_elem(1);
        let _ = v1.dot(v1);
        let _ = v1.magnitude_squared();
        let _ = v1.sum();
    }

    #[test]
    fn vector_addition() {
        let v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        let answer = Vector([-1, -17, 31, 30, 2]);
        assert_eq!(v1 + v2, answer)
    }

    #[test]
    fn vector_addition_assign() {
        let mut v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        v1 += v2;
        let answer = Vector([-1, -17, 31, 30, 2]);
        assert_eq!(v1, answer)
    }

    #[test]
    fn vector_subtraction() {
        let v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        let answer = Vector([3, -17, 15, -24, -6]);
        assert_eq!(v1 - v2, answer)
    }

    #[test]
    fn vector_subtraction_assign() {
        let mut v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        v1 -= v2;
        let answer = Vector([3, -17, 15, -24, -6]);
        assert_eq!(v1, answer)
    }

    #[test]
    fn vector_multiplication() {
        let v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        let answer = Vector([-2, 0, 184, 81, -8]);
        assert_eq!(v1 * v2, answer)
    }

    #[test]
    fn vector_multiplication_assign() {
        let mut v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        v1 *= v2;
        let answer = Vector([-2, 0, 184, 81, -8]);
        assert_eq!(v1, answer)
    }

    #[test]
    fn vector_division() {
        let v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 1, 8, 27, 4]);
        let answer = Vector([0, -17, 2, 0, 0]);
        assert_eq!(v1 / v2, answer)
    }

    #[test]
    fn vector_division_assign() {
        let mut v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 1, 8, 27, 4]);
        v1 /= v2;
        let answer = Vector([0, -17, 2, 0, 0]);
        assert_eq!(v1, answer)
    }

    #[test]
    fn vector_macro() {
        assert_eq!(vector!(1, -17, 23, 3, -2), Vector([1, -17, 23, 3, -2]));
        assert_eq!(vector!(1; 5), Vector([1, 1, 1, 1, 1]));
    }

    #[test]
    fn vector_sum() {
        assert_eq!(Vector([1, -17, 23, 3, -2]).sum(), 8)
    }

    #[test]
    fn vector_dot_product() {
        let v1 = Vector([1, -17, 23, 3, -2]);
        let v2 = Vector([-2, 0, 8, 27, 4]);
        assert_eq!(v1.dot(v2), 255)
    }

    #[test]
    fn vector_magnitude() {
        assert_eq!(Vector([1, -17, 23, 3, -2]).magnitude_squared(), 832)
    }

    #[test]
    fn vector_map() {
        assert_eq!(
            Vector([1, -17, 23, 3, -2]).map(|i| i + 2),
            Vector([3, -15, 25, 5, 0])
        )
    }

    #[test]
    fn vector_from_function() {
        assert_eq!(
            Vector::<_, 5>::from_function(|i| 2 * i),
            Vector([0, 2, 4, 6, 8])
        )
    }

    #[test]
    fn vector_num_casting() {
        let _: Vector<i8, 5> = Vector::<f32, 5>::from_array([1., 0., 2., 5., -1.]).into();
        let _: Vector<f64, 5> = Vector::<f32, 5>::from_array([1., 0., 2., 5., -1.]).into();
    }
}
