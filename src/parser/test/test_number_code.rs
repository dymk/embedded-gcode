
use crate::parser::parse_utils::number_code;

#[test]
fn test_parse_number_code() {
    let result = number_code("0")(b"0");
    assert_eq!(Ok((&b""[..], &b"0"[..])), result);

    let result = number_code("0")(b"00");
    assert!(result.is_err(), "{:?}", result);

    let result = number_code("1")(b"1 ");
    assert_eq!(Ok((&b" "[..], &b"1"[..])), result);
}
