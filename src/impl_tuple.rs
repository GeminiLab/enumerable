use crate::Enumerable;

/// This is an implementation of the `Enumerable` trait for `()`.
impl Enumerable for () {
    type Enumerator = std::iter::Once<()>;

    /// This method returns an iterator over all possible values of `()`.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut iter = <() as enumerable::Enumerable>::enumerator();
    /// assert_eq!(iter.next(), Some(()));
    /// assert_eq!(iter.next(), None);
    /// ```
    fn enumerator() -> Self::Enumerator {
        std::iter::once(())
    }
}

/// Enumerator for `(A,)`.
pub struct Tuple1Enumerator<A>
where
    A: Enumerable,
{
    a_enumerator: A::Enumerator,
}

impl<A: Enumerable> Iterator for Tuple1Enumerator<A> {
    type Item = (A,);

    fn next(&mut self) -> Option<Self::Item> {
        self.a_enumerator.next().map(|a| (a,))
    }
}

impl<A> Enumerable for (A,)
where
    A: Enumerable,
{
    type Enumerator = Tuple1Enumerator<A>;

    fn enumerator() -> Self::Enumerator {
        Tuple1Enumerator {
            a_enumerator: A::enumerator(),
        }
    }
}

/// Enumerator for `(A, B)`.
///
/// For `A` yielding `a0, a1, ...` and `B` yielding `b0, b1, ...`, this enumerator yields
/// `(a0, b0), (a0, b1), ..., (a1, b0), (a1, b1), ...`.
pub struct Tuple2Enumerator<A, B>
where
    A: Enumerable,
    B: Enumerable,
{
    a_enumerator: A::Enumerator,
    a_with_b_enumerator: Option<(A, B::Enumerator)>,
}

impl<A: Enumerable, B: Enumerable> Iterator for Tuple2Enumerator<A, B> {
    type Item = (A, B);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((a, current_b)) = &mut self.a_with_b_enumerator {
                if let Some(b) = current_b.next() {
                    return Some((*a, b));
                }
            }
            self.a_with_b_enumerator = self.a_enumerator.next().map(|a| (a, B::enumerator()));
            if self.a_with_b_enumerator.is_none() {
                return None;
            }
        }
    }
}

impl<A, B> Enumerable for (A, B)
where
    A: Enumerable,
    B: Enumerable,
{
    type Enumerator = Tuple2Enumerator<A, B>;

    fn enumerator() -> Self::Enumerator {
        Tuple2Enumerator {
            a_enumerator: A::enumerator(),
            a_with_b_enumerator: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Enumerable;

    fn collect_all<T: Enumerable>() -> Vec<T> {
        T::enumerator().collect()
    }

    #[test]
    fn test_tuple0() {
        assert_eq!(vec![()], collect_all::<()>());
    }

    #[test]
    fn test_tuple1() {
        assert_eq!(vec![(false,), (true,)], collect_all::<(bool,)>());
    }

    #[test]
    fn test_tuple2() {
        // Illustrate the return order of the enumerator.
        assert_eq!(
            vec![(0, false), (0, true), (1, false), (1, true), (2, false)],
            <(u8, bool)>::enumerator().take(5).collect::<Vec<_>>()
        );

        // Verify that the enumerator returns all possible values.
        assert_eq!(
            (0..=0xff)
                .flat_map(|a| [false, true].into_iter().map(move |b| (a, b)))
                .collect::<Vec<_>>(),
            collect_all::<(u8, bool)>()
        );
    }
}
