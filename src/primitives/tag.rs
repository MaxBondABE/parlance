/*!
# Tags

- A string literal such as `"foo"` can be used as shorthand for `tag("foo")`

*/

use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError, ParserResult},
    util::conditional_transforms::OrNotFound,
};

pub fn tag<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop(&s).ok_or_not_found()
}

pub fn tag_no_case<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop_no_case(&s).ok_or_not_found()
}

impl<I: Input> Parser<I, I> for &str {
    fn parse(&self, input: &I) -> ParserResult<I, I> {
        #[cfg(any(
            feature = "case_sensitive_tags",
            not(any(feature = "case_insensitive_tags", feature = "case_sensitive_tags"))
        ))]
        {
            tag(self).parse(input)
        }

        #[cfg(feature = "case_insensitive_tags")]
        {
            tag_no_case(self).parse(input)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(tag::<_, _>("foo").parse(&"foobar"), Ok(("foo", "bar")));

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
}
