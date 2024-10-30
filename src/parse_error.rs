use crate::parser::GcodeParseError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum ParseError<'a, ReadError> {
    Gcode(GcodeParseError<'a>),
    Read(ReadError),
    ReadSize(usize),
    LineTooLong,
}

impl<ReadError> From<ReadError> for ParseError<'_, ReadError>
where
    ReadError: embedded_io_async::Error,
{
    fn from(value: ReadError) -> Self {
        ParseError::Read(value)
    }
}
