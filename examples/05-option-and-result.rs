use enumerable::Enumerable;

// Some very useful and common types, like `Option` and `Result`, are actually enums. The
// `Enumerable` trait is implemented for them as well, just like the derived implementation for
// custom enums.

// A custom enum for demonstration purposes.
#[derive(Copy, Clone, Enumerable, Debug)]
enum EnumA {
    A,
    B,
    C,
}

// Another custom enum for demonstration purposes.
#[derive(Copy, Clone, Enumerable, Debug)]
enum EnumB {
    Y,
    Z,
}

fn main() {
    // For `Option`s, the built-in implementation will yield `None` and `Some(value)` for all
    // possible values of `value`.
    //
    // The number of possible values of `Option<T>` is 1 + the number of possible values of `T`.
    println!(
        "printing all possible values of Option<EnumA> ({} in total):",
        Option::<EnumA>::ENUMERABLE_SIZE
    );
    for value in Option::<EnumA>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "printing all possible values of Option<Option<EnumB>> ({} in total):",
        Option::<Option<EnumB>>::ENUMERABLE_SIZE
    );
    for value in Option::<Option<EnumB>>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // For `Result`s, the built-in implementation will yield `Ok(value)` and `Err(err)` for all
    // possible values of `value` and `err`.
    //
    // The number of possible values of `Result<T, E>` is the sum of the number of possible values
    // of `T` and `E`.
    println!(
        "printing all possible values of Result<EnumA, EnumB> ({} in total):",
        Result::<EnumA, EnumB>::ENUMERABLE_SIZE
    );
    for value in Result::<EnumA, EnumB>::enumerator() {
        println!("{:?}", value);
    }
}
