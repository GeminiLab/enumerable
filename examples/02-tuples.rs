use enumerable::Enumerable;

fn main() {
    // `Enumerable` is also implemented for other useful types, like tuples.
    //
    // Tuples with 0 to 16 elements are supported. The built-in implementation will yield all
    // possible values of the tuple in a lexicographic order. For example, for a tuple
    // `(bool, bool)`, the possible values will be:
    //
    // - (false, false)
    // - (false, true)
    // - (true, false)
    // - (true, true)

    // A tuple with no elements is equivalent to the unit type `()`, with only one possible value.
    println!(
        "printing all possible values of () ({} in total):",
        <()>::ENUMERABLE_SIZE
    );
    for value in <()>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // A tuple with one element is equivalent to the element itself.
    println!(
        "printing all possible values of (bool,) ({} in total):",
        <(bool,)>::ENUMERABLE_SIZE
    );
    for value in <(bool,)>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // A tuple with more elements.
    println!(
        "printing all possible values of (bool, bool, bool) ({} in total):",
        <(bool, bool, bool)>::ENUMERABLE_SIZE
    );
    for value in <(bool, bool, bool)>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // The number of possible values of a tuple is the product of the number of possible values of
    // each element. For example, for a tuple `(u16, u16, u16, u16)`, the number of possible values
    // will be 2^16 * 2^16 * 2^16 * 2^16 = 2^64, which exceeds `usize::MAX`.
    println!(
        "does the number of possible (u16, u16, u16, u16) values exceed usize::MAX? {}",
        <(u16, u16, u16, u16)>::ENUMERABLE_SIZE_OPTION.is_none()
    );
}
