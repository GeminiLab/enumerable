use enumerable::Enumerable;

pub fn main() {
    println!(
        "all possible values of bool         : {}",
        bool::enumerator()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "count of the possible values of u8  : {}",
        u8::enumerator().count()
    );
    println!(
        "count of the possible values of i16 : {}",
        i16::enumerator().count()
    );
    println!(
        "first 10 possible values of u8      : {}",
        u8::enumerator()
            .take(10)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "first 5 possible values of i32      : {}",
        i32::enumerator()
            .take(5)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
}
