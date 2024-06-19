mod impl_tuple;
pub use impl_tuple::*;

/// `Enumerable` is a trait for types that can have their possible values enumerated.
///
/// ## Derivable
///
/// This trait can be derived using `#[derive(Enumerable)]` on structs and enums, if
/// - they have no fields, or
/// - all of their fields implement `Enumerable`.
///
/// Types with generic parameters are not supported yet.
///
/// See "Guarantees and Limitations" below for more information.
///
/// ### Customizing the Generated Enumerator
///
/// In most cases, `#[derive(Enumerable)]` will generate a new enumerator type that enumerates all
/// possible values of the type to be derived. The default name of the enumerator type is
/// `<Type>Enumerator`, where `<Type>` is the name of the type to be derived.
///
/// You can customize the name of the generated type by using
/// - `#[enumerator = "DesiredEnumeratorName"]`, or
/// - `#[enumerator(DesiredEnumeratorName)]`,
///
/// they are equivalent.
///
/// `#[derive(Enumerable)]` will NOT generate a new type when the type to be derived is
/// - an enum with zero variants,
/// - an enum with no fields, or
/// - a struct with no fields,
///
/// in these cases, the custom enumerator name will be ignored.
///
/// ## Built-in Implementations
///
/// The following types have built-in implementations of the `Enumerable` trait:
/// - `bool`: Yields `false` and then `true`.
/// - Numeric types: Yields all possible values of the type from the minimum to the maximum one.
/// - [`Option`]: Yields `None` and then `Some(item)` for each possible value of `T`.
/// - [`Result`]: Yields `Ok(item)` for each possible value of `T` and `Err(error)` for each
/// possible value of `E`.
/// - `char`: Yields all possible Unicode scalar values from `U+0000` to `U+10FFFF`, excluding the
/// surrogate code points (`U+D800` to `U+DFFF`).
/// - Tuples: Yields all possible values of the tuple with 1 to 16 elements, in a lexicographic
/// ordering (as `std::cmp::Ord` does), provided that all elements implement `Enumerable`.
/// - `()`: Yields the unit value `()`.
///
/// ## Guarantees and Requirements
///
/// It is guaranteed that:
/// - The derived implementations will enumerate over all possible variants of an enum in the order
/// they are declared. The only exception is variants with fields of uninhabited types (e.g. empty
/// types), which will be skipped.
/// - The derived implementations will yield all possible values of a struct (or a variant with some
/// fields of an enum) in a lexicographic ordering based on the top-to-bottom declaration order of
/// the fields, as built-in implementations for tuples do.
///
/// It is **NOT** guaranteed that:
/// - The derived and the built-in implementations will return a specific type of [`Iterator`] as
/// enumerators.
///
///   Do **NOT** rely on the specific type of the enumerator provided by an `Enumerable` type,
/// unless you are using `#[enumerator(...)]` and knowing that `#[derive(Enumerable)]` will generate
/// an enumerator type, use `<T as Enumerable>::Enumerator` instead in all other cases.
///
/// It is **REQUIRED** that if you are implementing `Enumerable` for a type manually, your
/// enumerator should:
/// - Calling to `enumerator()` should be idempotent, i.e. calling it multiple times should return
/// iterators that yield the same values.
///
/// Failed to meet the requirements will result in unexpected behavior when deriving `Enumerable` on
/// types that contain the type you implemented `Enumerable` for.
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
    /// The number of elements in this enumerable.
    /// If the number exceeds the `usize::MAX`, accessing this constant fails at compile time.
    ///
    /// ## Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    /// let array = [0; u8::ENUMERABLE_SIZE];
    /// ```
    ///
    /// This fails to compile:
    ///
    /// ```compile_fail
    /// use enumerable::Enumerable;
    /// let array = [0; <(usize, usize)>::ENUMERABLE_SIZE];
    /// ```
    const ENUMERABLE_SIZE: usize;
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

            const ENUMERABLE_SIZE: usize =
                if std::mem::size_of::<$ty>() < std::mem::size_of::<usize>() {
                    match (<$ty>::MAX.abs_diff(<$ty>::MIN) as usize).checked_add(1) {
                        Some(size) => size,
                        None => {
                            unreachable!()
                        }
                    }
                } else {
                    panic!(concat!(
                        stringify!($ty),
                        "::ENUMERABLE_SIZE exceeds usize::MAX"
                    ))
                };
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

    const ENUMERABLE_SIZE: usize = 2;
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

    const ENUMERABLE_SIZE: usize = 0x10FFFF + 1;
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

    const ENUMERABLE_SIZE: usize = T::ENUMERABLE_SIZE + 1;
}

/// Implementation of the `Enumerable` trait for `Result<T, E>`, with std::iter::Chain and std::iter::Map.
impl<T, E> Enumerable for Result<T, E>
where
    T: Enumerable,
    E: Enumerable,
{
    type Enumerator = std::iter::Chain<
        std::iter::Map<<T as Enumerable>::Enumerator, fn(T) -> Result<T, E>>,
        std::iter::Map<<E as Enumerable>::Enumerator, fn(E) -> Result<T, E>>,
    >;

    /// This method returns an iterator over all possible values of `Result<T, E>`.
    fn enumerator() -> Self::Enumerator {
        let t: fn(T) -> Result<T, E> = Ok;
        let e: fn(E) -> Result<T, E> = Err;

        <T as Enumerable>::enumerator()
            .map(t)
            .chain(<E as Enumerable>::enumerator().map(e))
    }

    const ENUMERABLE_SIZE: usize = T::ENUMERABLE_SIZE + E::ENUMERABLE_SIZE;
}

pub use enumerable_derive::*;

#[cfg(test)]
#[path = "test.rs"]
mod test;
