use crate::{
    interpret::{
        model_state::{ModelState, ModelStateUnit},
        GCodeInterpreter,
    },
    parse_command, ParserAllocator,
};
use alloc::boxed::Box;
use core::error::Error;

#[test]
fn test_interpret_context() -> Result<(), Box<dyn Error>> {
    let mut interpreter = GCodeInterpreter::default();
    let mut heap = bump_into::space_uninit!(1024);
    let alloc = ParserAllocator::new(&mut heap);

    let mut model_state = ModelState::default();
    assert_eq!(model_state.selected_unit, ModelStateUnit::Mm);

    let (_, command) = parse_command(&alloc, b"G20").map_err(|_| "failed to parse G20")?;
    interpreter
        .interpret(&mut model_state, command)
        .map_err(|_| "failed to interpret G20")?;
    assert_eq!(model_state.selected_unit, ModelStateUnit::In);

    let (_, command) = parse_command(&alloc, b"G21").map_err(|_| "failed to parse G21")?;
    interpreter
        .interpret(&mut model_state, command)
        .map_err(|_| "failed to interpret G21")?;
    assert_eq!(model_state.selected_unit, ModelStateUnit::Mm);
    Ok(())
}
