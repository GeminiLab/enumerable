# `enumerable`

> Enumerate all possible values of a type.

See the [examples](./examples) for more examples and a guide on how to use the crate.

See the [documentation](https://docs.rs/enumerable) for detailed information on the crate.

## TL;DR

```rust
use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable)]
#[allow(dead_code)]
enum Food {
    Apple,
    Banana,
    Coffee { with_milk: bool },
}

#[derive(Debug, Copy, Clone, Enumerable)]
#[allow(dead_code)]
struct Meal {
    alice_eats: Food,
    bob_eats: Option<Food>,
    at_home: bool,
}

fn main() {
    println!("There are {} different meals, enumerated as follows:", Meal::ENUMERABLE_SIZE);
    for meal in Meal::enumerator() {
        println!("{:?}", meal);
    }
}
```

The code above will output:

```text
There are 40 different meals, enumerated as follows:
Meal { alice_eats: Apple, bob_eats: None, at_home: false }
Meal { alice_eats: Apple, bob_eats: None, at_home: true }
Meal { alice_eats: Apple, bob_eats: Some(Apple), at_home: false }
Meal { alice_eats: Apple, bob_eats: Some(Apple), at_home: true }
Meal { alice_eats: Apple, bob_eats: Some(Banana), at_home: false }
Meal { alice_eats: Apple, bob_eats: Some(Banana), at_home: true }
Meal { alice_eats: Apple, bob_eats: Some(Coffee { with_milk: false }), at_home: false }
Meal { alice_eats: Apple, bob_eats: Some(Coffee { with_milk: false }), at_home: true }
Meal { alice_eats: Apple, bob_eats: Some(Coffee { with_milk: true }), at_home: false }
Meal { alice_eats: Apple, bob_eats: Some(Coffee { with_milk: true }), at_home: true }
Meal { alice_eats: Banana, bob_eats: None, at_home: false }
...
```

