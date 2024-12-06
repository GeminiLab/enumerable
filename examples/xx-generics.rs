use enumerable::Enumerable;

// Where clauses with no predicates are allowed in Rust, so we will test whether
// the derive macro can handle it.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Enumerable)]
enum E where { 
    A,
    B,
    C,
}

// Test where clauses without predicates in a struct with generics.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Enumerable)]
struct G<T, U> where {
    t: T,
    u: U,
}

// Test a generic enum with complex bounds.
#[derive(Clone, Copy, Debug, Enumerable)]
enum EE<T: PartialOrd, U: PartialEq = T>
where
    T: Enumerable,
{
    A(T),
    B(U),
    C((T, U), (U, T)),
}

fn main() {
    println!(
        "All possible values of G<bool, E> ({} in total):",
        G::<bool, E>::ENUMERABLE_SIZE
    );
    for g in G::<bool, E>::enumerator() {
        println!("{:?}", g);
    }

    println!(
        "All possible values of G<E, bool> ({} in total):",
        G::<E, bool>::ENUMERABLE_SIZE
    );
    for g in G::<E, bool>::enumerator() {
        println!("{:?}", g);
    }

    println!(
        "All possible values of EE<bool> ({} in total):",
        EE::<bool>::ENUMERABLE_SIZE
    );
    for ee in EE::<bool>::enumerator() {
        println!("{:?}", ee);
    }

    println!(
        "All possible values of EE<bool, E> ({} in total):",
        EE::<bool, E>::ENUMERABLE_SIZE
    );
    for ee in EE::<bool, E>::enumerator() {
        println!("{:?}", ee);
    }
}
