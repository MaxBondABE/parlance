use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserResult, PartialResult},
    util::{
        as_partial::AsPartialParser,
        conditional_transforms::{CompleteIf, EitherCompleteIf, OkOrNotFound},
    },
};

/// Parses one or more whitespace characters.
/// ```
/// # use parlance::primitives::whitespace::whitespace;
/// # use parlance::parse::Parser;
/// assert_eq!(whitespace.parse(&"   hello"), Ok(("   ", "hello")));
/// assert_eq!(whitespace.parse(&"\t\n world"), Ok(("\t\n ", "world")));
/// ```
pub fn whitespace<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_whitespace())
        .ok_or_not_found(s.location())
}

/// Parses whitespace with partial parsing support for streaming input.
/// ```
/// # use parlance::primitives::whitespace::partial_whitespace;
/// # use parlance::parse::PartialParser;
/// # use parlance::parse::{PartialOk, PartialResult};
/// # assert_eq!(partial_whitespace.partial_parse(&"   hello"), Ok(PartialOk::Complete("   ", "hello")));
/// ```
pub fn partial_whitespace<I: Input>(s: &I) -> PartialResult<I, I> {
    whitespace.parse(s).has_stopped()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::{PartialOk, PartialParser};

    #[test]
    fn single_whitespace() {
        assert_eq!(whitespace.parse(&" foo"), Ok((" ", "foo")));
        assert_eq!(whitespace.parse(&"\tfoo"), Ok(("\t", "foo")));
        assert_eq!(whitespace.parse(&"\nfoo"), Ok(("\n", "foo")));
        assert_eq!(whitespace.parse(&"\r\nfoo"), Ok(("\r\n", "foo")));
    }

    #[test]
    fn multiple_whitespace() {
        assert_eq!(whitespace.parse(&"    foo"), Ok(("    ", "foo")));
        assert_eq!(whitespace.parse(&"  \t  foo"), Ok(("  \t  ", "foo")));
        assert_eq!(
            whitespace.parse(&"  \t  \n  foo"),
            Ok(("  \t  \n  ", "foo"))
        );
    }

    #[test]
    fn partial() {
        assert_eq!(
            partial_whitespace.partial_parse(&"    "),
            Ok(PartialOk::Partial("    ", ""))
        );
        assert_eq!(
            partial_whitespace.partial_parse(&"    x"),
            Ok(PartialOk::Complete("    ", "x"))
        );
    }
}
