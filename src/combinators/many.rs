use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

/// Applies a parser repeatedly until it fails, collecting all results.
/// Requires at least one successful parse to succeed.
/// ```
/// # use parlance::combinators::many::many;
/// # use parlance::parse::Parser;
/// let parser = many("a");
/// assert_eq!(parser.parse(&"aaab"), Ok((vec!["a", "a", "a"], "b")));
/// ```
pub fn many<I: Input, O, E, F, P: Parser<I, O, E, F>>(p: P) -> impl Parser<I, Vec<O>, NotFound, F> {
    move |input: &I| {
        let (first, mut remaining) = match p.parse(input) {
            Ok(x) => x,
            Err(ParserError::Error(_, l)) => return Err(ParserError::Error(NotFound, l)),
            Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (o, r) = match p.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_, _)) => break,
                Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
    }
}

/// Parses items separated by a delimiter, collecting all items.
/// Requires at least one item to succeed.
/// ```
/// # use parlance::combinators::many::delimited;
/// # use parlance::parse::Parser;
/// let parser = delimited("item", ",");
/// assert_eq!(parser.parse(&"item,item,item"), Ok((vec!["item", "item", "item"], "")));
/// ```
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
            Err(ParserError::Error(_, l)) => return Err(ParserError::Error(NotFound, l)),
            Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
        };
        let mut output = vec![first];
        while !remaining.is_empty() {
            let (_, r) = match delimiter.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_, _)) => break,
                Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
            };
            remaining = r;

            let (o, r) = match p.parse(&remaining) {
                Ok(x) => x,
                Err(ParserError::Error(_, _)) => break,
                Err(ParserError::Failure(e, l)) => return Err(ParserError::Failure(e, l)),
            };
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
    }
}

/// Applies a parser exactly a specified number of times.
/// All applications must succeed or the parser fails.
/// ```
/// # use parlance::combinators::many::repeat;
/// # use parlance::parse::Parser;
/// let parser = repeat("x", 3);
/// assert_eq!(parser.parse(&"xxxyyy"), Ok((vec!["x", "x", "x"], "yyy")));
/// ```
pub fn repeat<I: Input, O, E, F, P: Parser<I, O, E, F>>(
    p: P,
    c: usize,
) -> impl Parser<I, Vec<O>, E, F> {
    move |input: &I| {
        debug_assert!(c > 0);
        let mut output = Vec::with_capacity(c);
        let (first, mut remaining) = p.parse(input)?;
        output.push(first);
        for _ in 0..(c - 1) {
            let (o, r) = p.parse(&remaining)?;
            output.push(o);
            remaining = r;
        }

        Ok((output, remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::ParserResult;

    #[test]
    fn delimited_consumes_delimiter() {
        let parser = delimited("item", ",");
        let result = parser.parse(&"item,x");
        assert_eq!(result, Ok((vec!["item"], "x")));
    }
}
