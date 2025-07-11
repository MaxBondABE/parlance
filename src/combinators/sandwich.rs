//! Sandwich combinators are anything with a distinct beginning, middle, and end.

use crate::{
    input::Input,
    parse::{Never, NotFound, Parser, Sequence},
    primitives::whitespace,
};

/// Parses input surrounded by the same parser (bread) before and after a content parser, returning only the content result.
pub fn sandwich<
    I,
    BreadOutput,
    ContentOutput,
    E,
    F,
    Bread: Parser<I, BreadOutput, E, F>,
    Content: Parser<I, ContentOutput, E, F>,
>(
    bread: Bread,
    content: Content,
) -> impl Parser<I, ContentOutput, E, F> {
    move |input: &I| {
        // We can't use Sequence/.and() here because bread is referenced twice
        // (and is not Copy).
        let (_, remaining) = bread.parse(input)?;
        let (output, remaining) = content.parse(&remaining)?;
        let (_, remaining) = bread.parse(&remaining)?;
        Ok((output, remaining))
    }
}

/// Parses input between a start and an end parser, returning only the content result.
pub fn between<
    I,
    StartOutput,
    ContentOutput,
    EndOutput,
    E,
    F,
    Start: Parser<I, StartOutput, E, F>,
    Content: Parser<I, ContentOutput, E, F>,
    End: Parser<I, EndOutput, E, F>,
>(
    start: Start,
    content: Content,
    end: End,
) -> impl Parser<I, ContentOutput, E, F> {
    let parser = (start, content, end).and();
    move |input: &I| {
        let ((_, content, _), remaining) = parser.parse(input)?;
        Ok((content, remaining))
    }
}

/// Parses a key, delimiter, and value in sequence, returning the key and value as a tuple.
pub fn key_value<
    I,
    KeyOutput,
    DelimiterOutput,
    ValueOutput,
    E,
    F,
    Key: Parser<I, KeyOutput, E, F>,
    Delimiter: Parser<I, DelimiterOutput, E, F>,
    Value: Parser<I, ValueOutput, E, F>,
>(
    key: Key,
    delim: Delimiter,
    value: Value,
) -> impl Parser<I, (KeyOutput, ValueOutput), E, F> {
    let parser = (key, delim, value).and();
    move |input: &I| {
        let ((key, _, value), remaining) = parser.parse(input)?;
        Ok(((key, value), remaining))
    }
}

/// Parses a key-value pair separated by a colon (:), returning the key and value as a tuple.
pub fn header<
    I: Input,
    KeyOutput,
    ValueOutput,
    E: From<NotFound>,
    F: From<Never>,
    Key: Parser<I, KeyOutput, E, F>,
    Value: Parser<I, ValueOutput, E, F>,
>(
    key: Key,
    value: Value,
) -> impl Parser<I, (KeyOutput, ValueOutput), E, F> {
    key_value(key, ":".into_err().into_fail(), value)
}
