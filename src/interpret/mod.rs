use model_state::ModelState;

use crate::gcode::Command;

mod interpret_gcode;
mod model_state;
#[cfg(test)]
mod test;

#[derive(Debug, Default)]
pub struct GCodeInterpreter {
    local_vars: hashbrown::HashMap<alloc::string::String, f64>,
    global_vars: hashbrown::HashMap<alloc::string::String, f64>,
    model_state: model_state::ModelState,
}

#[derive(Debug)]
pub enum InterpretError {}

impl GCodeInterpreter {
    pub fn interpret(
        &mut self,
        model_state: &mut ModelState,
        command: Command,
    ) -> Result<(), InterpretError> {
        match command {
            Command::Comment(_) => todo!(),
            Command::Assign(_, _) => todo!(),
            Command::G(gcode) => self.interpret_gcode(model_state, gcode),
            Command::M(_) => todo!(),
            Command::O(_) => todo!(),
            Command::S(_) => todo!(),
            Command::T(_) => todo!(),
        }
    }
}
