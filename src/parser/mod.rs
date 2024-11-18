mod fold_many0_result;
pub mod input;
mod nom_types;
pub mod parse_code_and_number;
mod parse_utils;
pub mod parser_allocator;
pub mod toplevel;

#[cfg(test)]
mod test;

pub trait GcodeParser
where
    Self: Sized,
{
    fn parse(input: Input) -> IParseResult<Self>;
}
pub use fold_many0_result::fold_many0_result;
pub use input::*;
pub use nom_types::GcodeParseError;
pub use nom_types::IParseResult;
pub use parse_utils::*;
