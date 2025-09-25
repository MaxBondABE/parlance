use crate::{
    combinators::seek::seek,
    input::Input,
    parse::{Choice, FuseSequence, Parser, ParserResult, PartialResult},
    primitives::line,
    strfuse,
};

/// Parses C-style single-line comments starting with `//`.
/// ```
/// # use parlance::primitives::comment::single_line_comment;
/// # use parlance::parse::Parser;
/// assert_eq!(single_line_comment.parse(&"// foo\nbar"), Ok(("// foo\n", "bar")));
/// ```
pub fn single_line_comment<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("//", line)).parse(s)
}

/// Parses C-style multi-line comments enclosed in `/* */`.
/// ```
/// # use parlance::primitives::comment::multi_line_comment;
/// # use parlance::parse::Parser;
/// assert_eq!(multi_line_comment.parse(&"/* hello world */"), Ok(("/* hello world */", "")));
/// ```
pub fn multi_line_comment<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("/*", seek("*/"), "*/")).parse(s)
}

/// Parses either single-line or multi-line C-style comments.
/// ```
/// # use parlance::primitives::comment::{comment, Comment};
/// # use parlance::parse::Parser;
/// assert_eq!(comment.parse(&"// foo"), Ok((Comment::SingleLine("// foo"), "")));
/// assert_eq!(comment.parse(&"/* foo\nbar */"), Ok((Comment::MultiLine("/* foo\nbar */"), "")));
/// ```
pub fn comment<I: Input>(s: &I) -> ParserResult<I, Comment<I>> {
    (
        single_line_comment.map(Comment::SingleLine),
        multi_line_comment.map(Comment::MultiLine),
    )
        .or()
        .parse(s)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Comment<I> {
    SingleLine(I),
    MultiLine(I),
}
impl<I> Comment<I> {
    pub fn unwrap(self) -> I {
        match self {
            Comment::SingleLine(x) | Comment::MultiLine(x) => x,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse::{NotFound, ParserError};

    use super::*;

    // NB: We don't need to test both \n and \r\n cases, because these are handled
    // in `line`.

    #[test]
    fn single_line() {
        assert_eq!(single_line_comment.parse(&"// foo"), Ok(("// foo", "")));
        assert_eq!(
            single_line_comment.parse(&"// foo\nbar"),
            Ok(("// foo\n", "bar"))
        );

        assert_eq!(
            comment.parse(&"// foo"),
            Ok((Comment::SingleLine("// foo"), ""))
        );
        assert_eq!(
            comment.parse(&"// foo\nbar"),
            Ok((Comment::SingleLine("// foo\n"), "bar"))
        );
    }

    #[test]
    fn multi_line() {
        assert_eq!(
            multi_line_comment.parse(&"/* foo */"),
            Ok(("/* foo */", ""))
        );
        assert_eq!(
            multi_line_comment.parse(&"/* foo\nbar */"),
            Ok(("/* foo\nbar */", ""))
        );

        assert_eq!(
            multi_line_comment.parse(&"/* foo */baz"),
            Ok(("/* foo */", "baz"))
        );
        assert_eq!(
            multi_line_comment.parse(&"/* foo\nbar */baz"),
            Ok(("/* foo\nbar */", "baz"))
        );
    }

    #[test]
    fn not_a_comment() {
        assert_eq!(
            single_line_comment.parse(&"foo"),
            Err(ParserError::Error(NotFound, ()))
        );
        assert_eq!(
            multi_line_comment.parse(&"foo"),
            Err(ParserError::Error(NotFound, ()))
        );
        assert_eq!(comment.parse(&"foo"), Err(ParserError::Error(NotFound, ())));
    }

    #[test]
    fn wrong_kind_of_comment() {
        assert_eq!(
            single_line_comment.parse(&"/* foo */"),
            Err(ParserError::Error(NotFound, ()))
        );
        assert_eq!(
            multi_line_comment.parse(&"//"),
            Err(ParserError::Error(NotFound, ()))
        );
    }
}
