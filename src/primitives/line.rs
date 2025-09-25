use crate::{
    input::Input,
    parse::{Choice, NotFound, Parser, ParserError, ParserResult, PartialResult},
    util::conditional_transforms::MaybeCompleteIf,
    util::conditional_transforms::PartialOkOrNotFound,
};

use super::tag::tag;

/// Parses a single line including the line ending.
/// ```
/// # use parlance::primitives::line::line;
/// # use parlance::parse::Parser;
/// assert_eq!(line.parse(&"hello\nworld"), Ok(("hello\n", "world")));
/// assert_eq!(line.parse(&"hello"), Ok(("hello", "")));
/// ```
pub fn line<I: Input>(s: &I) -> ParserResult<I, I> {
    if let Some((line, remaining)) = s.pop(&"\n") {
        Ok((line, remaining))
    } else if let Some((line, remaining)) = s.take_while(|c| c != '\n') {
        if remaining.as_str().starts_with('\n') {
            let (line, remaining) = s.split_at(line.len() + '\n'.len_utf8());
            Ok((line, remaining))
        } else {
            Ok((line, remaining))
        }
    } else {
        Ok((s.take_none(), s.clone()))
    }
}

/// Parses a line with partial parsing support for streaming input.
/// ```
/// # use parlance::primitives::line::partial_line;
/// # use parlance::parse::PartialParser;
/// # use parlance::parse::PartialOk;
/// assert_eq!(partial_line.partial_parse(&"hello world"), Ok(PartialOk::Partial("hello world", "")));
/// ```
pub fn partial_line<I: Input>(s: &I) -> PartialResult<I, I> {
    s.take_while(|c| c != '\n')
        .as_complete_if(|_, remaining| !remaining.is_empty())
        .ok_or_not_found(s.location())
}

/// Parses end-of-line characters (`\n` or `\r\n`).
/// ```
/// # use parlance::primitives::line::eol;
/// # use parlance::parse::Parser;
/// assert_eq!(eol.parse(&"\nrest"), Ok(("\n", "rest")));
/// assert_eq!(eol.parse(&"\r\nrest"), Ok(("\r\n", "rest")));
/// ```
pub fn eol<I: Input>(s: &I) -> ParserResult<I, I> {
    ("\n", "\r\n").or().parse(s)
}

/// Parses end-of-file (succeeds only when input is empty).
/// ```
/// # use parlance::primitives::line::eof;
/// # use parlance::parse::Parser;
/// # use parlance::parse::ParserError;
/// # use parlance::parse::NotFound;
/// assert_eq!(eof.parse(&""), Ok(((), "")));
/// assert_eq!(eof.parse(&"foo"), Err(ParserError::Error(NotFound, ())));
/// ```
pub fn eof<I: Input>(s: &I) -> ParserResult<I, ()> {
    if s.len() == 0 {
        Ok(((), s.clone()))
    } else {
        Err(ParserError::Error(NotFound, s.location()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(line.parse(&""), Ok(("", "")));
    }

    #[test]
    fn blank_line() {
        assert_eq!(line.parse(&"\n\n"), Ok(("\n", "\n")));
        assert_eq!(line.parse(&"\r\n\r\n"), Ok(("\r\n", "\r\n")));
    }

    #[test]
    fn no_newline() {
        assert_eq!(line.parse(&"foo"), Ok(("foo", "")));
    }

    #[test]
    fn linebreak_is_present() {
        assert_eq!(line.parse(&"foo\n"), Ok(("foo\n", "")));
        assert_eq!(line.parse(&"foo\nbar"), Ok(("foo\n", "bar")));

        assert_eq!(line.parse(&"foo\r\n"), Ok(("foo\r\n", "")));
        assert_eq!(line.parse(&"foo\r\nbar"), Ok(("foo\r\n", "bar")));
    }

    #[test]
    fn multiple_linebreaks() {
        assert_eq!(line.parse(&"foo\nbar\nbaz"), Ok(("foo\n", "bar\nbaz")));
        assert_eq!(
            line.parse(&"foo\r\nbar\r\nbaz"),
            Ok(("foo\r\n", "bar\r\nbaz"))
        );
    }
}
