use crate::{
    input::Input,
    parse::{Choice, NotFound, Parser, ParserError, ParserResult, StreamingResult},
    util::conditional_transforms::MaybeCompleteIf,
    util::conditional_transforms::StreamingOrNotFound,
};

use super::tag::tag;

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
        Ok((s.empty(), s.clone()))
    }
}

pub fn line_stream<I: Input>(s: &I) -> StreamingResult<I, I> {
    s.take_while(|c| c != '\n')
        .as_complete_if(|_, remaining| !remaining.is_empty())
        .ok_or_not_found()
}

pub fn eol<I: Input>(s: &I) -> ParserResult<I, I> {
    ("\n", "\r\n").or().parse(s)
}

pub fn eof<I: Input, F>(s: &I) -> ParserResult<I, ()> {
    if s.len() == 0 {
        Ok(((), s.clone()))
    } else {
        Err(ParserError::Error(NotFound))
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
