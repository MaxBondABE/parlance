/*!
# Tags

- A string literal such as `"foo"` can be used as shorthand for `tag("foo")`
- The feature gates

*/

use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError, ParserResult, PartialParser},
    util::{as_partial::AsPartialParser, conditional_transforms::OkOrNotFound},
};

pub fn tag<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop(&s).ok_or_not_found()
}

pub fn tag_no_case<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop_no_case(&s).ok_or_not_found()
}

impl<I: Input> Parser<I, I> for &str {
    fn parse(&self, input: &I) -> ParserResult<I, I> {
        #[cfg(all(feature = "case_sensitive_tags", feature = "case_insensitive_tags"))]
        {
            compile_error!(
                "`case_sensitive_tags` and `case_insensitive_tags` are mutually exclusive."
            );
        }

        #[cfg(any(
            feature = "case_sensitive_tags",
            // Case sensitive is the default
            not(any(feature = "case_insensitive_tags", feature = "case_sensitive_tags"))
        ))]
        {
            return tag(self).parse(input);
        }

        #[cfg(feature = "case_insensitive_tags")]
        {
            return tag_no_case(self).parse(input);
        }
    }
}

#[cfg(feature = "partial_tags")]
mod partial_parsing {
    /// Because `Parser` and `PartialParser` have many methods of the same name,
    /// implementing `PartialParser` for string literals creates conflicts that
    /// lead to poor DX if you have both traits in scope. The compiler can't tell
    /// which trait you're referring to, and you have to use qualified paths.
    ///
    /// Since both traits are in the prelude, we can't to this by default. Thus,
    /// this feature gate.
    impl<I: Input> PartialParser<I, I> for &str {
        fn partial_parse(&self, input: &I) -> crate::prelude::PartialResult<I, I> {
            // NB: Because we are using the implementation of Parser on string literals
            // which is already there, we are correcly handling case sensitivy implicitly.
            self.as_partial().partial_parse(input)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn case_sensitive_tags() {
        assert_eq!(tag::<_, _>("foo").parse(&"foobar"), Ok(("foo", "bar")));
        assert_eq!(
            tag::<_, _>("foo").parse(&"Foobar"),
            Err(ParserError::Error(NotFound))
        );
    }

    #[test]
    fn case_insensitive_tags() {
        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"foobar"),
            Ok(("foo", "bar"))
        );
        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"FOObar"),
            Ok(("FOO", "bar"))
        );
        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"fOobar"),
            Ok(("fOo", "bar"))
        );
    }

    #[test]
    fn literal_tags() {
        /// Avoids confusion with core::str::parse
        fn parse(s: &str) -> ParserResult<&str, &str> {
            let tag = "foo";
            Parser::parse(&tag, &s)
        }

        #[cfg(any(
            feature = "case_sensitive_tags",
            not(any(feature = "case_insensitive_tags", feature = "case_sensitive_tags"))
        ))]
        {
            assert_eq!(parse(&"foo"), Ok(("foo", "")));
            assert_eq!(parse(&"FOO"), Err(ParserError::Error(NotFound)));
            assert_eq!(parse(&"bar"), Err(ParserError::Error(NotFound)));
        }

        #[cfg(feature = "case_insensitive_tags")]
        {
            assert_eq!(parse(&"foo"), Ok(("foo", "")));
            assert_eq!(parse(&"FOO"), Ok(("FOO", "")));
            assert_eq!(parse(&"bar"), Err(ParserError::Error(NotFound)));
        }
    }
}
