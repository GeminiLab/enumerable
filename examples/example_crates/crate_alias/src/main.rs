use ::enumerable1::Enumerable as E;

/// A conflicting trait. Used here to test if `#[derive(E)]` can find the correct trait.
mod enumerable1 {
    pub trait Enumerable {
        fn wow(&self) -> u8;
    }
}

#[derive(Copy, Clone, E)]
#[enumerator(Ehhhhh)]
struct A {
    a: u8,
}

fn main() {
    println!("Hello, world!");
    // does the `#[enumerator(Ehhhhh)]` attribute work?
    let mut enumerator: Ehhhhh = <A as E>::enumerator();
    println!("Number of possible values of A: {:?}", enumerator.count());
}
