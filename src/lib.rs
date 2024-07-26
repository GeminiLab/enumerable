#![doc = include_str!("./CRATE_DOC.md")]

/// `Enumerable` is a trait for types that can have their possible values enumerated.
///
/// ## Important Items
///
/// - [`fn enumerator() -> Self::Enumerator`](Enumerable::enumerator): Returns an iterator over all
/// possible values of the implementing type.
/// - [`const ENUMERABLE_SIZE_OPTION: Option<usize>`](Enumerable::ENUMERABLE_SIZE_OPTION): The
/// number of possible values of the implementing type, wrapped in `Some`, or `None` if it exceeds
/// [`usize::MAX`].
/// - [`fn enumerable_from_index(index: usize) -> Option<Self>`](Enumerable::enumerable_from_index):
/// Returns the `index`-th possible value of the implementing type to be enumerated.
///
/// For other items, see sections below.
///
/// ## Built-in Implementations
///
/// The following types have built-in implementations of the `Enumerable` trait:
/// - `bool`: Yields `false` and then `true`.
/// - Numeric types: Yields all possible values of the type from the minimum to the maximum one.
/// - [`Option`]: Yields `None` and then `Some(item)` for each possible value of `T`.
/// - [`Result`]: Yields `Ok(item)` for each possible value of `T` and then `Err(error)` for each
/// possible value of `E`.
/// - `char`: Yields all possible Unicode scalar values, i.e. all code points ranging from `U+0000`
///  to `U+10FFFF`, excluding the surrogate code points (`U+D800` to `U+DFFF`).
/// - Tuples: Yields all possible values of the tuple with 1 to 16 elements, in a lexicographic
/// ordering (as `core::cmp::Ord` does), provided that all elements implement `Enumerable`.
/// - `()`: Yields the unit value `()`.
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
/// In most cases, `#[derive(Enumerable)]` will generate a new enumerator type named
/// `<Type>Enumerator` that enumerates all possible values of the type to be derived `<Type>`. It's
/// possible to customize the name of the generated type by using
/// - `#[enumerator = "DesiredEnumeratorName"]`, or
/// - `#[enumerator(DesiredEnumeratorName)]`,
///
/// they are equivalent.
///
/// `#[derive(Enumerable)]` will NOT generate an enumerator type when the type to be derived is
/// - an enum with zero variants,
/// - an enum with no fields, or
/// - a struct with no fields,
///
/// in these cases, the custom enumerator name will be ignored.
///
/// ## Guarantees and Requirements
///
/// It is guaranteed that:
/// - The derived implementations will enumerate over all possible variants of an enum in the order
/// they are declared. The only exception is variants with fields of uninhabited types (e.g. empty
/// enums), which will be skipped.
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
/// - have an idempotent `enumerator()` method, i.e. calling it multiple times should return
/// iterators that yield the same values in the same order.
/// - have an `ENUMERABLE_SIZE_OPTION` constant that matches the number of elements returned by
/// `enumerator()`.
/// - use the default version of `ENUMERABLE_SIZE`, or provide a custom one that matches
/// `ENUMERABLE_SIZE_OPTION`.
///
/// Failed to meet the requirements will result in unexpected behavior when interacting with the
/// derived implementations.
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
    ///
    /// # Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// let mut enumerator = u8::enumerator();
    /// assert_eq!(enumerator.next(), Some(0)); // 0 is the first value
    /// assert_eq!(enumerator.count(), 255);    // 255 more values to go
    /// ```
    fn enumerator() -> Self::Enumerator;

    /// The number of possible values of the implementing type (i.e. the result of
    /// `Self::enumerator().count()`), wrapped in `Some`, or `None` if it exceeds [`usize::MAX`].
    ///
    /// If a `usize` without any wrapper is preferred, use
    /// [`ENUMERABLE_SIZE`](Self::ENUMERABLE_SIZE) instead.
    ///
    /// # Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// assert_eq!(u8::ENUMERABLE_SIZE_OPTION, Some(256usize));
    /// assert_eq!(<(usize, usize)>::ENUMERABLE_SIZE_OPTION, None);
    /// ```
    const ENUMERABLE_SIZE_OPTION: Option<usize>;
    /// The number of possible values of the implementing type (i.e. the result of
    /// `Self::enumerator().count()`). Panics at compile time if it exceeds [`usize::MAX`].
    ///
    /// It's generally unnecessary to provide this constant manually, as a default value is provided
    /// using [`ENUMERABLE_SIZE_OPTION`](Self::ENUMERABLE_SIZE_OPTION).
    ///
    /// # Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// let array = [0; u8::ENUMERABLE_SIZE];
    /// ```
    ///
    /// This fails to compile:
    ///
    /// ```compile_fail
    /// use enumerable::Enumerable;
    ///
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

    /// Returns an iterator over all possible values of the implementing type starting from the
    /// `start`-th (0-based) value.
    ///
    /// It's highly **RECOMMENDED** to override this default implementation whenever possible, as it
    /// is not efficient for most types.
    ///
    /// # Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// let mut enumerator = u8::enumerator_since(10);
    /// assert_eq!(enumerator.next(), Some(10));
    /// assert_eq!(enumerator.next(), Some(11));
    /// ```
    fn enumerator_since(start: usize) -> Self::Enumerator {
        let mut enumerator = Self::enumerator();
        for _ in 0..start {
            enumerator.next();
        }
        enumerator
    }

    /// Returns the `index`-th possible value of the implementing type to be enumerated.
    ///
    /// Like [`enumerator_since`](Enumerable::enumerator_since), it's highly **RECOMMENDED** to
    /// override this default implementation whenever possible.
    ///
    /// # Example
    ///
    /// ```
    /// use enumerable::Enumerable;
    ///
    /// let value = u8::enumerable_from_index(10);
    /// assert_eq!(value, Some(10));
    /// ```
    fn enumerable_from_index(index: usize) -> Option<Self> {
        Self::enumerator_since(index).next()
    }
}

mod impl_built_in;
mod impl_tuple;

pub use enumerable_derive::*;
pub use impl_built_in::*;
pub use impl_tuple::*;

#[cfg(test)]
mod test;
