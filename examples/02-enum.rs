use enumerable::Enumerable;

#[derive(Copy, Clone, Enumerable, Debug)]
enum SomeEnum {
    A,
    B(bool),
    C,
    D,
}

#[derive(Copy, Clone, Enumerable, Debug)]
enum OtherEnum {
    Z,
    Y,
    X,
    W,
    V,
    U,
}

pub fn main() {
    println!(
        "printing all possible values of SomeEnum ({} in total):",
        <SomeEnum as Enumerable>::ENUMERABLE_SIZE
    );
    for value in SomeEnum::enumerator() {
        println!("{:?}", value);
    }

    println!("printing all possible values of OtherEnum:");
    for value in OtherEnum::enumerator() {
        println!("{:?}", value);
    }
}
