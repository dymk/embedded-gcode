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
    pub fn interpret<'b>(
        &mut self,
        model_state: &mut ModelState,
        command: Command<'b>,
    ) -> Result<(), InterpretError> {
        match command {
            Command::Comment(_) => todo!(),
            Command::G(gcode) => self.interpret_gcode(model_state, gcode),
            Command::M(mcode) => todo!(),
            Command::O(ocode) => todo!(),
            Command::S(scode) => todo!(),
            Command::T(tcode) => todo!(),
        }
    }
}
