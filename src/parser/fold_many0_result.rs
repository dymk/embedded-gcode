use nom::{
    error::{ErrorKind, ParseError},
    Err, IResult, InputLength, Parser,
};

pub fn fold_many0_result<I, O, E, F, G, H, R>(
    mut f: F,
    mut init: H,
    mut g: G,
) -> impl FnMut(I) -> IResult<I, R, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: FnMut(R, O) -> Result<R, E>,
    H: FnMut() -> R,
    E: ParseError<I>,
{
    move |i: I| {
        let mut res = init();
        let mut input = i;

        loop {
            let i_ = input.clone();
            let len = input.input_len();
            match f.parse(i_) {
                Ok((i, o)) => {
                    // infinite loop check: the parser must always consume
                    if i.input_len() == len {
                        return Err(Err::Error(E::from_error_kind(input, ErrorKind::Many0)));
                    }

                    res = match g(res, o) {
                        Ok(r) => r,
                        Err(e) => {
                            return Err(nom::Err::Error(e));
                        }
                    };

                    input = i;
                }
                Err(Err::Error(_)) => {
                    return Ok((input, res));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
