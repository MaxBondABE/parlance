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

/// Case-sensitive tag.
pub fn tag<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop(&s).ok_or_not_found()
}

/// Case-insensitive tag.
pub fn tag_no_case<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop_no_case(&s).ok_or_not_found()
}

/// Tag with case sensitivity determined by the `case_sensitive_tags` and `case_insensitive_tags`
/// features.
pub fn tag_auto<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    #[cfg(all(feature = "case_sensitive_tags", feature = "case_insensitive_tags"))]
    {
        compile_error!("`case_sensitive_tags` and `case_insensitive_tags` are mutually exclusive.");
    }

    #[cfg(any(
            feature = "case_sensitive_tags",
            not(any(
                // Case sensitive is the default
                feature = "case_insensitive_tags",
                feature = "case_sensitive_tags"
            ))
        ))]
    {
        return tag(s);
    }

    #[cfg(feature = "case_insensitive_tags")]
    {
        return tag_no_case(s);
    }
}

mod string_literals {
    //! Use `tag_auto()` to implement `Parser` for string literals.
    //! Because `Parser` and `PartialParser` have many methods of the same name,
    //! implementing both for string literals creates conflicts that
    //! lead to poor DX if you have both traits in scope. The compiler can't tell
    //! which trait you're referring to, and you have to use qualified paths.
    //!
    //! Since both traits are in the prelude, we can't to this by default. Thus,
    //! these feature gate.
    //! NB: `complete_tags` and `partial_tags` are NOT mutually exclusive.
    //!     `complete_tags` is the default.

    use super::*;

    #[cfg(any(
        feature = "complete_tags",
        not(any(
            // Complete tags are the default
            feature = "complete_tags",
            feature = "partial_tags"
        ))
    ))]
    mod complete_parsing {
        use super::*;

        impl<I: Input> Parser<I, I> for &str {
            fn parse(&self, input: &I) -> ParserResult<I, I> {
                return tag_auto(self).parse(input);
            }
        }
    }

    #[cfg(feature = "partial_tags")]
    mod partial_parsing {
        use super::*;

        impl<I: Input> PartialParser<I, I> for &str {
            fn partial_parse(&self, input: &I) -> crate::prelude::PartialResult<I, I> {
                tag_auto(self).as_partial().partial_parse(input)
            }
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
