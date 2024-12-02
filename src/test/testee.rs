use crate::Enumerable;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub enum Enum0 {}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub enum Enum3 {
    A,
    B,
    C,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub enum Enum4 {
    W,
    X,
    Y,
    Z,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct StructUnit;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct StructUnitFieldsNamed {}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct StructUnitFieldsUnnamed();

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct Struct2 {
    pub e3: Enum3,
    pub e4: Enum4,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
#[enumerator(YesThisTypeEnumeratesStructTuple2)] // test custom enumerator names with a weird one
pub struct StructTuple2(pub Enum3, pub Enum4);

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
#[enumerator = "ComplexEnumerator"]
pub enum ComplexEnum {
    NoField,
    UnnamedField(Enum3),
    NamedField { e3: Enum3 },
    MultipleUnnamedFields(Enum3, Enum4),
    MultipleNamedFields { e3: Enum3, e4: Enum4 },
    EmptyBranch(Enum0),
    UnnamedFieldAfterEmpty { e3: Enum3 },
}

// following are test types for generic types.
//
// they are also used to test whether the `#[derive(Enumerable)]` macro can
// handle all kinds of generic types and bounds correctly.

// test no where clause
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct GenericStruct1<T: Copy + Enumerable> {
    pub field: T,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub struct GenericStruct2<T, U: Enumerable = T>
where
    T: Clone + Enumerable + PartialEq,
    <T as Enumerable>::Enumerator: ExactSizeIterator,
{
    pub field1: T,
    pub field2: U,
}

#[rustfmt::skip] // test where clause without trailing comma
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
pub enum GenericEnum3<
    T: Enumerable<Enumerator: ExactSizeIterator>,
    U: Enumerable,
    V = U,
> where T: Clone + PartialEq, V: Copy {
    Variant1(GenericStruct2<T, U>),
    Variant2, // test empty variant
    Variant3(Result<U, V>),
}
