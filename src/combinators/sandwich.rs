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
        let (remaining, _) = bread.parse(input)?;
        let (remaining, output) = content.parse(input)?;
        let (remaining, _) = bread.parse(input)?;
        Ok((remaining, output))
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
        let (remaining, _) = start.parse(input)?;
        let (remaining, output) = content.parse(input)?;
        let (remaining, _) = end.parse(input)?;
        Ok((remaining, output))
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
        let (remaining, key) = start.parse(input)?;
        let (remaining, _) = content.parse(input)?;
        let (remaining, value) = end.parse(input)?;
        Ok((remaining, (key, value)))
    }
}
