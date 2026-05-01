//! Tests for the public vertex index API.

mod basics {
    use cityjson_types::v2_0::*;

    #[test]
    fn constructors_and_accessors_work() {
        let idx16 = VertexIndex16::new(42);
        let idx32 = VertexIndex32::new(42);
        let idx64 = VertexIndex64::new(42);

        assert_eq!(idx16.value(), 42u16);
        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);
        assert_eq!(idx16.to_usize(), 42usize);
        assert_eq!(idx32.to_usize(), 42usize);
        assert_eq!(idx64.to_usize(), 42usize);
    }

    #[test]
    fn helper_methods_report_index_state() {
        assert!(VertexIndex16::new(u16::MAX).is_max());
        assert!(!VertexIndex16::new(7).is_max());

        assert!(VertexIndex32::new(0).is_zero());
        assert!(!VertexIndex32::new(1).is_zero());

        assert_eq!(VertexIndex16::new(42).next().unwrap().value(), 43);
        assert!(VertexIndex16::new(u16::MAX).next().is_none());
    }

    #[test]
    fn from_u32_accepts_fitting_values() {
        assert_eq!(VertexIndex16::from_u32(42).unwrap().value(), 42u16);
        assert_eq!(VertexIndex32::from_u32(70_000).unwrap().value(), 70_000u32);
        assert_eq!(
            VertexIndex64::from_u32(u32::MAX).unwrap().value(),
            u64::from(u32::MAX)
        );
        assert!(VertexIndex16::from_u32(70_000).is_none());
    }
}

mod arithmetic {
    use cityjson_types::error::Error;
    use cityjson_types::v2_0::*;

    #[test]
    fn checked_add_returns_sum_without_overflow() {
        let sum = VertexIndex16::new(10)
            .checked_add(VertexIndex16::new(5))
            .unwrap();

        assert_eq!(sum.value(), 15);
    }

    #[test]
    fn add_assign_saturates_on_overflow() {
        let mut idx = VertexIndex16::new(u16::MAX);

        idx += VertexIndex16::new(1);

        assert_eq!(idx.value(), u16::MAX);
    }

    #[test]
    fn try_add_assign_reports_overflow_and_keeps_value() {
        let mut idx = VertexIndex16::new(u16::MAX);

        let error = idx.try_add_assign(VertexIndex16::new(1)).unwrap_err();

        assert!(matches!(
            error,
            Error::IndexOverflow { ref index_type, ref value }
                if index_type == "u16" && value == "65535"
        ));
        assert_eq!(idx.value(), u16::MAX);
    }
}

mod conversions {
    use cityjson_types::error::Error;
    use cityjson_types::v2_0::*;

    #[test]
    fn vertex_index_conversions_handle_widening_and_narrowing() {
        let idx16 = VertexIndex16::new(42);
        let idx32: VertexIndex32 = idx16.try_into().unwrap();
        let idx64: VertexIndex64 = idx16.try_into().unwrap();

        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);

        let idx32 = VertexIndex32::new(50_000);
        let idx64: VertexIndex64 = idx32.try_into().unwrap();
        assert_eq!(idx64.value(), 50_000u64);

        let too_large = VertexIndex32::new(u32::from(u16::MAX) + 1);
        let error: Error = VertexIndex16::try_from(too_large).unwrap_err();

        assert!(matches!(
            error,
            Error::IndexConversion { ref source_type, ref target_type, ref value }
                if source_type == "u32" && target_type == "u16" && value == "65536"
        ));
    }

    #[test]
    fn raw_integer_conversions_cover_success_and_failure_cases() {
        let idx16: VertexIndex16 = 42u16.into();
        let idx32: VertexIndex32 = 42u32.into();
        let idx64: VertexIndex64 = 42u64.into();
        let widened_from_u16: VertexIndex32 = 42u16.into();
        let widened_from_u32: VertexIndex64 = 42u32.into();

        assert_eq!(idx16.value(), 42u16);
        assert_eq!(idx32.value(), 42u32);
        assert_eq!(idx64.value(), 42u64);
        assert_eq!(widened_from_u16.value(), 42u32);
        assert_eq!(widened_from_u32.value(), 42u64);

        assert!(VertexIndex16::try_from(65_536u32).is_err());
        assert!(VertexIndex32::try_from(0x0001_0000_0000_u64).is_err());
        assert_eq!(VertexIndex16::try_from(42usize).unwrap().value(), 42u16);
        assert_eq!(VertexIndex32::try_from(42usize).unwrap().value(), 42u32);
        assert_eq!(VertexIndex64::try_from(42usize).unwrap().value(), 42u64);
    }
}

mod collections {
    use cityjson_types::error::Error;
    use cityjson_types::v2_0::*;

    #[test]
    fn sequence_generates_contiguous_indices() {
        let indices = VertexIndex16::sequence(10, 5).unwrap();

        assert_eq!(
            indices.iter().map(VertexIndex::value).collect::<Vec<_>>(),
            vec![10, 11, 12, 13, 14]
        );
        assert!(VertexIndex32::sequence(0, 0).unwrap().is_empty());
    }

    #[test]
    fn sequence_reports_overflow() {
        let error = VertexIndex16::sequence(u16::MAX - 2, 5).unwrap_err();

        assert!(matches!(
            error,
            Error::IndexConversion { ref source_type, ref target_type, ref value }
                if source_type == "65535 + 1" && target_type == "u16" && value == "overflow"
        ));
    }

    #[test]
    fn vec_helper_wraps_raw_indices() {
        let raw_u16 = vec![0u16, 1, 2, 3];
        let raw_u32 = vec![100u32, 200, 300];

        let wrapped_u16 = raw_u16.to_vertex_indices();
        let wrapped_u32 = raw_u32.to_vertex_indices();

        assert_eq!(
            wrapped_u16
                .iter()
                .map(VertexIndex::value)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert_eq!(
            wrapped_u32
                .iter()
                .map(VertexIndex::value)
                .collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
    }
}
