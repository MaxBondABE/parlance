use crate::{
    input::Input,
    parse::{
        Missing, Never, NotFound, Parser, ParserError, PartialError, PartialOk, PartialParser,
    },
    util::splits::splits,
};

/// Searches for the first position where the given parser succeeds, returning everything before that position.
/// ```
/// # use parlance::combinators::seek::seek;
/// # use parlance::parse::{Parser, Choice};
/// let parser = seek(("world", "test").or());
/// assert_eq!(parser.parse(&"hello world!"), Ok(("hello ", "world!")));
/// ```
pub fn seek<I: Input, O, E, F, P: Parser<I, O, E, F>>(parser: P) -> impl Parser<I, I, NotFound, F> {
    move |input: &I| {
        for (head, tail) in splits(input) {
            match parser.parse(&tail) {
                Err(ParserError::Error(_, _)) => (),

                Ok(_) => return Ok(input.split_at(head.len())),
                Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
            }
        }

        Err(ParserError::Error(NotFound, input.location()))
    }
}

/// Partial version of seek that can handle incomplete input streams.
/// Searches for the first position where the parser succeeds in a streaming context.
/// ```
/// # use parlance::combinators::seek::partial_seek;
/// # use parlance::parse::{PartialParser, Choice, Missing};
/// # use parlance::util::as_partial::AsPartialParser;
/// let parser = partial_seek(("end", "stop").or().as_partial().to_failure::<Missing>());
/// let result = parser.partial_parse(&"beginning end of text");
/// ```
pub fn partial_seek<I: Input, O, E, F: From<Missing>, P: PartialParser<I, O, E, F>>(
    parser: P,
) -> impl PartialParser<I, I, E, F> {
    move |input: &I| {
        for (head, tail) in splits(input) {
            match parser.partial_parse(&tail) {
                Err(PartialError::Error(_, _)) => (),

                Ok(_) => {
                    let (o, r) = input.split_at(head.len());
                    return Ok(PartialOk::Complete(o, r));
                }
                Err(PartialError::Incomplete(_, l)) => {
                    return Err(PartialError::Incomplete(Missing.into(), l))
                }
                Err(PartialError::Failure(e, l)) => return Err(PartialError::Failure(e, l)),
            }
        }

        Err(PartialError::Incomplete(Missing.into(), input.location()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::Choice;

    #[test]
    fn simple() {
        assert_eq!(seek(("a", "b").or()).parse(&"123abc"), Ok(("123", "abc")));

        assert_eq!(
            seek(("d", "e").or()).parse(&"123abc"),
            Err(ParserError::Error(NotFound, ()))
        );
    }

    #[test]
    fn seek_at_position_zero() {
        let parser = seek("start");
        let result = parser.parse(&"start of text");
        assert_eq!(result, Ok(("", "start of text")));
    }
}
