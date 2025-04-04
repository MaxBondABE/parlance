use crate::{
    input::Input,
    parse::{AsPartialResult, NotFound, Parser, ParserResult, PartialResult},
    util::conditional_transforms::{CompleteIf, EitherCompleteIf, OrNotFound},
};

pub fn whitespace<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_whitespace()).ok_or_not_found()
}

pub fn partial_whitespace<I: Input>(s: &I) -> PartialResult<I, I> {
    whitespace.parse(s).has_stopped()
}
