use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserResult, PartialResult},
    util::{
        as_partial::AsPartialParser,
        conditional_transforms::{CompleteIf, EitherCompleteIf, OkOrNotFound},
    },
};

pub fn whitespace<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_whitespace()).ok_or_not_found()
}

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
