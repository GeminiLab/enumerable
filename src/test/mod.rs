use super::Enumerable;
use std::vec;

mod testee;
mod tester;
use testee::*;
use tester::*;

mod primitive {
    use super::*;

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
        assert_enumerator_eq_with_size_hint(vec![Ok(false), Ok(true), Err(false), Err(true)]);
    }
    
    #[test]
    fn test_primitive_numeric() {
        assert_enumerator_eq_with_size_hint(u8::MIN..=u8::MAX);
        assert_enumerator_eq_with_size_hint(i8::MIN..=i8::MAX);
        assert_enumerator_eq_with_size_hint(i16::MIN..=i16::MAX);
        assert_enumerator_eq_with_size_hint(u16::MIN..=u16::MAX);
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
}

mod enum_and_struct {
    use super::*;

    #[test]
    fn test_enum_derive() {
        // plain enums return a slice iter, so it must have a exact size hint
        assert_enumerator_eq_with_size_hint(vec![Enum3::A, Enum3::B, Enum3::C]);
        assert_enumerator_eq_with_size_hint(vec![Enum4::W, Enum4::X, Enum4::Y, Enum4::Z])
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
        let e: ComplexEnumerator = ComplexEnum::enumerator();
        assert_eq!(ComplexEnum::ENUMERABLE_SIZE, e.count());
    }

    #[test]
    fn test_unit_struct() {
        assert_enumerator_eq(vec![StructUnit {}]);
        assert_enumerator_eq(vec![StructUnitFieldsNamed {}]);
        assert_enumerator_eq(vec![StructUnitFieldsUnnamed()]);
    }

    #[test]
    fn test_structs() {
        let expected = Enum3::enumerator()
            .flat_map(|e3| Enum4::enumerator().map(move |e4| (e3, e4)))
            .collect::<Vec<_>>();

        assert_enumerator_eq(expected.iter().map(|(e3, e4)| Struct2 { e3: *e3, e4: *e4 }));
        assert_enumerator_eq(expected.iter().map(|(e3, e4)| StructTuple2(*e3, *e4)));
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq, Enumerable)]
    struct UnitStruct;

    #[test]
    fn test_derive_unit_struct() {
        assert_eq!(collect_all::<UnitStruct>(), vec![UnitStruct]);
    }
}

mod tuple {
    use super::*;

    #[test]
    fn test_tuple0() {
        assert_eq!(vec![()], collect_all::<()>());
    }

    #[test]
    fn test_tuple1() {
        assert_eq!(vec![(false,), (true,)], collect_all::<(bool,)>());
    }

    #[test]
    fn test_tuple2() {
        // Illustrate the return order of the enumerator.
        assert_eq!(
            vec![(0, false), (0, true), (1, false), (1, true), (2, false)],
            <(u8, bool)>::enumerator().take(5).collect::<Vec<_>>()
        );

        // Verify that the enumerator returns all possible values.
        assert_enumerator_eq((0u8..=0xff).flat_map(|a| [false, true].into_iter().map(move |b| (a, b))));
    }

    #[test]
    fn test_tuple16() {
        type Tuple16 = (
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
        );
        let tuple16_to_u16 = |t: Tuple16| {
            let mut result = 0u16;
            if t.0 {
                result |= 1 << 15;
            }
            if t.1 {
                result |= 1 << 14;
            }
            if t.2 {
                result |= 1 << 13;
            }
            if t.3 {
                result |= 1 << 12;
            }
            if t.4 {
                result |= 1 << 11;
            }
            if t.5 {
                result |= 1 << 10;
            }
            if t.6 {
                result |= 1 << 9;
            }
            if t.7 {
                result |= 1 << 8;
            }
            if t.8 {
                result |= 1 << 7;
            }
            if t.9 {
                result |= 1 << 6;
            }
            if t.10 {
                result |= 1 << 5;
            }
            if t.11 {
                result |= 1 << 4;
            }
            if t.12 {
                result |= 1 << 3;
            }
            if t.13 {
                result |= 1 << 2;
            }
            if t.14 {
                result |= 1 << 1;
            }
            if t.15 {
                result |= 1 << 0;
            }
            result
        };

        let collected_as_u16 = <Tuple16 as Enumerable>::enumerator()
            .map(tuple16_to_u16)
            .collect::<Vec<_>>();
        assert_eq!(
            collected_as_u16,
            <u16 as Enumerable>::enumerator().collect::<Vec<_>>()
        )
    }
}
