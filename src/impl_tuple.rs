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

/// This macro generates a fragment of the body of the `calculate_next` method for a tuple enumerator.
///
/// Rust macros are literally magic! This macro is a little bit tricky, but it's not really that complicated.
macro_rules! calculate_next_fn_body_impl {
    (@ $tt:tt # $self:ident) => {
        $tt
    };
    (($var0:ident: $gen0:ident <- $enum_field0:ident) $(($var:ident: $gen:ident <- $enum_field:ident))* @ $tt:tt # $self:ident) => {
        calculate_next_fn_body_impl!($(($var: $gen <- $enum_field))* @ {
            *$var0 = match $self.$enum_field0.next() {
                Some(value) => value,
                None => {
                    $tt

                    $self.$enum_field0 = <$gen0 as Enumerable>::enumerator();
                    $self.$enum_field0.next().unwrap()
                }
            }
        } # $self)
    };
}

/// This macro generates the body of the `calculate_next` method for a tuple enumerator.
///
/// The `calculate_next` method advances the enumerator of the last element in the tuple. If the enumerator is exhausted, it resets the enumerator and advances the previous element's enumerator. This process continues until the first element's enumerator is exhausted, at which point the enumerator is exhausted.
///
/// For example, for a tuple `(A, B, C)`, the generated code will look like this:
/// ```rust,ignore
/// fn calculate_next(&mut self) {
///     if let Some((a, b, c)) = &mut self.calculated_next { // If `None`, the enumerator is exhausted.
///         *c = match self.c_enumerator.next() {
///             Some(value) => value,
///             None => {
///                 *b = match self.b_enumerator.next() {
///                     Some(value) => value,
///                     None => {
///                         *a = match self.a_enumerator.next() {
///                             Some(value) => value,
///                             None => {
///                                 // The enumerator is exhausted.
///                                 self.calculated_next = None;
///                                 return;
///                             }
///                         }
///
///                         // Reset the `b` enumerator.
///                         self.b_enumerator = <B as Enumerable>::enumerator();
///                         self.b_enumerator.next().unwrap()
///                     }
///                 }
///
///                 // Reset the `c` enumerator.
///                 self.c_enumerator = <C as Enumerable>::enumerator();
///                 self.c_enumerator.next().unwrap()
///             }
///         }
///     }
/// }
/// ```
macro_rules! calculate_next_fn_body {
    ($($gen:ident),+ # $self:ident) => {
        paste::paste! {
            if let Some(($([< $gen:lower >]),+)) = &mut $self.calculated_next {
                calculate_next_fn_body_impl!($(([<$gen:lower>]: $gen <- [< $gen:lower _enumerator >]))+ @ { $self.calculated_next = None; return; } # $self)
            }
        }
    };
}

/// This macro implements `Enumerable` for tuples with a given number of elements.
///
/// Details:
/// 1. This macro accepts these arguments: the number of elements in the tuple (`$count`) and the list of the tuple's generic parameters (`$gen`). The number is used to generate the name of the enumerator struct only, and the generic parameters are used elsewhere.
/// 2. This macro generates a struct named `[< Tuple $count Enumerator >]`(`TupleXEnumerator`) where `$count` is the number of elements in the tuple. This struct is the enumerator for the tuple.
/// 3. The enumerator has a field for each element in the tuple - enumerators for the elements. The field names are the lowercase version of the element's type name followed by `_enumerator`. For example, if the tuple is `(A, B)`, the struct will have fields `a_enumerator` and `b_enumerator`.
/// 4. The enumerator also has a field named `calculated_next` that holds the next tuple to be returned by the enumerator. This field will be `None` if the enumerator has been exhausted. When the enumerator is created, it will be set to the first tuple to be enumerated, or `None` if the list of possible tuples is empty (e.g., if any of the element has no possible values).
/// 5. Whenever the enumerator is asked for the next tuple, it will calculate the next tuple and return the current one.
/// 6. The `calculate_next` method is used to calculate the next tuple. It is generated by the `calculate_next_fn_body` macro, see its documentation for more details.
macro_rules! impl_enumerable_for_tuple {
    ($count:literal, $($gen:ident),+) => {
        paste::paste! {
            #[doc = "Enumerator for tuples with " $count " elements."]
            pub struct [< Tuple $count Enumerator >]<$($gen),+>
            where
                $($gen: Enumerable,)+
            {
                $(
                    [< $gen:lower _enumerator >]: <$gen as Enumerable>::Enumerator,
                )+
                calculated_next: Option<($($gen, )+)>,
            }

            #[automatically_derived]
            impl<$($gen),+> Iterator for [< Tuple $count Enumerator >]<$($gen),+>
            where
                $($gen: Enumerable,)+
            {
                type Item = ($($gen, )+);

                fn next(&mut self) -> Option<Self::Item> {
                    let result = self.calculated_next;
                    self.calculate_next();

                    result
                }
            }

            impl<$($gen),+> [< Tuple $count Enumerator >]<$($gen),+>
            where
                $($gen: Enumerable,)+
            {
                #[doc = "Creates a new enumerator for tuples with " $count " elements."]
                pub fn new() -> Self {
                    $(
                        let mut [< $gen:lower _enumerator >] = <$gen as Enumerable>::enumerator();
                        let [< $gen:lower >] = [< $gen:lower _enumerator >].next();
                    )+

                    let calculated_next = if false $(|| [< $gen:lower >].is_none())+ {
                        None
                    } else {
                        Some(($([< $gen:lower >].unwrap()),+))
                    };

                    Self {
                        $(
                            [< $gen:lower _enumerator >],
                        )+
                        calculated_next,
                    }
                }

                #[allow(unreachable_code)]
                fn calculate_next(&mut self) {
                    calculate_next_fn_body!($($gen),+ # self)
                }
            }

            #[automatically_derived]
            impl<$($gen),+> Enumerable for ($($gen, )+)
            where
                $($gen: Enumerable,)+
            {
                type Enumerator = [< Tuple $count Enumerator >]<$($gen),+>;

                fn enumerator() -> Self::Enumerator {
                    Self::Enumerator::new()
                }

                const ENUMERABLE_SIZE_OPTION: Option<usize> = {
                    let size: Option<usize> = Some(1usize);
                    $(
                        let size: Option<usize> = match (size, $gen::ENUMERABLE_SIZE_OPTION) {
                            (Some(size), Some(gen_size)) => size.checked_mul(gen_size),
                            _ => None,
                        };
                    )+
                    size
                };
            }
        }
    };
}

impl_enumerable_for_tuple!(2, A, B);
impl_enumerable_for_tuple!(3, A, B, C);
impl_enumerable_for_tuple!(4, A, B, C, D);
impl_enumerable_for_tuple!(5, A, B, C, D, E);
impl_enumerable_for_tuple!(6, A, B, C, D, E, F);
impl_enumerable_for_tuple!(7, A, B, C, D, E, F, G);
impl_enumerable_for_tuple!(8, A, B, C, D, E, F, G, H);
impl_enumerable_for_tuple!(9, A, B, C, D, E, F, G, H, I);
impl_enumerable_for_tuple!(10, A, B, C, D, E, F, G, H, I, J);
impl_enumerable_for_tuple!(11, A, B, C, D, E, F, G, H, I, J, K);
impl_enumerable_for_tuple!(12, A, B, C, D, E, F, G, H, I, J, K, L);
impl_enumerable_for_tuple!(13, A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_enumerable_for_tuple!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_enumerable_for_tuple!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_enumerable_for_tuple!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
