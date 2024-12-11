use enumerable::Enumerable;

// Generally, you don't need to implement `Enumerable` manually for your types. For enums (and
// structs, as we'll see later), you can derive it using `#[derive(Enumerable)]`, as long as the
// types of the fields (if any) also implement `Enumerable`.
//
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
    // Since `SomeEnum` has 4 variants, and no fields, it has 4 possible values, therefore, the
    // `ENUMERABLE_SIZE` constant will be 4.
    println!(
        "printing all possible values of SomeEnum ({} in total):",
        SomeEnum::ENUMERABLE_SIZE
    );
    // The output will be: "A\nB\nC\nD\n"
    for value in SomeEnum::enumerator() {
        println!("{:?}", value);
    }
    println!();

    // Empty enums have no possible values, so the `ENUMERABLE_SIZE` constant will be 0.
    println!(
        "number of possible values of EmptyEnum: {:?}\n",
        EmptyEnum::ENUMERABLE_SIZE
    );

    // `OtherEnum` has 6 variants, the number of possible values will be the sum of the possible
    // values of each variant. In this case, it will be:
    //
    // 1 (Z) + 2 (Y) + 4 (X) + 0 (W) + 2 * 2 (V) + 1 (U) = 12
    println!(
        "printing all possible values of OtherEnum ({} in total):",
        OtherEnum::ENUMERABLE_SIZE
    );
    for value in OtherEnum::enumerator() {
        println!("{:?}", value);
    }
    println!()
}
