use crate::{
    gcode::{expression::Expression, Ocode, OcodeStatement},
    parser::{
        map_res_f1,
        nom_types::IParseResult,
        ok,
        parse_utils::{parse_u32, space_before},
        Input,
    },
    GcodeParser,
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::map_res,
    sequence::{preceded, tuple},
    Parser as _,
};

impl GcodeParser for Ocode {
    fn parse(input: Input) -> IParseResult<Self> {
        preceded(space_before(tag_no_case("O")), parse_ocode).parse(input)
    }
}

fn parse_ocode(input: Input) -> IParseResult<Ocode> {
    map_res(
        tuple((
            parse_u32(),
            space_before(alt((
                map_res(tag_no_case("sub"), |_| ok(OcodeStatement::Sub)),
                map_res(tag_no_case("endsub"), |_| ok(OcodeStatement::EndSub)),
                preceded(
                    tag_no_case("if"),
                    map_res_f1(Expression::parse, OcodeStatement::If),
                ),
                map_res(tag_no_case("endif"), |_| ok(OcodeStatement::EndIf)),
            ))),
        )),
        |(id, stmt)| ok(Ocode::new(id, stmt)),
    )(input)
}
