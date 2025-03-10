/*!
# Tags

- A string literal such as `"foo"` can be used as shorthand for `tag("foo")`

*/

use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError, ParserResult},
    util::rotate::Rotate,
};

pub fn tag<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| input.pop(&s).rot().ok_or(ParserError::Error(NotFound))
}

pub fn tag_no_case<T: AsRef<str>, I: Input>(s: T) -> impl Parser<I, I> {
    move |input: &I| {
        input
            .pop_no_case(&s)
            .rot()
            .ok_or(ParserError::Error(NotFound))
    }
}

impl<I: Input> Parser<I, I> for &str {
    fn parse(&self, input: &I) -> ParserResult<I, I> {
        tag(self).parse(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(tag::<_, _>("foo").parse(&"foobar"), Ok(("bar", "foo")));

        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"foobar"),
            Ok(("bar", "foo"))
        );
        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"FOObar"),
            Ok(("bar", "FOO"))
        );
        assert_eq!(
            tag_no_case::<_, _>("foo").parse(&"fOobar"),
            Ok(("bar", "fOo"))
        );
    }
}
