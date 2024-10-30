extern crate std;

use super::parse_axes;
use super::parse_command::parse_command;

use crate::gcode::{
    expression::Expression, Axes, Axis, Command, Gcode, Mcode, Ocode, OcodeStatement, Scode, Tcode,
};
use crate::parser::nom_alloc::NomAlloc;
use crate::permute_whitespace;
use bump_into::BumpInto;

#[rstest::rstest]
#[case(&["X", "1"], Axes::new().set(Axis::X, 1.0))]
#[case(&["x", "1"], Axes::new().set(Axis::X, 1.0))]
#[case(&["X", "1.0"], Axes::new().set(Axis::X, 1.0))]
#[case(&["X", "-1.0"], Axes::new().set(Axis::X, -1.0))]
#[case(&["X", "1", "Y", "2.4"], Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
#[case(&["Y", "2.4", "X", "1"], Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
fn test_parse_axes(#[case] tokens: &[&str], #[case] expected: Axes) {
    for input in permute_whitespace(tokens) {
        let result = parse_axes()(input.as_bytes()).unwrap();
        assert_eq!(result.1, expected);
    }
}

#[rstest::rstest]
// gcode
#[case(&["G0", "X1"], Gcode::G0(Some(Axes::new().set(Axis::X, 1.0))))]
#[case(&["g0", "X1"], Gcode::G0(Some(Axes::new().set(Axis::X, 1.0))))]
// ocode
#[case(&["O100", "sub"], Ocode::new(100, OcodeStatement::Sub))]
#[case(&["o100", "sub"], Ocode::new(100, OcodeStatement::Sub))]
#[case(&["o100", "sub"], Ocode::new(100, OcodeStatement::Sub))]
#[case(&["o100", "endsub"], Ocode::new(100, OcodeStatement::EndSub))]
#[case(
    &["o100", "if", "[", "#2", "]"],
    Ocode::new(100, OcodeStatement::If(Expression::NumberedParam(2)))
)]
#[case(&["o100", "endif"], Ocode::new(100, OcodeStatement::EndIf))]
// mcode
#[case(&["M3"], Mcode::M3)]
#[case(&["M4"], Mcode::M4)]
#[case(&["M5"], Mcode::M5)]
#[case(&["M6"], Mcode::M6(None))]
#[case(&["m6"], Mcode::M6(None))]
#[case(&["M6", "T8"], Mcode::M6(Some(Tcode(8))))]
#[case(&["M7"], Mcode::M7)]
#[case(&["M8"], Mcode::M8)]
#[case(&["M9"], Mcode::M9)]
#[case(&["S1000"], Scode(1000.0))]
#[case(&["T1"], Tcode(1))]
fn test_parse_command<'a>(#[case] tokens: &[&str], #[case] expected: impl Into<Command<'a>>) {
    let expected = expected.into();
    for input in permute_whitespace(tokens) {
        let mut heap = bump_into::space_uninit!(64);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let result = match parse_command(alloc)(input.as_bytes()) {
            Ok(result) => result,
            Err(err) => {
                panic!("Failed to parse `{}`: {:?}", input, err);
            }
        };
        assert_eq!(result.1, expected, "for input `{}`", input);
    }
}
