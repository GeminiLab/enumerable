use std::{fmt::Debug, marker::PhantomData};

use enumerable::Enumerable;

// In the last example, we saw how `Enumerable` works on `Option` and `Result`. You may have noticed
// that, unlike example enums before, `Option` and `Result` are generic types.
//
// Yes, `Enumerable` works with generics as well! You can derive `Enumerable` for structs and enums
// with generics, and the derived implementation will yield all possible values of the type, just
// like before.
//
// However, only type parameters are allowed, and lifetime and const parameters are not supported.
// Lifetimes are not supported because they are always associated with references, and references
// are inherently not `Enumerable`. Const parameters are not supported because implementing
// `Enumerable` for const generics is too complex, compared to the benefits it would bring.

// A simple enum for demonstration purposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enumerable)]
enum EnumA {
    A,
    B,
    C,
}

// Another simple enum for demonstration purposes.
//
// Note that `where` clauses can actually be used even there are no generic parameters, and they
// can be totally empty. It's tested here to make sure the derive macro can handle it correctly.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enumerable)]
enum EnumB where {
    Y,
    Z,
}

// A simplest struct with generics. No bounds are specified. No default parameters are provided. No
// where clauses are used.
#[derive(Clone, Copy, Debug, Enumerable)]
struct GenericStruct<T, U> {
    t: T,
    u: U,
}

// A more complex enum with generics. Bounds are specified. Default parameters are provided. Where
// clauses are also used. Also, the generic parameter `V` is used "indirectly" in the enum variant
// `C`.
#[derive(Clone, Copy, Debug, Enumerable)]
enum GenericEnum<T: PartialOrd + Debug, U: PartialEq = T, V: PartialEq<T> = T>
where
    T: Enumerable + Eq,
    U: Eq,
{
    A(T),
    B(U),
    C(Result<(T, U), (bool, V)>),
}

fn main() {
    // Test `GenericStruct` with different type parameters, and print all possible values of the
    // types.
    println!(
        "All possible values of GenericStruct<bool, EnumA> ({} in total):",
        GenericStruct::<bool, EnumA>::ENUMERABLE_SIZE
    );
    for gs in GenericStruct::<bool, EnumA>::enumerator() {
        println!("{:?}", gs);
    }
    println!();

    println!(
        "All possible values of GenericStruct<EnumB, bool> ({} in total):",
        GenericStruct::<EnumB, bool>::ENUMERABLE_SIZE
    );
    for gs in GenericStruct::<EnumB, bool>::enumerator() {
        println!("{:?}", gs);
    }
    println!();

    // Test `GenericEnum` with different number of type parameters, check if the derived
    // implementation works with different bounds and default parameters.
    //
    // First, test `GenericEnum` with one type parameter
    println!(
        "All possible values of GenericEnum<bool> ({} in total):",
        GenericEnum::<bool>::ENUMERABLE_SIZE
    );
    for ge in GenericEnum::<bool>::enumerator() {
        println!("{:?}", ge);
    }

    // Then, test `GenericEnum` with two type parameters
    println!(
        "All possible values of GenericEnum<bool, EnumA> ({} in total):",
        GenericEnum::<bool, EnumA>::ENUMERABLE_SIZE
    );
    for ge in GenericEnum::<bool, EnumA>::enumerator() {
        println!("{:?}", ge);
    }

    // Finally, test `GenericEnum` with three type parameters
    println!(
        "All possible values of GenericEnum<EnumB, bool, EnumB> ({} in total):",
        GenericEnum::<EnumB, bool, EnumB>::ENUMERABLE_SIZE
    );
    for ge in GenericEnum::<EnumB, bool, EnumB>::enumerator() {
        println!("{:?}", ge);
    }
}
