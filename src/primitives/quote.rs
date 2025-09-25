use crate::{
    combinators::sandwich::sandwich,
    input::Input,
    parse::{
        Choice, Never, NotFound, Parser, ParserError, ParserResult, PartialChoice, PartialError,
        PartialOk, PartialParser, PartialResult,
    },
    primitives::tag::tag,
};

use super::take::take_until1;

pub const DOUBLE_QUOTE: char = '"';
pub const DOUBLE_QUOTE_STR: &str = "\"";
pub const SINGLE_QUOTE: char = '\'';
pub const SINGLE_QUOTE_STR: &str = "'";
pub const ESCAPE: char = '\\';
pub const ESCAPE_STR: &str = "\\";

/// Parses single-quoted strings with partial parsing support.
/// ```
/// # use parlance::primitives::quote::partial_single_quoted;
/// # use parlance::parse::PartialParser;
/// # use parlance::parse::{PartialOk, PartialResult};
/// assert_eq!(partial_single_quoted.partial_parse(&"'hello'"), Ok(PartialOk::Complete("hello", "")));
/// ```
pub fn partial_single_quoted<I: Input>(s: &I) -> PartialResult<I, I, NotFound, UnterminatedQuote> {
    let Ok((_, remaining)) = tag(SINGLE_QUOTE_STR).parse(s) else {
        return Err(PartialError::Error(NotFound, s.location()));
    };

    if let Some(idx) = find_quote_mark(SINGLE_QUOTE, remaining.as_str()) {
        let (output, r) = remaining.split_at(idx);
        let remaining = r.slice(SINGLE_QUOTE.len_utf8()..r.len());
        Ok(PartialOk::Complete(output, remaining))
    } else {
        Err(PartialError::Incomplete(UnterminatedQuote, s.location()))
    }
}

/// Parses single-quoted strings (content between single quotes).
/// ```
/// # use parlance::primitives::quote::single_quoted;
/// # use parlance::parse::Parser;
/// assert_eq!(single_quoted.parse(&"'hello' world"), Ok(("hello", " world")));
/// ```
pub fn single_quoted<I: Input>(s: &I) -> ParserResult<I, I, NotFound, UnterminatedQuote> {
    partial_single_quoted.complete().parse(s)
}

/// Parses double-quoted strings with partial parsing support.
/// ```
/// # use parlance::primitives::quote::partial_double_quoted;
/// # use parlance::parse::PartialParser;
/// # use parlance::parse::PartialOk;
/// assert_eq!(partial_double_quoted.partial_parse(&"\"hello\""), Ok(PartialOk::Complete("hello", "")));
/// ```
pub fn partial_double_quoted<I: Input>(s: &I) -> PartialResult<I, I, NotFound, UnterminatedQuote> {
    let Ok((_, remaining)) = tag(DOUBLE_QUOTE_STR).parse(s) else {
        return Err(PartialError::Error(NotFound, s.location()));
    };

    if let Some(idx) = find_quote_mark(DOUBLE_QUOTE, remaining.as_str()) {
        let (output, r) = remaining.split_at(idx);
        let remaining = r.slice(DOUBLE_QUOTE.len_utf8()..r.len());
        Ok(PartialOk::Complete(output, remaining))
    } else {
        Err(PartialError::Incomplete(UnterminatedQuote, s.location()))
    }
}

/// Parses double-quoted strings (content between double quotes).
/// ```
/// # use parlance::primitives::quote::double_quoted;
/// # use parlance::parse::Parser;
/// assert_eq!(double_quoted.parse(&"\"hello\" world"), Ok(("hello", " world")));
/// ```
pub fn double_quoted<I: Input>(s: &I) -> ParserResult<I, I, NotFound, UnterminatedQuote> {
    partial_double_quoted.complete().parse(s)
}

/// Parses either single or double-quoted strings with partial parsing support.
/// ```
/// # use parlance::primitives::quote::partial_quoted;
/// # use parlance::parse::PartialParser;
/// # use parlance::parse::PartialOk;
/// assert_eq!(partial_quoted.partial_parse(&"'hello'"), Ok(PartialOk::Complete("hello", "")));
/// ```
pub fn partial_quoted<I: Input>(s: &I) -> PartialResult<I, I, NotFound, UnterminatedQuote> {
    (partial_single_quoted, partial_double_quoted)
        .or()
        .partial_parse(s)
}

/// Parses either single or double-quoted strings.
/// ```
/// # use parlance::primitives::quote::quoted;
/// # use parlance::parse::Parser;
/// assert_eq!(quoted.parse(&"'hello' world"), Ok(("hello", " world")));
/// assert_eq!(quoted.parse(&"\"hello\" world"), Ok(("hello", " world")));
/// ```
pub fn quoted<I: Input>(s: &I) -> ParserResult<I, I, NotFound, UnterminatedQuote> {
    (single_quoted, double_quoted).or().parse(s)
}

fn find_quote_mark(quote: char, s: &str) -> Option<usize> {
    let mut iter = s.char_indices();
    let mut prev = match iter.next() {
        Some((idx, c)) if c == quote => return Some(idx),
        Some((_, c)) => c,
        None => return None,
    };
    while let Some((idx, current)) = iter.next() {
        if current == quote && prev != ESCAPE {
            return Some(idx);
        }
        prev = current;
    }

    None
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnterminatedQuote;
impl From<NotFound> for UnterminatedQuote {
    fn from(value: NotFound) -> Self {
        Self
    }
}
impl From<Never> for UnterminatedQuote {
    fn from(value: Never) -> Self {
        Self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(single_quoted.parse(&"'foo' bar"), Ok(("foo", " bar")));
        assert_eq!(quoted.parse(&"'foo' bar"), Ok(("foo", " bar")));

        assert_eq!(double_quoted.parse(&"\"foo\" bar"), Ok(("foo", " bar")));
        assert_eq!(quoted.parse(&"\"foo\" bar"), Ok(("foo", " bar")));
    }

    #[test]
    fn escaped() {
        assert_eq!(
            single_quoted.parse(&"'foo\\' bar' baz"),
            Ok(("foo\\' bar", " baz"))
        );
        assert_eq!(
            quoted.parse(&"'foo\\' bar' baz"),
            Ok(("foo\\' bar", " baz"))
        );

        assert_eq!(
            double_quoted.parse(&"\"foo\\\" bar\" baz"),
            Ok(("foo\\\" bar", " baz"))
        );
        assert_eq!(
            quoted.parse(&"\"foo\\\" bar\" baz"),
            Ok(("foo\\\" bar", " baz"))
        );
    }
}
