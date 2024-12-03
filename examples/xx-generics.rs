use enumerable::Enumerable;

#[derive(Clone, Copy, Debug, Enumerable)]
enum E {
    A,
    B,
    C,
}

#[derive(Clone, Copy, Debug, Enumerable)]
struct G<T, U> {
    t: T,
    u: U,
}

#[derive(Clone, Copy, Debug, Enumerable)]
enum EE<T, U = T> {
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

// All possible values of G<bool, E>:
// G { t: false, u: A }
// G { t: false, u: B }
// G { t: false, u: C }
// G { t: true, u: A }
// G { t: true, u: B }
// G { t: true, u: C }
// All possible values of G<E, bool>:
// G { t: A, u: false }
// G { t: A, u: true }
// G { t: B, u: false }
// G { t: B, u: true }
// G { t: C, u: false }
// G { t: C, u: true }
// All possible values of EE<bool>:
// A(false)
// A(true)
// B(false)
// B(true)
// C((false, false))
// C((false, true))
// C((true, false))
// C((true, true))
// All possible values of EE<bool, E>:
// A(false)
// A(true)
// B(A)
// B(B)
// B(C)
// C((false, A))
// C((false, B))
// C((false, C))
// C((true, A))
// C((true, B))
// C((true, C))
