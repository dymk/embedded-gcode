use crate::{
    gcode::Gcode,
    interpret::model_state::{ModelState, ModelStateUnit},
    GCodeInterpreter, InterpretError,
};

impl<'a> GCodeInterpreter<'a> {
    pub fn interpret_gcode(
        &mut self,
        model_state: &mut ModelState,
        gcode: Gcode<'a>,
    ) -> Result<(), InterpretError<'a>> {
        match gcode {
            Gcode::G20 => {
                model_state.selected_unit = ModelStateUnit::In;
            }
            Gcode::G21 => {
                model_state.selected_unit = ModelStateUnit::Mm;
            }
            _ => todo!("{:?}", gcode),
        }
        Ok(())
    }
}
