use enumerable::Enumerable;

#[derive(Copy, Clone, Enumerable, Debug)]
enum EnumA {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Enumerable, Debug)]
enum EnumB {
    Y,
    Z,
}

fn main() {
    // The `Option` and `Result` types are also supported.
    // - For `Option`, the built-in implementation will yield `None` and `Some(value)` for all
    // possible values of `value`.
    // - For `Result`, the built-in implementation will yield `Ok(value)` and `Err(value)` for all
    // possible values of `value`.
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

    println!(
        "printing all possible values of Result<EnumA, EnumB> ({} in total):",
        Result::<EnumA, EnumB>::ENUMERABLE_SIZE
    );
    for value in Result::<EnumA, EnumB>::enumerator() {
        println!("{:?}", value);
    }
}
