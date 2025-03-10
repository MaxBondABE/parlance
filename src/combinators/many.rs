use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

pub fn many<I: Input, O, E, F, P: Parser<I, O, E, F>>(p: P) -> impl Parser<I, Vec<O>, NotFound, F> {
    move |input: &I| {
        let (mut remaining, first) = match p.parse(input) {
            Ok(x) => x,
            Err(ParserError::Error(_)) => return Err(ParserError::Error(NotFound)),
            Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            Err(ParserError::Incomplete(e)) => return Err(ParserError::Incomplete(e)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (r, o) = match p.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
                Err(ParserError::Incomplete(e)) => return Err(ParserError::Incomplete(e)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((remaining, output))
    }
}

pub fn delimited<
    I: Input,
    Output,
    DelimiterOutput,
    E,
    F,
    P: Parser<I, Output, E, F>,
    Delimiter: Parser<I, DelimiterOutput, E, F>,
>(
    p: P,
    delimiter: Delimiter,
) -> impl Parser<I, Vec<Output>, NotFound, F> {
    move |input: &I| {
        let (mut remaining, first) = match p.parse(input) {
            Ok(x) => x,
            Err(ParserError::Error(_)) => return Err(ParserError::Error(NotFound)),
            Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            Err(ParserError::Incomplete(e)) => return Err(ParserError::Incomplete(e)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (r, _) = match delimiter.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
                Err(ParserError::Incomplete(e)) => return Err(ParserError::Incomplete(e)),
            };

            let (r, o) = match p.parse(&r) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
                Err(ParserError::Incomplete(e)) => return Err(ParserError::Incomplete(e)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((remaining, output))
    }
}

pub fn repeat<I: Input, O, E, F, P: Parser<I, O, E, F>>(
    p: P,
    c: usize,
) -> impl Parser<I, Vec<O>, E, F> {
    move |input: &I| {
        debug_assert!(c > 0);
        let mut output = Vec::with_capacity(c);
        let (mut remaining, first) = p.parse(input)?;
        for _ in 0..(c - 1) {
            let (r, o) = p.parse(&remaining)?;
            output.push(o);
            remaining = r;
        }

        Ok((remaining, output))
    }
}
