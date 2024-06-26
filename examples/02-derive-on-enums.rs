use enumerable::Enumerable;

// Generally, you don't need to implement `Enumerable` manually for your types. For enums (and
// structs, as we'll see later), you can derive it using `#[derive(Enumerable)]`, as long as the
// types of the fields (if any) also implement `Enumerable`.
// For enums, the derived implementation will yield all possible values of the enum in the order
// they are declared.
#[derive(Copy, Clone, Enumerable, Debug)]
enum SomeEnum {
    A,
    B,
    C,
    D,
}

// You can also derive `Enumerable` for empty enums. Note that the derived implementation will not
// yield any value, as empty enums are inherently uninhabited.
#[derive(Copy, Clone, Enumerable, Debug)]
enum EmptyEnum {}

// Enums with fields are also supported. The derived implementation will also enumerate all possible
// values of the fields.
#[derive(Copy, Clone, Enumerable, Debug)]
#[allow(dead_code)]
enum OtherEnum {
    Z,
    Y(bool),
    X(SomeEnum),
    W(EmptyEnum),
    V { field_a: bool, field_b: bool },
    U,
}

pub fn main() {
    println!(
        "printing all possible values of SomeEnum ({} in total):",
        SomeEnum::ENUMERABLE_SIZE
    );
    for value in SomeEnum::enumerator() {
        println!("{:?}", value);
    }
    println!();

    println!(
        "number of possible values of EmptyEnum: {:?}\n",
        EmptyEnum::ENUMERABLE_SIZE
    );

    println!(
        "printing all possible values of OtherEnum ({} in total):",
        OtherEnum::ENUMERABLE_SIZE
    );
    for value in OtherEnum::enumerator() {
        println!("{:?}", value);
    }
    println!()
}
