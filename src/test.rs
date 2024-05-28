use super::Enumerable;

fn collect_all<T: Enumerable>() -> Vec<T> {
    T::enumerator().collect()
}

#[test]
fn test_bool() {
    assert_eq!(collect_all::<bool>(), vec![false, true]);
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
