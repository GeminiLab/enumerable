mod impl_tuple;
pub use impl_tuple::*;

/// `Enumerable` is a trait for types that can have their possible values enumerated.
///
/// ## Derivable
///
/// This trait can be derived using `#[derive(Enumerable)]` on:
/// - Enums with no associated data.
/// - Structs with fields that implement `Enumerable`.
///
/// It's NOT guaranteed that the derived implementation will return a specific type of [`Iterator`].
/// Do NOT rely on the type of the iterator used by the derived implementation.
///
/// It's guaranteed that the derived implementation will yield all possible variants of the enum
/// from the top to the bottom.
///
/// It's guaranteed that the derived implementation will yield all possible values of the struct
/// in a lexicographic ordering based on the top-to-bottom declaration order of the struct’s members,
/// as [`Ord`] does.
///
/// ## Built-in Implementations
///
/// The following types have built-in implementations of the `Enumerable` trait:
/// - `bool`: Yields `false` and then `true`.
/// - Numeric types: Yields all possible values of the type from the minimum to the maximum one.
/// - [`Option`]: Yields `None` and then `Some(item)` for each possible value of `T`.
/// - [`Result`]: Yields `Ok(item)` for each possible value of `T` and `Err(error)` for each possible value of `E`.
/// - `char`: Yields all possible Unicode scalar values from `U+0000` to `U+10FFFF`, excluding the surrogate code points.
/// - Tuples: Yields all possible values of the tuple with 1 to 2 elements, in a lexicographic ordering, provided that all elements implement `Enumerable`.
/// - `()`: Yields the unit value `()`.
///
/// ## Example
///
/// ```
/// use enumerable::Enumerable;
///
/// #[derive(Copy, Clone, Eq, PartialEq, Debug, Enumerable)]
/// enum SomeEnum { A, B, C }
///
/// let mut enumerated = SomeEnum::enumerator().collect::<Vec<_>>();
/// assert_eq!(enumerated, vec![SomeEnum::A, SomeEnum::B, SomeEnum::C]);
///
/// let mut enumerated = Option::<SomeEnum>::enumerator().collect::<Vec<_>>();
/// assert_eq!(enumerated, vec![None, Some(SomeEnum::A), Some(SomeEnum::B), Some(SomeEnum::C)]);
/// ```
///
pub trait Enumerable: Copy {
    /// The type of the iterator that will be returned by the `enumerator` method.
    type Enumerator: Iterator<Item = Self>;
    /// Return an iterator over all possible values of the implementing type.
    fn enumerator() -> Self::Enumerator;
}

/// Macro to implement the `Enumerable` trait for a numeric type.
macro_rules! impl_enumerable_for_numeric_type {
    ($ty:ty) => {
        #[automatically_derived]
        impl Enumerable for $ty {
            type Enumerator = std::ops::RangeInclusive<$ty>;

            /// Returns an iterator over all possible values of this type.
            fn enumerator() -> Self::Enumerator {
                <$ty>::MIN..=<$ty>::MAX
            }
        }
    };
}

/// Macro to implement the `Enumerable` trait for multiple numeric types.
macro_rules! impl_enumerable_for_numeric_types {
    ($ty:ty) => { impl_enumerable_for_numeric_type!($ty); };
    ($ty:ty, $($tys:ty),+) => {
        impl_enumerable_for_numeric_type!($ty);
        impl_enumerable_for_numeric_types!($($tys),+);
    };
}

// Implement the `Enumerable` trait for all standard numeric types.
impl_enumerable_for_numeric_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// `BoolEnumeratorState` is an enum that represents the state of a `BoolEnumerator`.
/// It has three variants: `False`, `True`, and `Done`.
/// `False` means that the next item to yield is `false`.
/// `True` means that the next item to yield is `true`.
/// `Done` means that all items have been yielded.
enum BoolEnumeratorState {
    False,
    True,
    Done,
}

/// `BoolEnumerator` is the iterator over `false` and `true`.
pub struct BoolEnumerator {
    state: BoolEnumeratorState,
}

impl BoolEnumerator {
    /// Creates a new `BoolEnumerator`.
    fn new() -> Self {
        Self {
            state: BoolEnumeratorState::False,
        }
    }
}

/// This is an implementation of the `Iterator` trait for `BoolEnumerator`.
impl Iterator for BoolEnumerator {
    type Item = bool;

    /// Returns the next item from the `BoolEnumerator`.
    ///
    /// # Returns
    ///
    /// If the current state is `False`, sets the state to `True` and returns `Some(false)`.
    /// If the current state is `True`, sets the state to `Done` and returns `Some(true)`.
    /// If the current state is `Done`, returns `None`.
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            BoolEnumeratorState::False => {
                self.state = BoolEnumeratorState::True;
                Some(false)
            }
            BoolEnumeratorState::True => {
                self.state = BoolEnumeratorState::Done;
                Some(true)
            }
            _ => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.state {
            BoolEnumeratorState::False => (2, Some(2)),
            BoolEnumeratorState::True => (1, Some(1)),
            BoolEnumeratorState::Done => (0, Some(0)),
        }
    }
}

impl ExactSizeIterator for BoolEnumerator {}

/// This is an implementation of the `Enumerable` trait for `bool`.
impl Enumerable for bool {
    type Enumerator = BoolEnumerator;

    /// This method returns an iterator over all possible values of `bool`.
    /// It creates a new `BoolEnumerator`.
    fn enumerator() -> Self::Enumerator {
        BoolEnumerator::new()
    }
}

/// This is an implementation of the `Enumerable` trait for `char`.
impl Enumerable for char {
    type Enumerator =
        std::iter::Chain<std::ops::RangeInclusive<char>, std::ops::RangeInclusive<char>>;

    /// This method returns an iterator over all possible values of `char`, which is `U+0000` to
    /// `U+10FFFF`, excluding the surrogate code points.
    ///
    /// ## Example
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// assert_eq!(char::enumerator().skip(0x41).next(), Some('\u{41}'));
    /// ```
    fn enumerator() -> Self::Enumerator {
        ('\u{0}'..='\u{D7FF}').chain('\u{E000}'..='\u{10FFFF}')
    }
}

/// `OptionEnumerator` is an iterator over possible values of `Option<T>`.
/// It yields `None` first, then yields `Some(item)` for each possible value of `T`.
pub struct OptionEnumerator<T: Enumerable> {
    first: bool,
    inner: <T as Enumerable>::Enumerator,
}

impl<T> OptionEnumerator<T>
where
    T: Enumerable,
{
    /// Creates a new `OptionEnumerator` that wraps the enumerator of `T`.
    pub(crate) fn new() -> Self {
        Self {
            first: true,
            inner: T::enumerator(),
        }
    }
}

/// This is an implementation of the `Iterator` trait for `Option<T>` where `T` is `Enumerable`.
impl<T> Iterator for OptionEnumerator<T>
where
    T: Enumerable,
{
    type Item = Option<T>;

    /// Returns the next item from the `OptionEnumerator`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            Some(None)
        } else {
            match self.inner.next() {
                Some(item) => Some(Some(item)),
                None => None,
            }
        }
    }
}

/// This is an implementation of the `Enumerable` trait for `Option<T>` where `T` is `Enumerable`.
impl<T> Enumerable for Option<T>
where
    T: Enumerable,
{
    type Enumerator = OptionEnumerator<T>;

    /// This method returns an iterator over all possible values of `Option<T>`.
    fn enumerator() -> Self::Enumerator {
        OptionEnumerator::new()
    }
}

/// `ResultEnumerator` is an iterator over possible values of `Result<T, E>`.
pub struct ResultEnumerator<T: Enumerable, E: Enumerable> {
    over_results: bool,
    results: <T as Enumerable>::Enumerator,
    errors: <E as Enumerable>::Enumerator,
}

impl<T, E> ResultEnumerator<T, E>
where
    T: Enumerable,
    E: Enumerable,
{
    /// Creates a new `ResultEnumerator` that wraps the enumerators of `T` and `E`.
    pub(crate) fn new() -> Self {
        Self {
            over_results: true,
            results: T::enumerator(),
            errors: E::enumerator(),
        }
    }
}

impl<T, E> Iterator for ResultEnumerator<T, E>
where
    T: Enumerable,
    E: Enumerable,
{
    type Item = Result<T, E>;

    /// Returns the next item from the `ResultEnumerator`.
    fn next(&mut self) -> Option<Self::Item> {
        if self.over_results {
            match self.results.next() {
                Some(result) => Some(Ok(result)),
                None => {
                    self.over_results = false;
                    self.next()
                }
            }
        } else {
            match self.errors.next() {
                Some(error) => Some(Err(error)),
                None => None,
            }
        }
    }
}

impl<T, E> Enumerable for Result<T, E>
where
    T: Enumerable,
    E: Enumerable,
{
    type Enumerator = ResultEnumerator<T, E>;

    /// This method returns an iterator over all possible values of `Result<T, E>`.
    fn enumerator() -> Self::Enumerator {
        ResultEnumerator::new()
    }
}

pub use enumerable_derive::*;

#[cfg(test)]
#[path = "test.rs"]
mod test;
