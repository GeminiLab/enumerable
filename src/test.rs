use super::Enumerable;
use std::{cmp::PartialEq, fmt::Debug};

fn collect_all<T: Enumerable>() -> Vec<T> {
    T::enumerator().collect()
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
    assert_eq!(
        collect_all::<Option<bool>>(),
        vec![None, Some(false), Some(true)]
    );
}

#[test]
fn test_result_bool_bool() {
    assert_eq!(
        collect_all::<Result<bool, bool>>(),
        vec![Ok(false), Ok(true), Err(false), Err(true)]
    );
}

#[test]
fn test_u8_i8() {
    assert_eq!(collect_all::<u8>(), (u8::MIN..=u8::MAX).collect::<Vec<_>>());
    assert_eq!(collect_all::<i8>(), (i8::MIN..=i8::MAX).collect::<Vec<_>>());
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

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Enumerable)]
enum TestEnum3 {
    A,
    B,
    C,
}

#[test]
fn test_enum_derive() {
    assert_eq!(
        collect_all::<TestEnum3>(),
        vec![TestEnum3::A, TestEnum3::B, TestEnum3::C]
    );
}
