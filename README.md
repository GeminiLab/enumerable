# `enumerable`

> Enumerate all possible values of a type.

## Guide

```rust
use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable)]
enum Food {
    Apple,
    Banana,
    Carrot,
    Donut,
}

#[derive(Debug, Copy, Clone, Enumerable)]
#[allow(dead_code)]
struct Meal {
    alice_eats: Food,
    bob_eats: Option<Food>,
    at_home: bool,
}

fn main() {
    for meal in Meal::enumerator() {
        println!("{:?}", meal);
    }
}
```

See the [documentation](https://docs.rs/enumerable) for more information.

See the [examples](./examples) for more examples.
