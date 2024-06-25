use enumerable::Enumerable;

pub fn main() {
    // To enumerate all possible values of a type implementing the `Enumerable` trait, call
    // `Enumerator::enumerator()`, which will give you an iterator.
    //
    // This crate provides implementations for most primitive types, namely
    // - i8, i16, i32, i64, i128, isize
    // - u8, u16, u32, u64, u128, usize
    // - char
    // - bool
    // - ()
    //
    // For example, to list all possible values of `bool`, you can do:
    println!(
        "all possible values of bool         : {}",
        bool::enumerator()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    // You can also use `<T as Enumerable>::enumerator()` explicitly:
    println!(
        "first 10 possible values of u8      : {}",
        <u8 as Enumerable>::enumerator()
            .take(10)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    // `Enumerable` also provides an associated constant named `ENUMERABLE_SIZE` which gives the
    // number of possible values of the type.
    println!(
        "count of the possible values of u8  : {}",
        u8::ENUMERABLE_SIZE
    );
    // which is equivalent but more efficient than counting the iterator:
    println!(
        "counting it in a inefficient way    : {}",
        u8::enumerator().count()
    );
    // However, if the number of possible values exceeds `usize::MAX`, `ENUMERABLE_SIZE` will panic
    // at compile time. If that's not the desired behavior, use `ENUMERABLE_SIZE_OPTION` instead,
    // which will give you a `None` in that case.
    println!(
        "count of the possible values of i32 : {:?}",
        i32::ENUMERABLE_SIZE_OPTION
    );
    println!(
        "count of the possible values of i64 : {:?}",
        i64::ENUMERABLE_SIZE_OPTION
    );
    // `()` is also implemented, and it has only one possible value, `()`.
    println!(
        "all possible values of ()           : {}",
        <() as Enumerable>::enumerator()
            .map(|u| format!("{:?}", u))
            .collect::<Vec<_>>()
            .join(", ")
    );
    // `Enumerable::enumerator()` returns a iterator, so you can use all iterator methods on it.
    println!(
        "i32 from smallest, step 16, skip 10 : {}, ...",
        i32::enumerator()
            .step_by(16)
            .skip(10)
            .take(5)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
