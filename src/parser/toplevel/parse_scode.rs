use crate::{
    gcode::Scode,
    parser::{map_res_f1, nom_types::IParseResult, space_before, Input},
    GcodeParser,
};
use nom::{bytes::complete::tag_no_case, number::complete::float, sequence::preceded, Parser as _};

impl GcodeParser for Scode {
    fn parse(input: Input) -> IParseResult<Self> {
        preceded(space_before(tag_no_case("S")), map_res_f1(float, Scode)).parse(input)
    }
}
