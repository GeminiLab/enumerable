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
    println!("printing all possible values of Option<EnumA>:");
    for value in Option::<EnumA>::enumerator() {
        println!("{:?}", value);
    }

    println!("printing all possible values of Option<Option<EnumB>>:");
    for value in Option::<Option<EnumB>>::enumerator() {
        println!("{:?}", value);
    }
}
