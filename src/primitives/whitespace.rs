use crate::{
    input::Input,
    parse::{IntoStreamingResult, NotFound, Parser, ParserResult, StreamingResult},
    util::conditional_transforms::{CompleteIf, EitherCompleteIf, OrNotFound},
};

pub fn whitespace<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_whitespace()).ok_or_not_found()
}

pub fn whitespace_stream<I: Input>(s: &I) -> StreamingResult<I, I> {
    whitespace.parse(s).has_stopped()
}
