use crate::parse::{Parser, ParserError};

pub fn required<I, O, E: Default, F, P: Parser<I, Option<O>, E, F>>(
    p: P,
) -> impl Parser<I, O, E, F> {
    required_or(p, Default::default)
}

pub fn required_or<I, O, E, F, P: Parser<I, Option<O>, E, F>, BuildError: Fn() -> E>(
    p: P,
    err: BuildError,
) -> impl Parser<I, O, E, F> {
    move |input: &I| match p.parse(input)? {
        (remaining, Some(output)) => Ok((remaining, output)),
        _ => Err(ParserError::Error(err())),
    }
}
