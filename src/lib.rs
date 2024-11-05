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
pub use crate::interpret::{GCodeInterpreter, InterpretError};
pub use crate::parser::parser_allocator::ParserAllocator;
pub use crate::parser::GcodeParseError;
