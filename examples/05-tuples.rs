use enumerable::Enumerable;

fn main() {
    // Tuples with 0 to 16 elements are supported.
    // The built-in implementation will yield all possible values of the tuple in a lexicographic
    // order.
    println!(
        "printing all possible values of () ({} in total):",
        <()>::ENUMERABLE_SIZE
    );
    for value in <()>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "printing all possible values of (bool,) ({} in total):",
        <(bool,)>::ENUMERABLE_SIZE
    );
    for value in <(bool,)>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "printing all possible values of (bool, bool, bool) ({} in total):",
        <(bool, bool, bool)>::ENUMERABLE_SIZE
    );
    for value in <(bool, bool, bool)>::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "does the number of possible (u16, u16, u16, u16) values exceed usize::MAX? {}",
        <(u16, u16, u16, u16)>::ENUMERABLE_SIZE_OPTION.is_none()
    );
}
