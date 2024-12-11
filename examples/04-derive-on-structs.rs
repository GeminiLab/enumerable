use enumerable::Enumerable;

// As mentioned in the previous example, you can also derive `Enumerable` for structs.
//
// Here we use two simple enums as example fields. One of them doesn't have fields,
#[derive(Copy, Clone, Enumerable, Debug)]
enum FieldA {
    A,
    B,
    C,
}

// and the other has fields.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
enum FieldB {
    X,
    Y(bool),
    Z(bool, bool),
}

// The derived implementation for structs will yield all possible values of the struct in a
// lexicographic order, i.e., enumerating the last field first, and the first field last.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
struct Struct {
    a: FieldA,
    b: FieldB,
}

// Tuple-like structs are also supported. The lexicographic order is also used here.hhh
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
struct TupleLikeStruct(FieldA, FieldB);

// Unit structs are also supported. They behave like `()`, with only one possible value.
#[derive(Copy, Clone, Enumerable, Debug)]
struct UnitStruct;

// Note that structs are product types, so any uninhabited type in a struct will make the struct
// uninhabited.
//
// For example, if we have a struct with an empty enum, like this:
#[derive(Copy, Clone, Enumerable, Debug)]
enum EmptyEnum {}

// The struct will also be uninhabited, and the derived implementation will not yield any value.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(unused_variables)]
struct StructWithEmptyEnum {
    a: bool,
    b: bool,
    empty: EmptyEnum,
}

pub fn main() {
    // `FieldA` has 3 possible values, and `FieldB` has 1 + 2 + 2 * 2 = 7 possible values.
    //
    // Therefore, `Struct` will have 3 * 7 = 21 possible values.
    println!(
        "printing all possible values of Struct ({} in total):",
        Struct::ENUMERABLE_SIZE
    );
    for value in Struct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // For `TupleLikeStruct`, the number of possible values is the same as `Struct`, as they have
    // the same fields.
    println!(
        "printing all possible values of TupleLikeStruct ({} in total):",
        TupleLikeStruct::ENUMERABLE_SIZE
    );
    for value in TupleLikeStruct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // `UnitStruct` has only one possible value.
    println!(
        "printing all possible values of UnitStruct ({} in total):",
        UnitStruct::ENUMERABLE_SIZE
    );
    for value in UnitStruct::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // `StructWithEmptyEnum` has no possible values, as it contains an uninhabited type.
    println!(
        "number of possible values of structs with empty fields: {:?}\n",
        StructWithEmptyEnum::ENUMERABLE_SIZE
    );
}
