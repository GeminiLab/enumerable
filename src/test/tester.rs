use crate::Enumerable;
use std::{cmp::PartialEq, fmt::Debug};

/// Assert enumerator yields all elements in order.
pub fn assert_enumerator_eq<T: Enumerable + Debug + PartialEq>(
    expected: impl IntoIterator<Item = T>,
) {
    let expected = Vec::from_iter(expected);
    assert_eq!(T::ENUMERABLE_SIZE, expected.len());
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
pub fn assert_enumerator_eq_with_size_hint<T: Enumerable + Debug + PartialEq>(
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

/// Collect all elements of an enumerable into a vector.
#[allow(dead_code)]
pub fn collect_all<T: Enumerable>() -> Vec<T> {
    T::enumerator().collect()
}
