use super::*;

/// The direct product of two groups `T × U`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectProduct<T: Group + 'static, U: Group + 'static>(pub T, pub U);

impl<T: Group, U: Group> Identity for DirectProduct<T, U> {
    const IDENTITY: Self = DirectProduct(T::IDENTITY, U::IDENTITY);
}

impl<T: Group, U: Group> BinaryOp for DirectProduct<T, U> {
    fn binary_op(a: Self, b: Self) -> Self {
        DirectProduct(T::binary_op(a.0, b.0), U::binary_op(a.1, b.1))
    }

    fn order(&self) -> usize {
        crate::math::lcm(self.0.order(), self.1.order())
    }
}

impl<T: Group, U: Group> Inverse for DirectProduct<T, U> {
    fn inverse(&self) -> Self {
        DirectProduct(self.0.inverse(), self.1.inverse())
    }
}

impl<T: Group, U: Group> Group for DirectProduct<T, U> {
    fn iter_elements() -> Box<dyn std::iter::Iterator<Item = Self>> {
        Box::new(
            T::iter_elements()
                .flat_map(|elem1| std::iter::repeat(elem1).zip(U::iter_elements()))
                .map(|(elem1, elem2)| DirectProduct(elem1, elem2)),
        )
    }

    fn group_order() -> usize {
        T::group_order() * U::group_order()
    }

    fn is_closed() -> bool {
        T::is_closed() && U::is_closed()
    }

    fn has_valid_inverse() -> bool {
        T::has_valid_inverse() && U::has_valid_inverse()
    }

    fn is_associative() -> bool {
        T::is_associative() && U::is_associative()
    }

    fn is_abelian() -> bool {
        T::is_abelian() && U::is_abelian()
    }

    fn is_valid_group() -> bool {
        T::is_valid_group() && U::is_valid_group()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_order() {
        assert_eq!(
            DirectProduct(Cyclic::<7>::from_index(2), Cyclic::<2>::from_index(1)).order(),
            14
        )
    }

    #[test]
    fn group_order() {
        assert_eq!(DirectProduct::<Cyclic<7>, Cyclic<10>>::group_order(), 70)
    }
}
