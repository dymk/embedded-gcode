#![no_std]

extern crate alloc;

mod bind;
mod enum_value_map;
mod gcode;
mod interpret;
mod line_reader;
mod parse_error;
mod parser;

const NUM_AXES: usize = 3;
pub use crate::gcode::Command;
pub use crate::interpret::InterpretError;
pub use crate::interpret::Interpreter;
pub use crate::parser::GcodeParseError;
pub use crate::parser::GcodeParser;
