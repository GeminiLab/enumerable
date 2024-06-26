use crate::Enumerable;

/// Macro to implement the `Enumerable` trait for a numeric type.
macro_rules! impl_enumerable_for_numeric_type {
    ($ty:ty) => {
        #[automatically_derived]
        impl Enumerable for $ty {
            type Enumerator = core::ops::RangeInclusive<$ty>;

            /// Returns an iterator over all possible values of this type.
            fn enumerator() -> Self::Enumerator {
                <$ty>::MIN..=<$ty>::MAX
            }

            const ENUMERABLE_SIZE_OPTION: Option<usize> = {
                if core::mem::size_of::<$ty>() < core::mem::size_of::<usize>() {
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
    type Enumerator = core::iter::Copied<core::slice::Iter<'static, bool>>;

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
        core::iter::Chain<core::ops::RangeInclusive<char>, core::ops::RangeInclusive<char>>;

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

/// Implementation of the `Enumerable` trait for `Result<T, E>`, with core::iter::Chain and core::iter::Map.
impl<T, E> Enumerable for Result<T, E>
where
    T: Enumerable,
    E: Enumerable,
{
    type Enumerator = core::iter::Chain<
        core::iter::Map<<T as Enumerable>::Enumerator, fn(T) -> Result<T, E>>,
        core::iter::Map<<E as Enumerable>::Enumerator, fn(E) -> Result<T, E>>,
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
