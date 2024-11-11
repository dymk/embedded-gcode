use crate::parser::{number_code, space_before, IParseResult};
use crate::GcodeParseError;
use nom::{bytes::complete::tag_no_case, character::complete::space0, sequence::preceded, Parser};
use variadics_please::all_tuples_enumerated;

pub trait List<'a, O> {
    fn choice(&mut self, input: &'a [u8]) -> IParseResult<'a, O>;
}

fn try_choice<'a, O, P>(
    input: &'a [u8],
    number: &'static str,
    parser: &mut P,
) -> IParseResult<'a, O>
where
    P: Parser<&'a [u8], O, GcodeParseError<'a>>,
{
    let input_without_number = match number_code(number).parse(input) {
        Ok((i, _)) => i,
        Err(e) => return Err(e),
    };
    let input_without_space = match space0.parse(input_without_number) {
        Ok((i, _)) => i,
        Err(e) => return Err(e),
    };
    parser.parse(input_without_space)
}

macro_rules! expand_code_parsers {
    ($(($n:tt, $Parser:ident)),*) => {
        impl<'a, O, $($Parser),*> List<'a, O> for ($((&'static str, $Parser),)*)
        where
            $($Parser: Parser<&'a [u8], O, GcodeParseError<'a>>),*
        {
            fn choice(&mut self, input: &'a [u8]) -> IParseResult<'a, O> {
                $(
                    if let Ok(result) = try_choice(input, self.$n .0, &mut self.$n .1) {
                        return Ok(result);
                    }
                )*

                Err(nom::Err::Error(GcodeParseError::NomError(
                    nom::error::Error::new(input, nom::error::ErrorKind::Alt),
                )))
            }
        }
    };
}

all_tuples_enumerated!(expand_code_parsers, 0, 16, P);

pub fn parse_code_and_number<'a, O: 'static>(
    code_char: u8,
    mut parsers: impl List<'a, O>,
) -> impl Parser<&'a [u8], O, GcodeParseError<'a>> {
    preceded(space_before(tag_no_case([code_char])), move |input| {
        parsers.choice(input)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        gcode::{expression::Expression, Axes, Axis, Gcode},
        parser::map_res_f1,
        GcodeParser as _,
    };
    use nom::combinator::opt;

    #[test]
    fn test_parse_code_and_number() {
        let mut parser =
            parse_code_and_number(b'G', (("0", map_res_f1(opt(Axes::parse), Gcode::G0)),));
        let (_, result) = parser.parse(b"G0 X10").unwrap();
        assert_eq!(
            result,
            Gcode::G0(Some(Axes::new().set(Axis::X, Expression::lit(10.0))))
        );
    }
}
