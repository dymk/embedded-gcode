use crate::parser::{parse_utils::number_code, Input};
use core::fmt::Debug;

#[track_caller]
fn assert_same_input<E: Debug>(
    expected: Result<(&[u8], &[u8]), E>,
    result: Result<(Input<'_>, Input<'_>), E>,
) {
    match (&expected, &result) {
        (Ok((expected_left, expected_right)), Ok((result_left, result_right))) => {
            assert_eq!(expected_left, result_left);
            assert_eq!(expected_right, result_right);
        }
        _ => panic!("expected {:?} but got {:?}", expected, result),
    }
}

#[test]
fn test_parse_number_code() {
    let result = number_code("0")(b"0"[..].into());
    assert_same_input(Ok((b"", b"0")), result);

    let result = number_code("0")(b"00"[..].into());
    assert!(result.is_err(), "{:?}", result);

    let result = number_code("1")(b"1 "[..].into());
    assert_same_input(Ok((b" ", b"1")), result);
}
