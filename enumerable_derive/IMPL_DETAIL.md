# The details hidden behind the `#[derive(Enumerable)]` macro

The built-in implementations of the `Enumerable` trait are quite simple. For numeric types, just use `RangeInclusive`s. For `()`, `std::iter::once(())` is enough. For `bool`, why not create a const array `[false, true]` and return an iterator of it every time? The standard library has already done a great job for us, and there's no need to create new enumerator types for them.

But implementing `Enumerable` for structs and enums is not that straightforward. There are new types to create, new methods to implement, and many edge cases to consider. Here are some details about how the `Enumerable` trait should be implemented for structs and enums and how `#[derive(Enumerable)]` macro works.

## How to implement `Enumerable` on structs?

Loosely speaking, a struct is a "product type" (similar to tuples, and fields of a variant of an enum, as will be mentioned below), it's a combination of multiple types, its values are constructed by combining values of its fields.

Therefore, the set of all possible values of a struct is the **Cartesian product** of the sets of all possible values of its fields, or in a less formal way, to enumerate all possible values of a struct, we need to enumerate **all possible values of each field** and then enumerate **all possible combinations of them**.

Here comes the first question: in which order should we enumerate the combinations of fields? The answer is not hard to find: **lexicographical order**. Its advantages are:

- It's the most natural way, and it's easy to understand.
- It's the default order of `std::cmp::Ord` and `std::cmp::PartialOrd` traits. Having the result of enumeration ordered is generally a good thing.
- It allows potential lazy evaluation optimizations. For example, if only the first several values are needed, then the unnecessary enumeration of the first fields can be skipped.

Readers familiar with the concept of generators might have already realized that this can be implemented using a generator easily, like:

```rust
#[derive(Clone, Copy)]
struct Example {
    field1: u8,
    field2: bool,
    field3: u16,
}

fn example_enumerator() -> impl Iterator<Item = Example> {
    // Technically, "generators" and `gen` blocks are not exactly the same thing
    // in Rust now, but we don't need to care about this detail here.
    gen {
        for field1 in <u8 as Enumerable>::enumerator() {
            for field2 in <bool as Enumerable>::enumerator() {
                for field3 in <u16 as Enumerable>::enumerator() {
                    yield Example { field1, field2, field3 };
                }
            }
        }
    }
}
```

If you compile the code above with the nightly compiler (as of the time of writing, `1.85.0-nightly (2024-12-01)`) with the feature `gen_blocks` enabled, you will find that it works perfectly. Sadly, there are some problems preventing us from using this approach:

- Generators, as well as `gen` blocks, are still unstable now, and it may take a long time to stabilize. Relying on unstable features is not a good idea.
- The generated iterator may not provide some useful methods like `size_hint`, `count`, etc.
- The generated iterator does not have a explicit type name, which may cause some problems in some cases.

Here comes the second question: if we can't use generators, what should we do to implement `Enumerable` for structs? The simplest way is to "transform" the generator into a normal iterator. First, we need to store all local variables in a struct:

```rust
struct ExampleEnumerator {
    field1_enumerator: <u8 as Enumerable>::Enumerator,
    field2_enumerator: <bool as Enumerable>::Enumerator,
    field3_enumerator: <u16 as Enumerable>::Enumerator,
    field1: u8,
    field2: bool,
    field3: u16,
}
```

Also, we need to store the current state of the generator. But since the generator above has only one `yield` statement, there are only three possible states: 1. The generator has not started yet, 2. The generator has yielded a value, and, 3. The generator has finished. Therefore, it's actually not necessary to store the state of the generator explicitly, with some modifications:

```rust
struct ExampleEnumerator {
    // The enumerators of all fields.
    field1_enumerator: <u8 as Enumerable>::Enumerator,
    field2_enumerator: <bool as Enumerable>::Enumerator,
    field3_enumerator: <u16 as Enumerable>::Enumerator,
    // The value to be yielded at the **next** call of `next`. If the generator
    // has finished, this field will be `None`.
    next: Option<Example>,
}

impl ExampleEnumerator {
    fn new() -> Self {
        let mut field1_enumerator = <u8 as Enumerable>::enumerator();
        let mut field2_enumerator = <bool as Enumerable>::enumerator();
        let mut field3_enumerator = <u16 as Enumerable>::enumerator();

        // We "pre-move" the generator to the state at the first `yield`
        // statement, to skip the state of "not started yet".
        let field1 = field1_enumerator.next();
        let field2 = field2_enumerator.next();
        let field3 = field3_enumerator.next();

        // If any of the fields is `None`, then the field has no possible
        // values, then the struct has no possible values, so the generator has
        // already finished.
        //
        // Of course in this example, none of the fields will be `None`, but in
        // a general case, we need to check this.
        let next = match (field1, field2, field3) {
            (Some(field1), Some(field2), Some(field3)) => Some(Example {
                field1,
                field2,
                field3,
            }),
            _ => None,
        };

        Self { field1_enumerator, field2_enumerator, field3_enumerator, next }
    }

    /// Move the generator to the next state. And store the next value to yield
    /// in the `next` field. It's basically an de-sugared version of the
    /// generator above.
    fn step(&mut self) {
        // If the generator has finished, do nothing.
        if let Some(Example { field1, field2, field3 }) = &mut self.next {
            // This if corresponds to the inner loop in the generator above.
            if let Some(next_field3) = self.field3_enumerator.next() {
                // The inner loop has not finished yet.
                *field3 = next_field3;
            } else {
                // The inner loop has finished. Move to the middle loop.

                // This corresponds to the middle loop in the generator above.
                if let Some(next_field2) = self.field2_enumerator.next() {
                    // The middle loop has not finished yet.
                    *field2 = next_field2;
                } else {
                    // The middle loop has finished. Move to the outer loop.

                    // This corresponds to the outer loop in the generator above.
                    if let Some(next_field1) = self.field1_enumerator.next() {
                        // The outer loop has not finished yet.
                        *field1 = next_field1;
                    } else {
                        // The outer loop has finished. The generator has finished.
                        self.next = None;
                        return;
                    }

                    // Re-enter the middle loop.
                    // Reset the `field2_enumerator` to the initial state.
                    self.field2_enumerator = <bool as Enumerable>::enumerator();
                    // `unwrap` is safe here because we know that the
                    // `field2_enumerator` will always have at least one value.
                    *field2 = self.field2_enumerator.next().unwrap();
                }

                // Re-enter the inner loop.
                // Reset the `field3_enumerator` to the initial state.
                self.field3_enumerator = <u16 as Enumerable>::enumerator();
                // `unwrap` is safe here because we know that the
                // `field3_enumerator` will always have at least one value.
                *field3 = self.field3_enumerator.next().unwrap();
            }
        }
    }
}

impl Iterator for ExampleEnumerator {
    type Item = Example;

    fn next(&mut self) -> Option<Self::Item> {
        // Return the next value to yield.

        // First fetch the value to yield.
        let result = self.next;
        // Then "pre-move" the generator to the next state.
        self.step();
        // Finally, return the value to yield.
        result
    }
}
```

The code above is a little bit long and complex, but it's not that hard to understand with the help of the comments.

The key point here is that, the order of "move to the next state" and "yield the value" is rearranged. When a normal generator is called, it first move the itself to the next state, then yield a value based on the that state. But in this implementation, the generator "pre-moves" itself to the next state, then yield the value based on the previous state, which is evaluated during the previous call (or the initialization, for the first value) and stored in the `next` field. If the generator has finished, the `next` field will be `None`.

The advantage of this approach is that the state of "not started yet" can be skipped, and the state of "finished" can be determined by checking the `next` field (which, in fact, does not need to be checked explicitly). As a result, the state of the generator is not necessary to be stored explicitly.

Clearly, the code above can be generated easily by a procedural macro. To implement `Enumerable` for a struct, we need one more thing: the `ENUMERABLE_SIZE_OPTION` const. Thanks to `usize::checked_mul`, it's easy to calculate it by multiplying the sizes of all fields.

Finally, the implementation of `Enumerable` for a struct looks like this:

```rust
impl Enumerable for Example {
    type Enumerator = ExampleEnumerator;

    fn enumerator() -> Self::Enumerator {
        ExampleEnumerator::new()
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = {
        let size = Some(1usize);
        let size = match (size, <u8 as Enumerable>::ENUMERABLE_SIZE_OPTION) {
            (Some(size), Some(other_size)) => size.checked_mul(other_size),
            _ => None,
        };
        let size = match (size, <bool as Enumerable>::ENUMERABLE_SIZE_OPTION) {
            (Some(size), Some(other_size)) => size.checked_mul(other_size),
            _ => None,
        };
        let size = match (size, <u16 as Enumerable>::ENUMERABLE_SIZE_OPTION) {
            (Some(size), Some(other_size)) => size.checked_mul(other_size),
            _ => None,
        };
        size
    };
}
```

There is one more thing to mention: if the struct is an `EmptyStruct` with no fields, we can just return `core::iter::once(EmptyStruct)` in the `enumerator` method, and set `ENUMERABLE_SIZE_OPTION` to `Some(1)`. It's simpler and more efficient.

## How to implement `Enumerable` on enums?

An enum is a "sum type", its values are constructed by choosing one of its variants and then constructing the value based on the fields of the chosen variant. Therefore, the set of all possible values of an enum is the **union** of all possible values of its variants. So, to enumerate all possible values of an enum, we need to enumerate **all possible values of each variant** and yield them in order.

Unlike structs, where we start directly with general cases, let's take a look at some simpler enums first. For example, if an enum has no fields at all, like:

```rust
#[derive(Clone, Copy)]
enum SimpleEnum {
    Variant1,
    Variant2,
    Variant3,
}
```

It's possible to store all its variants in a const array and return an iterator of it:

```rust
impl Enumerable for SimpleEnum {
    type Enumerator = core::iter::Copied<core::slice::Iter<'static, SimpleEnum>>;

    fn enumerator() -> Self::Enumerator {
        static VARIANTS: [SimpleEnum; 3] = [
            SimpleEnum::Variant1,
            SimpleEnum::Variant2,
            SimpleEnum::Variant3
        ];
        VARIANTS.iter().copied()
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = Some(3);
}
```

If an enum has only no variants, then it is **uninhabited**, and has **no possible values**. In this case, we can just return `core::iter::empty()` in the `enumerator` method, and set `ENUMERABLE_SIZE_OPTION` to `Some(0)`.

```rust
#[derive(Clone, Copy)]
enum UninhabitedEnum {}

impl Enumerable for UninhabitedEnum {
    type Enumerator = core::iter::Empty<UninhabitedEnum>;

    fn enumerator() -> Self::Enumerator {
        core::iter::empty()
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = Some(0);
}
```

But if an enum has fields, it's much more complex:

```rust
#[derive(Clone, Copy)]
enum ComplexEnum {
    Variant1(u8),
    Variant2,
    Variant3(UninhabitedEnum),
    Variant4{ field1: SimpleEnum, field2: SimpleEnum },
}
```

Luckily, we can use the same approach as we used for structs: write a generator, then transform it into a normal iterator. The generator for the enum above looks like this:

```rust
fn enumerate_complex_enum() -> impl Iterator<Item = ComplexEnum> {
    gen {
        // State: BeforeVariant1

        for field1 in <u8 as Enumerable>::enumerator() {
            yield ComplexEnum::Variant1(field1); // State: InVariant1
        }

        // State: BeforeVariant2

        yield ComplexEnum::Variant2; // State: InVariant2

        // State: BeforeVariant3

        for field1 in <UninhabitedEnum as Enumerable>::enumerator() {
            yield ComplexEnum::Variant3(field1); // State: InVariant3
        }

        // State: BeforeVariant4

        for field1 in <SimpleEnum as Enumerable>::enumerator() {
            for field2 in <SimpleEnum as Enumerable>::enumerator() {
                yield ComplexEnum::Variant4{ field1, field2 }; // State: InVariant4
            }
        }

        // State: Finished
    }
}
```

It's longer than the generators for structs, but it's still not hard to understand. Basically, it's the concatenation of the generators for each variant. The states of the generator are labeled with comments. You may find that the `Before<Variant>` states are not really necessary, but they are very helpful, as we will see later. The transformed generator is much longer, take a deep breath, and here it is:

```rust
/// The states of the generator, with all local variables stored inside.
enum ComplexEnumEnumerator {
    BeforeVariant1,
    InVariant1 {
        field1_enumerator: <u8 as Enumerable>::Enumerator,
        field1: u8,
    },
    BeforeVariant2,
    InVariant2,
    BeforeVariant3,
    InVariant3 {
        field1_enumerator: <UninhabitedEnum as Enumerable>::Enumerator,
        field1: UninhabitedEnum,
    },
    BeforeVariant4,
    InVariant4 {
        field1_enumerator: <SimpleEnum as Enumerable>::Enumerator,
        field2_enumerator: <SimpleEnum as Enumerable>::Enumerator,
        field1: SimpleEnum,
        field2: SimpleEnum,
    },
    Finished,
}

impl ComplexEnumEnumerator {
    fn new() -> Self {
        // Start with the first state.
        let mut result = Self::BeforeVariant1;
        // "Pre-move" the generator to the value to be yielded.
        result.step();
        result
    }

    /// Move the generator to the next state. Same as the `step` method of the
    /// generators for structs.
    fn step(&mut self) {
        // The `loop` here is actually just a "label", and the `continue`s in
        // this loop is actually just `goto`s.
        loop {
            match self {
                // The Before<Variant> states. It does almost the same thing as
                // the `new` methods of the generators for structs:
                // - create the enumerators of all fields,
                // - try to move to the state at the first `yield` statement in
                //   this variant, and
                // - if failed, move to the next variant.
                ComplexEnumEnumerator::BeforeVariant1 => {
                    let mut field1_enumerator = <u8 as Enumerable>::enumerator();
                    let field1 = field1_enumerator.next();

                    match field1 {
                        Some(field1) => {
                            *self = ComplexEnumEnumerator::InVariant1 {
                                field1_enumerator,
                                field1,
                            };
                        }
                        _ => {
                            // If this variant has no possible values, then goto
                            // the initializer of the next variant.
                            *self = ComplexEnumEnumerator::BeforeVariant2;
                            continue;
                        }
                    }
                }
                // The In<Variant> states. It does almost the same thing as the
                // `step` methods of the generators for structs:
                // - try to move to the next state, and
                // - if failed, move to the next variant.
                ComplexEnumEnumerator::InVariant1 {
                    field1_enumerator,
                    field1,
                } => {
                    if let Some(next_field1) = field1_enumerator.next() {
                        *field1 = next_field1;
                    } else {
                        // If the possible values of this variant have been
                        // exhausted, then goto the initializer of the next
                        // variant.
                        *self = ComplexEnumEnumerator::BeforeVariant2;
                        continue;
                    }
                }
                // These variants are similar to the first two.
                ComplexEnumEnumerator::BeforeVariant2 => {
                    *self = ComplexEnumEnumerator::InVariant2;
                }
                ComplexEnumEnumerator::InVariant2 => {
                    *self = ComplexEnumEnumerator::BeforeVariant3;
                    continue;
                }
                ComplexEnumEnumerator::BeforeVariant3 => {
                    let mut field1_enumerator = <UninhabitedEnum as Enumerable>::enumerator();
                    let field1 = field1_enumerator.next();

                    #[allow(unreachable_patterns)]
                    match field1 {
                        Some(field1) => {
                            *self = ComplexEnumEnumerator::InVariant3 {
                                field1_enumerator,
                                field1,
                            };
                        }
                        _ => {
                            *self = ComplexEnumEnumerator::BeforeVariant4;
                            continue;
                        }
                    }
                }
                ComplexEnumEnumerator::InVariant3 {
                    field1_enumerator,
                    field1,
                } =>
                {
                    #[allow(unreachable_patterns)]
                    if let Some(next_field1) = field1_enumerator.next() {
                        *field1 = next_field1;
                    } else {
                        *self = ComplexEnumEnumerator::BeforeVariant4;
                        continue;
                    }
                }
                ComplexEnumEnumerator::BeforeVariant4 => {
                    let mut field1_enumerator = <SimpleEnum as Enumerable>::enumerator();
                    let mut field2_enumerator = <SimpleEnum as Enumerable>::enumerator();
                    let field1 = field1_enumerator.next();
                    let field2 = field2_enumerator.next();

                    match (field1, field2) {
                        (Some(field1), Some(field2)) => {
                            *self = ComplexEnumEnumerator::InVariant4 {
                                field1_enumerator,
                                field2_enumerator,
                                field1,
                                field2,
                            };
                        }
                        _ => {
                            *self = ComplexEnumEnumerator::Finished;
                            continue;
                        }
                    }
                }
                ComplexEnumEnumerator::InVariant4 {
                    field1_enumerator,
                    field2_enumerator,
                    field1,
                    field2,
                } => {
                    if let Some(next_field2) = field2_enumerator.next() {
                        *field2 = next_field2;
                    } else {
                        if let Some(next_field1) = field1_enumerator.next() {
                            *field1 = next_field1;
                        } else {
                            *self = ComplexEnumEnumerator::Finished;
                            return;
                        }

                        *field2_enumerator = <SimpleEnum as Enumerable>::enumerator();
                        *field2 = field2_enumerator.next().unwrap();
                    }
                }
                // The explicit `Finished` state, it's absorbing.
                ComplexEnumEnumerator::Finished => {}
            }

            break;
        }
    }

    /// Unlike the generators for structs, where the next value to yield is
    /// stored in the `next` field, the next value to yield here needs to be
    /// extracted from the current variant.
    fn next_to_yield(&self) -> Option<ComplexEnum> {
        match self {
            ComplexEnumEnumerator::InVariant1 { field1, .. } => {
                Some(ComplexEnum::Variant1(*field1))
            }
            ComplexEnumEnumerator::InVariant2 => Some(ComplexEnum::Variant2),
            ComplexEnumEnumerator::InVariant3 { field1, .. } => {
                Some(ComplexEnum::Variant3(*field1))
            }
            ComplexEnumEnumerator::InVariant4 { field1, field2, .. } => {
                Some(ComplexEnum::Variant4 {
                    field1: *field1,
                    field2: *field2,
                })
            }
            ComplexEnumEnumerator::Finished => None,
            _ => unreachable!(),
        }
    }
}

impl Iterator for ComplexEnumEnumerator {
    type Item = ComplexEnum;

    /// Same as the `next` method of the generators for structs. Fetch the value
    /// to yield, move to the next state, and return the value.
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next_to_yield();
        self.step();
        result
    }
}
```

Whew! It's really long, right? But the overall structure is still the same as the generators for structs. The differences are:

- There are more states, and states are stored explicitly.
- The `step` method is more complex. It looks like a concatenation of the `step` and `new` methods of the generators for fields of the variants, with some extra logic for state transitions.
- The `next_to_yield` method is added to extract the value to yield from the state.

As we have the enum enumerator, the implementation of `Enumerable` for the enum is straightforward. The `ENUMERABLE_SIZE_OPTION` is calculated by summing the sizes of all variants.

```rust
impl Enumerable for ComplexEnum {
    type Enumerator = ComplexEnumEnumerator;
    
    fn enumerator() -> Self::Enumerator {
        ComplexEnumEnumerator::new()
    }

    const ENUMERABLE_SIZE_OPTION: Option<usize> = {
        let size: Option<usize> = Some(0usize);
        let size: Option<usize> = match (size, <u8 as Enumerable>::ENUMERABLE_SIZE_OPTION) {
            (Some(size), Some(size_field)) => size.checked_add(size_field),
            _ => None,
        };
        let size: Option<usize> = match (size, Some(1usize)) {
            (Some(size), Some(size_field)) => size.checked_add(size_field),
            _ => None,
        };
        let size: Option<usize> = match (size, <UninhabitedEnum as Enumerable>::ENUMERABLE_SIZE_OPTION) {
            (Some(size), Some(size_field)) => size.checked_add(size_field),
            _ => None,
        };
        let size: Option<usize> = match (
            size,
            {
                let size: Option<usize> = Some(1usize);
                let size: Option<usize> = match (size, <SimpleEnum as Enumerable>::ENUMERABLE_SIZE_OPTION) {
                    (Some(0), _) | (_, Some(0)) => Some(0),
                    (Some(size), Some(size_field)) => size.checked_mul(size_field),
                    _ => None,
                };
                let size: Option<usize> = match (size, <SimpleEnum as Enumerable>::ENUMERABLE_SIZE_OPTION) {
                    (Some(0), _) | (_, Some(0)) => Some(0),
                    (Some(size), Some(size_field)) => size.checked_mul(size_field),
                    _ => None,
                };
                size
            },
        ) {
            (Some(size), Some(size_field)) => size.checked_add(size_field),
            _ => None,
        };
        size
    };
}
```

## How to deal with generic parameters?

So far, we have only considered the case where the fields are concrete types. But what if the fields are generic parameters? Generic parameters are already a complex topic, and it's even more complex when combined with bounds and defaults, like:

```rust
#[derive(Clone, Copy)]
struct ExampleGeneric<
    A: Enumerable + Hash,
    B: Enumerable<Enumerator: ExactSizeIterator>,
    C: Hash + Copy = A
> where
    A::Enumerator: ExactSizeIterator, B: PartialOrd + PartialEq,
{
    field1: A,
    field2: bool,
    field3: Result<B, C>,
}
```

Luckily, implementing `Enumerable` for types with generic parameters is not that hard. We can just copy the generic parameters and bounds (both the bounds in the `where` clause and the bounds in generic parameters) to the definition of enumerators (with some modifications, of course), and the implementation of `Enumerable`, all other things are the same. The enumerator for the example above looks like this:

```rust
pub struct ExampleGenericEnumerator<
    A: Enumerable + Hash,
    B: Enumerable<Enumerator: ExactSizeIterator>,
    C: Hash + Copy, // ⬅️ default removed here
> where
    // ⬇️ where clause copied from the struct
    A::Enumerator: ExactSizeIterator,
    B: PartialOrd + PartialEq,
    // ⬇️ where clause for all types in the struct
    A: Enumerable,
    bool: Enumerable,
    Result<B, C>: Enumerable,
{
    field1_enumerator: <A as Enumerable>::Enumerator,
    field2_enumerator: <bool as Enumerable>::Enumerator,
    field3_enumerator: <Result<B, C> as Enumerable>::Enumerator,
    next: Option<ExampleGeneric<A, B, C>>,
}
```

And the implementation of `Enumerable` for the example above looks like this:

```rust
impl<
    A: Enumerable + Hash,
    B: Enumerable<Enumerator: ExactSizeIterator>,
    C: Hash + Copy, // ⬅️ default removed here also
> ExampleGenericEnumerator<A, B, C> // ⬅️ only the identifiers here
where
    // ⬇️ where clause same as the definition of the enumerator
    A::Enumerator: ExactSizeIterator,
    B: PartialOrd + PartialEq,
    A: Enumerable,
    bool: Enumerable,
    Result<B, C>: Enumerable, 
{
    // methods here are unchanged
}

// Generic parameters and where clause are the same as above in
// `impl<...> Iterator for ExampleGenericEnumerator<...> where ...` and
// `impl<...> Enumerable for ExampleGeneric<...> where ...`.
```
