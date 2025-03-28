use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

pub fn many<I: Input, O, E, F, P: Parser<I, O, E, F>>(p: P) -> impl Parser<I, Vec<O>, NotFound, F> {
    move |input: &I| {
        let (first, mut remaining) = match p.parse(input) {
            Ok(x) => x,
            Err(ParserError::Error(_)) => return Err(ParserError::Error(NotFound)),
            Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (o, r) = match p.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
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
        let (first, mut remaining) = match p.parse(input) {
            Ok(x) => x,
            Err(ParserError::Error(_)) => return Err(ParserError::Error(NotFound)),
            Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (_, r) = match delimiter.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            };

            let (o, r) = match p.parse(&r) {
                Ok(x) => x,
                Err(ParserError::Error(_)) => break,
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
    }
}

pub fn repeat<I: Input, O, E, F, P: Parser<I, O, E, F>>(
    p: P,
    c: usize,
) -> impl Parser<I, Vec<O>, E, F> {
    move |input: &I| {
        debug_assert!(c > 0);
        let mut output = Vec::with_capacity(c);
        let (first, mut remaining) = p.parse(input)?;
        for _ in 0..(c - 1) {
            let (o, r) = p.parse(&remaining)?;
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
    }
}
