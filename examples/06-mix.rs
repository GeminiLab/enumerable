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

#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
struct Struct {
    a: EnumA,
    flag: bool,
}

fn main() {
    println!("printing all possible values of Result<Struct, Option<EnumB>>:");
    for value in Result::<Struct, Option<EnumB>>::enumerator() {
        println!("{:?}", value);
    }
}
