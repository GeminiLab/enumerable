use enumerable::Enumerable;

// As mentioned in the previous example, you can also derive `Enumerable` for structs.
// Here we use two simple enums as example fields.
#[derive(Copy, Clone, Enumerable, Debug)]
enum FieldA {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Enumerable, Debug)]
enum FieldB {
    X,
    Y(bool),
    Z(bool, bool),
}

// The derived implementation will yield all possible values of the struct in a lexicographic order.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
struct Struct {
    a: FieldA,
    b: FieldB,
}

// Tuple-like structs are also supported.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
struct TupleLikeStruct(FieldA, FieldB);

// Unit structs are also supported.
#[derive(Copy, Clone, Enumerable, Debug)]
struct UnitStruct;

// Note that structs are product types, so any uninhabited type in a struct will make the struct
// uninhabited.
#[derive(Copy, Clone, Enumerable, Debug)]
enum EmptyEnum {}

#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(unused_variables)]
struct StructWithEmptyEnum {
    a: bool,
    b: bool,
    empty: EmptyEnum,
}

pub fn main() {
    println!(
        "printing all possible values of Struct ({} in total):",
        Struct::ENUMERABLE_SIZE
    );
    for value in Struct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "printing all possible values of TupleLikeStruct ({} in total):",
        TupleLikeStruct::ENUMERABLE_SIZE
    );
    for value in TupleLikeStruct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "printing all possible values of UnitStruct ({} in total):",
        UnitStruct::ENUMERABLE_SIZE
    );
    for value in UnitStruct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "number of possible values of structs with empty fields: {:?}\n",
        StructWithEmptyEnum::ENUMERABLE_SIZE
    );
}
