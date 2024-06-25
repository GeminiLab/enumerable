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
