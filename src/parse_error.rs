use core::str::Utf8Error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn inc_line(&mut self) {
        self.line += 1;
        self.byte += 1;
        self.column = 0;
    }

    pub fn inc_column_by(&mut self, by: usize) {
        self.column += by;
        self.byte += by;
    }
}

#[derive(Debug)]
pub enum ParseError<'b, ReadError> {
    NomError(nom::Err<nom::error::Error<&'b [u8]>>),
    ReadError(ReadError),
    Utf8Error(Utf8Error),
    InvalidReadSize(usize),
    LineTooLongForBuffer,
    InvalidToken(u8, Position),
}

impl<'b, ReadError> From<ReadError> for ParseError<'b, ReadError>
where
    ReadError: embedded_io_async::Error,
{
    fn from(value: ReadError) -> Self {
        ParseError::ReadError(value)
    }
}
