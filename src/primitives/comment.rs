use crate::{
    combinators::seek::seek,
    input::Input,
    parse::{Choice, FuseSequence, Parser, ParserResult, PartialResult},
    primitives::line,
    strfuse,
};

/// C-style single-line comment (// ... ).
pub fn single_line_comment<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("//", line)).parse(s)
}

/// C-style single-line comment (/* ... */).
pub fn partial_single_line_comment<I: Input>(s: &I) -> PartialResult<I, I> {
    //strfuse!(("//", line)).parse(s)
    todo!()
}

/// C-style multi-line comment (/* ... */).
pub fn multi_line_comment<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("/*", seek("*/"), "*/")).parse(s)
}

/// C-style multi-line comment (/* ... */).
pub fn partial_multi_line_comment<I: Input>(s: &I) -> PartialResult<I, I> {
    //("/*", take_until("*/"), "*/").fuse().parse(s)
    todo!()
}

/// C-style single-line (// ...) or multi-line (/* ... */) comment.
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
            Err(ParserError::Error(NotFound))
        );
        assert_eq!(
            multi_line_comment.parse(&"foo"),
            Err(ParserError::Error(NotFound))
        );
        assert_eq!(comment.parse(&"foo"), Err(ParserError::Error(NotFound)));
    }

    #[test]
    fn wrong_kind_of_comment() {
        assert_eq!(
            single_line_comment.parse(&"/* foo */"),
            Err(ParserError::Error(NotFound))
        );
        assert_eq!(
            multi_line_comment.parse(&"//"),
            Err(ParserError::Error(NotFound))
        );
    }
}
