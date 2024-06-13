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
#[allow(dead_code)]
struct Struct {
    a: FieldA,
    b: FieldB,
}

pub fn main() {
    println!("printing all possible values of Struct:");
    for value in Struct::enumerator() {
        println!("{:?}", value);
    }

    println!("printing all possible values of (FieldA, FieldB):");
    for value in <(FieldA, FieldB) as Enumerable>::enumerator() {
        println!("{:?}", value);
    }
}
