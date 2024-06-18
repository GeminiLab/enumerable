use enumerable1::Enumerable as E;

#[derive(Copy, Clone, E)]
#[enumerator(Ehhhhh)]
struct A {
    a: u8,
}

fn main() {
    println!("Hello, world!");
    println!("Number of possible values of A: {:?}", <A as E>::enumerator().count());
}
