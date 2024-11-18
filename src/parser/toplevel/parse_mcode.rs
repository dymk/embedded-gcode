use nom::{
    combinator::{map_res, opt},
    Parser as _,
};

use crate::{
    gcode::{Mcode, Tcode},
    parser::{
        nom_types::{IParseResult, IntoParser as _},
        ok,
        parse_code_and_number::parse_code_and_number,
        Input,
    },
    GcodeParser,
};

impl GcodeParser for Mcode {
    fn parse(input: Input) -> IParseResult<Self> {
        parse_code_and_number(
            b'M',
            (
                ("3", Mcode::M3.into_parser()),
                ("4", Mcode::M4.into_parser()),
                ("5", Mcode::M5.into_parser()),
                (
                    "6",
                    map_res(opt(Tcode::parse), |tcode| ok(Mcode::M6(tcode))),
                ),
                ("7", Mcode::M7.into_parser()),
                ("8", Mcode::M8.into_parser()),
                ("9", Mcode::M9.into_parser()),
            ),
        )
        .parse(input)
    }
}
