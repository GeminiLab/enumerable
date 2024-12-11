//! This example tests the `Enumerable` derive macro in a `no_std` environment.

#![no_std]

use enumerable::Enumerable;

#[derive(Copy, Clone, Enumerable)]
pub enum Example {
    First,
    Second,
    Third,
    Fourth,
}

fn main() -> Result<(), ()> {
    // The `ENUMERABLE_SIZE` constant is correctly calculated for the tuple.
    let tuple_size = <(Example, u8, bool, Option<Example>) as Enumerable>::ENUMERABLE_SIZE;
    assert_eq!(tuple_size, 4 * 256 * 2 * 5);

    // All `u8` values are enumerated.
    let u8_sum: usize = u8::enumerator().map(|v| v as usize).sum();
    assert_eq!(u8_sum, 255 * 256 / 2);

    // `enumerator` returns an iterator over values of the correct amount.
    if <Example as Enumerable>::enumerator().count() == <Example as Enumerable>::ENUMERABLE_SIZE {
        Ok(())
    } else {
        Err(())
    }
}
