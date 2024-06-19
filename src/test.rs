use super::Enumerable;
use std::{cmp::PartialEq, fmt::Debug, vec};

mod utils {
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

    #[allow(dead_code)]
    pub fn collect_all<T: Enumerable>() -> Vec<T> {
        T::enumerator().collect()
    }
}

use utils::*;

fn assert_enumerator_eq<T: Enumerable + Debug + PartialEq>(expected: impl IntoIterator<Item = T>) {
    let mut expected_iter = expected.into_iter();
    let mut actual_iter = T::enumerator();

    loop {
        let expected = expected_iter.next();
        let actual = actual_iter.next();

        assert_eq!(expected, actual);
        if expected.is_none() {
            break;
        }
    }
}

/// Assert enumerator yields all elements in order and provides correct size hint.
fn assert_enumerator_eq_with_size_hint<T: Enumerable + Debug + PartialEq>(
    expected: impl IntoIterator<Item = T>,
) {
    let mut expected = expected.into_iter().collect::<Vec<T>>().into_iter();
    let mut iter = T::enumerator();
    loop {
        assert_eq!(iter.size_hint(), expected.size_hint());
        assert_eq!(iter.next(), expected.next());
        if expected.len() == 0 {
            break;
        }
    }
}

#[test]
fn test_bool() {
    assert_enumerator_eq_with_size_hint(vec![false, true]);
}

#[test]
fn test_option_bool() {
    assert_enumerator_eq(vec![None, Some(false), Some(true)]);
}

#[test]
fn test_result_bool_bool() {
    assert_enumerator_eq(vec![Ok(false), Ok(true), Err(false), Err(true)]);
}

#[test]
fn test_primitive_numeric() {
    assert_enumerator_eq(u8::MIN..=u8::MAX);
    assert_enumerator_eq(i8::MIN..=i8::MAX);
    assert_enumerator_eq(i16::MIN..=i16::MAX);
    assert_enumerator_eq(u16::MIN..=u16::MAX);
    /*
       // Very slow, tested locally.
       assert_enumerator_eq(i32::MIN..=i32::MAX);
       assert_enumerator_eq(u32::MIN..=u32::MAX);
    */
}

#[test]
fn test_char() {
    assert_eq!(char::enumerator().skip(0x61).next(), Some('\u{61}'));
    assert_ne!(char::enumerator().skip(0xF987).next(), Some('\u{F987}'));
    assert_eq!(
        char::enumerator().skip(0xF987 - 0x800).next(),
        Some('\u{F987}')
    );
}

#[test]
fn test_enum_derive() {
    assert_enumerator_eq(vec![Enum3::A, Enum3::B, Enum3::C]);
    assert_enumerator_eq(vec![Enum4::W, Enum4::X, Enum4::Y, Enum4::Z])
}

#[test]
fn test_enum_derive_complex() {
    let mut expected = vec![ComplexEnum::NoField];
    expected.extend(Enum3::enumerator().map(ComplexEnum::UnnamedField));
    expected.extend(Enum3::enumerator().map(|e3| ComplexEnum::NamedField { e3 }));
    expected.extend(
        <(Enum3, Enum4) as Enumerable>::enumerator()
            .map(|(e3, e4)| ComplexEnum::MultipleUnnamedFields(e3, e4)),
    );
    expected.extend(
        <(Enum3, Enum4) as Enumerable>::enumerator()
            .map(|(e3, e4)| ComplexEnum::MultipleNamedFields { e3, e4 }),
    );
    expected.extend(Enum3::enumerator().map(|e3| ComplexEnum::UnnamedFieldAfterEmpty { e3 }));
    assert_enumerator_eq(expected);

    // Checks whether the custom enumerator name is used.
    let _: ComplexEnumerator = ComplexEnum::enumerator();
}

#[test]
fn test_unit_struct() {
    assert_enumerator_eq(vec![StructUnit {}]);
    assert_enumerator_eq(vec![StructUnitFieldsNamed {}]);
    assert_enumerator_eq(vec![StructUnitFieldsUnnamed()]);
}

#[test]
fn test_structs() {
    assert_enumerator_eq(vec![
        Struct2 {
            e3: Enum3::A,
            e4: Enum4::W,
        },
        Struct2 {
            e3: Enum3::A,
            e4: Enum4::X,
        },
        Struct2 {
            e3: Enum3::A,
            e4: Enum4::Y,
        },
        Struct2 {
            e3: Enum3::A,
            e4: Enum4::Z,
        },
        Struct2 {
            e3: Enum3::B,
            e4: Enum4::W,
        },
        Struct2 {
            e3: Enum3::B,
            e4: Enum4::X,
        },
        Struct2 {
            e3: Enum3::B,
            e4: Enum4::Y,
        },
        Struct2 {
            e3: Enum3::B,
            e4: Enum4::Z,
        },
        Struct2 {
            e3: Enum3::C,
            e4: Enum4::W,
        },
        Struct2 {
            e3: Enum3::C,
            e4: Enum4::X,
        },
        Struct2 {
            e3: Enum3::C,
            e4: Enum4::Y,
        },
        Struct2 {
            e3: Enum3::C,
            e4: Enum4::Z,
        },
    ]);

    assert_enumerator_eq(vec![
        StructTuple2(Enum3::A, Enum4::W),
        StructTuple2(Enum3::A, Enum4::X),
        StructTuple2(Enum3::A, Enum4::Y),
        StructTuple2(Enum3::A, Enum4::Z),
        StructTuple2(Enum3::B, Enum4::W),
        StructTuple2(Enum3::B, Enum4::X),
        StructTuple2(Enum3::B, Enum4::Y),
        StructTuple2(Enum3::B, Enum4::Z),
        StructTuple2(Enum3::C, Enum4::W),
        StructTuple2(Enum3::C, Enum4::X),
        StructTuple2(Enum3::C, Enum4::Y),
        StructTuple2(Enum3::C, Enum4::Z),
    ]);
}
