<!-- markdownlint-disable-next-line MD041 : this markdown file is intended to be included in `lib.rs` -->
Enumerate all possible values of a type.

[`Enumerable`](trait.Enumerable.html) is a trait used for enumerating all possible values of a type. Calling [`enumerator`](trait.Enumerable.html#tymethod.enumerator) on a `Enumerable` type will return an iterator that yields all possible values of that type.

```rust
use enumerable::Enumerable;

// The output will be:
// 0
// 1
// ...
// 255
for value in u8::enumerator() {
    println!("{}", value);
}
```

`Enumerable` is implemented for most primitive types and some standard library types. You can also derive `Enumerable` for your own types by `#[derive(Enumerable)]`.

```rust
use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable)]
enum Food {
    Apple,
    Banana,
    Coffee { with_milk: bool },
}

// The output will be:
// None
// Some(Apple)
// Some(Banana)
// Some(Coffee { with_milk: false })
// Some(Coffee { with_milk: true })
for value in <Option<Food> as Enumerable>::enumerator() {
    println!("{:?}", value);
}
```

See the documentation of [`Enumerable`](trait.Enumerable.html) for more details.
