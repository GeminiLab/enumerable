use crate::Enumerable;

/// This is an implementation of the `Enumerable` trait for `()`.
impl Enumerable for () {
    type Enumerator = core::iter::Once<()>;

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
        core::iter::once(())
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = Some(1);
}

/// Enumerator for `(A,)`.
#[doc(hidden)]
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

    const ENUMERABLE_SIZE_OPTION: Option<usize> = A::ENUMERABLE_SIZE_OPTION;
}

// impl Enumerable for tuples of size 2..=16
enumerable_derive::__impl_enumerable_for_tuples!(2, 16);
