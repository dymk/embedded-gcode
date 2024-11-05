use crate::{
    gcode::Command, interpret::model_state::ModelStateUnit, GcodeParser as _, Interpreter,
};
use alloc::boxed::Box;
use core::error::Error;

#[test]
fn test_interpret_context() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::default();
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::Mm
    );

    let (_, command) = Command::parse(b"G20").map_err(|_| "failed to parse G20")?;
    interpreter
        .interpret(command)
        .map_err(|_| "failed to interpret G20")?;
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::In
    );

    let (_, command) = Command::parse(b"G21").map_err(|_| "failed to parse G21")?;
    interpreter
        .interpret(command)
        .map_err(|_| "failed to interpret G21")?;
    assert_eq!(
        interpreter.get_model_state().selected_unit,
        ModelStateUnit::Mm
    );
    Ok(())
}
