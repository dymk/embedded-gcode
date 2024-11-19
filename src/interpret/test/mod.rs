use super::interpreter::InterpretValue;
use crate::{
    gcode::Command, interpret::model_state::ModelStateUnit, GcodeParser as _, Interpreter,
};
use alloc::boxed::Box;
use core::error::Error;

extern crate std;

fn try_parse_interpret(
    interpreter: &mut Interpreter,
    input: &[u8],
) -> Result<Command, Box<dyn Error>> {
    use crate::parser::Input;
    let input = Input::new(input, interpreter);

    Command::parse(input)
        .map_err(|e| std::format!("error parsing {}: {:?}", input.as_utf8().unwrap(), e).into())
        .map(|cmd| cmd.1)
}
fn try_interpret(
    interpreter: &mut Interpreter,
    input: &[u8],
) -> Result<InterpretValue, Box<dyn Error>> {
    let command = try_parse_interpret(interpreter, input)?;
    interpreter.interpret(command).map_err(|e| {
        std::format!(
            "error interpreting {}: {:?}",
            std::str::from_utf8(input).unwrap(),
            e
        )
        .into()
    })
}

#[test]
fn test_interpret_context() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::default();
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::Mm
    );

    try_interpret(&mut interpreter, b"G20")?;
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::In
    );

    try_interpret(&mut interpreter, b"G21")?;
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::Mm
    );
    Ok(())
}

#[test]
fn test_interpret_assign() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::default();

    // assign a param
    let value = try_interpret(&mut interpreter, b"#1 = 10")?;
    assert_eq!(value, InterpretValue::EvalExpr(10.0));
    assert_eq!(interpreter.get_numbered_param(1), Some(10.0));

    // reassign a param
    let value = try_interpret(&mut interpreter, b"#1 = 20")?;
    assert_eq!(value, InterpretValue::EvalExpr(20.0));
    assert_eq!(interpreter.get_numbered_param(1), Some(20.0));

    // assign from another param
    let value = try_interpret(&mut interpreter, b"#2 = #1")?;
    assert_eq!(value, InterpretValue::EvalExpr(20.0));
    assert_eq!(interpreter.get_numbered_param(2), Some(20.0));

    // test indirectly assigning a param
    let value = try_interpret(&mut interpreter, b"##1 = 5")?;
    assert_eq!(value, InterpretValue::EvalExpr(5.0));
    assert_eq!(interpreter.get_numbered_param(20), Some(5.0));

    // test assigning from a param indirectly
    try_interpret(&mut interpreter, b"#2 = 8")?;
    try_interpret(&mut interpreter, b"#1 = 2")?;
    let value = try_interpret(&mut interpreter, b"#20 = ##1")?;
    assert_eq!(value, InterpretValue::EvalExpr(8.0));
    assert_eq!(interpreter.get_numbered_param(20), Some(8.0));

    Ok(())
}
