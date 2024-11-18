use crate::{
    gcode::Tcode,
    parser::{map_res_f1, nom_types::IParseResult, parse_utils::parse_u32, space_before, Input},
    GcodeParser,
};
use nom::{bytes::complete::tag_no_case, sequence::preceded, Parser};

impl GcodeParser for Tcode {
    fn parse(input: Input) -> IParseResult<Self> {
        preceded(
            space_before(tag_no_case("T")),
            map_res_f1(parse_u32(), Tcode),
        )
        .parse(input)
    }
}
