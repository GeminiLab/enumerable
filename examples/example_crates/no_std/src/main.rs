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
    if <Example as Enumerable>::enumerator().count() == <Example as Enumerable>::ENUMERABLE_SIZE {
        Ok(())
    } else {
        Err(())
    }
}
