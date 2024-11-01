use crate::{
    interpret::model_state::{ModelState, ModelStateUnit},
    parse_command, ParserAllocator,
};

use super::GCodeInterpreter;

#[test]
fn test_interpret_context() {
    let mut interpreter = GCodeInterpreter::default();
    let mut heap = bump_into::space_uninit!(1024);
    let alloc = ParserAllocator::new(&mut heap);

    let mut model_state = ModelState::default();
    assert_eq!(model_state.selected_unit, ModelStateUnit::Mm);

    let (_, command) = parse_command(&alloc, b"G20").unwrap();
    interpreter.interpret(&mut model_state, command).unwrap();
    assert_eq!(model_state.selected_unit, ModelStateUnit::In);

    let (_, command) = parse_command(&alloc, b"G21").unwrap();
    interpreter.interpret(&mut model_state, command).unwrap();
    assert_eq!(model_state.selected_unit, ModelStateUnit::Mm);
}
