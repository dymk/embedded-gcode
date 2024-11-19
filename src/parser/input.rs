use core::{
    fmt::{Display, Formatter},
    iter::{Copied, Enumerate},
    ops::{Range, RangeFrom, RangeTo},
    slice::Iter,
    str::Utf8Error,
};
use nom::error::Error;

use crate::eval::EvalContext;

#[derive(Debug, Copy, Clone)]
pub struct Input<'a> {
    context: &'a dyn EvalContext,
    input: &'a [u8],
}

impl<'a> Input<'a> {
    pub fn new(input: &'a [u8], context: &'a dyn EvalContext) -> Self {
        Input { context, input }
    }
    pub fn as_utf8(&self) -> Result<&str, Utf8Error> {
        core::str::from_utf8(self.input)
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.input
    }
    pub fn context(&'a self) -> &'a dyn EvalContext {
        self.context
    }

    pub fn iter(&self) -> impl Iterator<Item = &u8> + '_ {
        self.input.iter()
    }

    fn with_slice(&self, slice: &'a [u8]) -> Self {
        Input {
            context: self.context,
            input: slice,
        }
    }

    pub fn convert_error<E2>(&self, e: nom::Err<nom::error::Error<&'a [u8]>>) -> nom::Err<E2>
    where
        E2: nom::error::ParseError<Self>,
    {
        match e {
            nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
            nom::Err::Error(Error { input, code }) => {
                nom::Err::Error(E2::from_error_kind(self.with_slice(input), code))
            }
            nom::Err::Failure(Error { input, code }) => {
                nom::Err::Failure(E2::from_error_kind(self.with_slice(input), code))
            }
        }
    }
}

impl Display for Input<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Input(fold: {}, \"{}\")",
            self.context.const_fold(),
            self.as_utf8().unwrap()
        )
    }
}

impl<'a> PartialEq<&'a [u8]> for Input<'a> {
    fn eq(&self, other: &&[u8]) -> bool {
        self.input == *other
    }
}
impl<'a> PartialEq<Input<'a>> for &[u8] {
    fn eq(&self, other: &Input<'a>) -> bool {
        self == &other.input
    }
}

// nom traits impl
impl<'a> nom::InputLength for Input<'a> {
    fn input_len(&self) -> usize {
        self.input.len()
    }
}
impl<'a> nom::InputTake for Input<'a> {
    fn take(&self, count: usize) -> Self {
        let take = nom::InputTake::take(&self.input, count);
        Input {
            context: self.context,
            input: take,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let split = nom::InputTake::take_split(&self.input, count);
        (self.with_slice(split.0), self.with_slice(split.1))
    }
}

impl<'a> nom::Compare<&[u8]> for Input<'a> {
    fn compare(&self, t: &[u8]) -> nom::CompareResult {
        nom::Compare::compare(&self.input, t)
    }

    fn compare_no_case(&self, t: &[u8]) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.input, t)
    }
}

impl<'a, const N: usize> nom::Compare<[u8; N]> for Input<'a> {
    fn compare(&self, t: [u8; N]) -> nom::CompareResult {
        nom::Compare::compare(&self.input, &t[..])
    }

    fn compare_no_case(&self, t: [u8; N]) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.input, &t[..])
    }
}

impl<'a> nom::Compare<&str> for Input<'a> {
    fn compare(&self, t: &str) -> nom::CompareResult {
        nom::Compare::compare(&self.input, t)
    }

    fn compare_no_case(&self, t: &str) -> nom::CompareResult {
        nom::Compare::compare_no_case(&self.input, t)
    }
}

impl<'a> nom::InputTakeAtPosition for Input<'a> {
    type Item = u8;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self
            .input
            .split_at_position::<_, nom::error::Error<&[u8]>>(predicate)
        {
            Ok(split) => Ok((self.with_slice(split.0), self.with_slice(split.1))),
            Err(e) => Err(self.convert_error(e)),
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self
            .input
            .split_at_position1::<_, nom::error::Error<&[u8]>>(predicate, e)
        {
            Ok(split) => Ok((self.with_slice(split.0), self.with_slice(split.1))),
            Err(e) => Err(self.convert_error(e)),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self
            .input
            .split_at_position_complete::<_, nom::error::Error<&[u8]>>(predicate)
        {
            Ok(split) => Ok((self.with_slice(split.0), self.with_slice(split.1))),
            Err(e) => Err(self.convert_error(e)),
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self
            .input
            .split_at_position1_complete::<_, nom::error::Error<&[u8]>>(predicate, e)
        {
            Ok(split) => Ok((self.with_slice(split.0), self.with_slice(split.1))),
            Err(e) => Err(self.convert_error(e)),
        }
    }
}

impl<'a> nom::InputIter for Input<'a> {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        nom::InputIter::iter_indices(&self.input)
    }

    fn iter_elements(&self) -> Self::IterElem {
        nom::InputIter::iter_elements(&self.input)
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        nom::InputIter::position(&self.input, predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        nom::InputIter::slice_index(&self.input, count)
    }
}

impl<'a> nom::Slice<RangeFrom<usize>> for Input<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.with_slice(&self.input[range])
    }
}

impl<'a> nom::Slice<RangeTo<usize>> for Input<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.with_slice(&self.input[range])
    }
}

impl<'a> nom::Slice<Range<usize>> for Input<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        self.with_slice(&self.input[range])
    }
}

impl<'a> nom::Offset for Input<'a> {
    fn offset(&self, second: &Self) -> usize {
        self.input.offset(second.input)
    }
}

impl<'a, R: core::str::FromStr> nom::ParseTo<R> for Input<'a> {
    fn parse_to(&self) -> Option<R> {
        self.input.parse_to()
    }
}

impl<'a> nom::AsBytes for Input<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.input
    }
}

impl<'a> nom::FindSubstring<&str> for Input<'a> {
    fn find_substring(&self, substr: &str) -> Option<usize> {
        nom::FindSubstring::find_substring(&self.input, substr)
    }
}
