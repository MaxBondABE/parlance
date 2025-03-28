use crate::parse::{Parser, Sequence};

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
        let (_, remaining) = bread.parse(input)?;
        let (output, remaining) = content.parse(input)?;
        let (_, remaining) = bread.parse(input)?;
        Ok((output, remaining))
    }
}

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
    move |input: &I| {
        let (_, remaining) = start.parse(input)?;
        let (output, remaining) = content.parse(input)?;
        let (_, remaining) = end.parse(input)?;
        Ok((output, remaining))
    }
}

pub fn key_value<
    I,
    KeyOutput,
    ContentOutput,
    ValueOutput,
    E,
    F,
    Key: Parser<I, KeyOutput, E, F>,
    Delimiter: Parser<I, ContentOutput, E, F>,
    Value: Parser<I, ValueOutput, E, F>,
>(
    start: Key,
    content: Delimiter,
    end: Value,
) -> impl Parser<I, (KeyOutput, ValueOutput), E, F> {
    move |input: &I| {
        let (key, remaining) = start.parse(input)?;
        let (_, remaining) = content.parse(input)?;
        let (value, remaining) = end.parse(input)?;
        Ok(((key, value), remaining))
    }
}
