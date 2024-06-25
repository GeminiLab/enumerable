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

    /// The number of elements in this enumerable wrapped in `Option::Some` if it does not exceed `usize::MAX`, `None` otherwise.
    ///
    /// If a `usize` without any wrapper is preferred, use `ENUMERABLE_SIZE` instead.
    ///
    /// ## Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    /// assert_eq!(u8::ENUMERABLE_SIZE_OPTION, Some(256usize));
    /// assert_eq!(<(usize, usize)>::ENUMERABLE_SIZE_OPTION, None);
    /// ```
    const ENUMERABLE_SIZE_OPTION: Option<usize>;
    /// The number of elements in this enumerable.
    /// If the number exceeds the `usize::MAX`, accessing this constant fails at compile time.
    ///
    /// It's generally unnecessary to provide this constant manually, as a default value is provided using `ENUMERABLE_SIZE_OPTION`.
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
    const ENUMERABLE_SIZE: usize = {
        match Self::ENUMERABLE_SIZE_OPTION {
            Some(size) => size,
            None => {
                panic!("cannot evaluate Enumerable::ENUMERABLE_SIZE because it exceeds usize::MAX")
            }
        }
    };
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

            const ENUMERABLE_SIZE_OPTION: Option<usize> = {
                if std::mem::size_of::<$ty>() < std::mem::size_of::<usize>() {
                    match (<$ty>::MAX.abs_diff(<$ty>::MIN) as usize).checked_add(1) {
                        Some(size) => Some(size),
                        None => {
                            unreachable!()
                        }
                    }
                } else {
                    None
                }
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

/// This is an implementation of the `Enumerable` trait for `bool`.
impl Enumerable for bool {
    type Enumerator = std::iter::Copied<std::slice::Iter<'static, bool>>;

    /// This method returns an iterator over all possible values of `bool`.
    fn enumerator() -> Self::Enumerator {
        const ALL_VARIANTS: &[bool; 2] = &[false, true];

        return ALL_VARIANTS.iter().copied();
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = Some(2);
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

    const ENUMERABLE_SIZE_OPTION: Option<usize> =
        Some((0xD7FF - 0x0 + 1) + (0x10FFFF - 0xE000 + 1));
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

    const ENUMERABLE_SIZE_OPTION: Option<usize> = {
        match <T as Enumerable>::ENUMERABLE_SIZE_OPTION {
            Some(size) => size.checked_add(1),
            None => None,
        }
    };
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

    const ENUMERABLE_SIZE_OPTION: Option<usize> = {
        match (
            <T as Enumerable>::ENUMERABLE_SIZE_OPTION,
            <E as Enumerable>::ENUMERABLE_SIZE_OPTION,
        ) {
            (Some(t), Some(e)) => t.checked_add(e),
            _ => None,
        }
    };
}

pub use enumerable_derive::*;

#[cfg(test)]
#[path = "test.rs"]
mod test;
