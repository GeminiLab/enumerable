use enumerable::Enumerable;

#[derive(Copy, Clone, Enumerable, Debug)]
enum FieldA {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Enumerable, Debug)]
enum FieldB {
    X,
    Y,
    Z,
}

#[derive(Copy, Clone, Enumerable, Debug)]
struct Struct {
    a: FieldA,
    b: FieldB,
}

pub fn main() {
    println!("printing all possible values of Struct:");
    for value in Struct::enumerator() {
        println!("{:?}", value);
    }
}
