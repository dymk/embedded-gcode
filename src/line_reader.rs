use crate::parse_error::ParseError;

pub async fn next_line<'r, 'b, Read, ReadError>(
    read: &'r mut Read,
    buffer: &'b mut [u8],
) -> Result<Option<&'b mut [u8]>, ParseError<'b, ReadError>>
where
    Read: embedded_io_async::Read<Error = ReadError>,
    ReadError: embedded_io_async::Error,
{
    let mut bytes_read = 0;
    loop {
        if bytes_read == buffer.len() {
            return Err(ParseError::LineTooLong);
        }

        let byte = match read_byte(read).await? {
            None if bytes_read == 0 => return Ok(None),
            Some(b'\n') | None => break,
            Some(byte) => byte,
        };
        buffer[bytes_read] = byte;
        bytes_read += 1;
    }
    Ok(Some(&mut buffer[..bytes_read]))
}

async fn read_byte<'b, Read, ReadError>(
    read: &mut Read,
) -> Result<Option<u8>, ParseError<'b, ReadError>>
where
    ReadError: embedded_io_async::Error,
    Read: embedded_io_async::Read<Error = ReadError>,
{
    let mut byte: [u8; 1] = [0; 1];
    match read.read(&mut byte).await {
        Ok(1) => Ok(Some(byte[0])),
        Ok(0) => Ok(None),
        Ok(other) => Err(ParseError::ReadSize(other)),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use crate::line_reader::next_line;
    use core::str::from_utf8;
    use futures_lite::future::block_on;
    use std::prelude::v1::*;

    pub fn collect_lines(read: &str) -> Vec<String> {
        use std::borrow::ToOwned;
        let mut bytes = read.as_bytes();
        block_on(async {
            let mut buffer = [0; 256];
            let mut lines = std::vec![];
            loop {
                match next_line(&mut bytes, &mut buffer).await {
                    Ok(Some(line)) => {
                        let line = from_utf8(line).unwrap();
                        lines.push(line.to_owned());
                    }
                    Ok(None) => break,
                    Err(err) => panic!("{:?}", err),
                };
            }
            lines
        })
    }

    #[test]
    fn test() {
        assert_eq!(collect_lines("G0"), ["G0"]);
        assert_eq!(collect_lines("G0\n"), ["G0"]);
        assert_eq!(collect_lines("G0\nG1"), ["G0", "G1"]);
    }
}
