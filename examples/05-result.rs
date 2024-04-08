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
    println!("printing all possible values of Result<EnumA, EnumB>:");
    for value in Result::<EnumA, EnumB>::enumerator() {
        println!("{:?}", value);
    }
}
