use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

/// Takes zero or more characters matching the predicate.
/// ```
/// # use parlance::primitives::take::take_while0;
/// # use parlance::parse::Parser;
/// assert_eq!(take_while0(|c| c == 'a').parse(&"aaabbb"), Ok(("aaa", "bbb")));
/// assert_eq!(take_while0(|c| c == 'x').parse(&"aaabbb"), Ok(("", "aaabbb")));
/// ```
pub fn take_while0<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_while(&predicate) {
            Ok((s, remaining))
        } else {
            Ok((input.take_none(), input.clone()))
        }
    }
}

/// Takes one or more characters matching the predicate.
/// ```
/// # use parlance::primitives::take::take_while;
/// # use parlance::parse::Parser;
/// assert_eq!(take_while(|c| c == 'a').parse(&"aaabbb"), Ok(("aaa", "bbb")));
/// ```
pub fn take_while<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_while(&predicate) {
            Ok((s, remaining))
        } else {
            Err(ParserError::Error(NotFound, input.location()))
        }
    }
}

/// Takes characters until the predicate matches (or end of input).
/// ```
/// # use parlance::primitives::take::take_until;
/// # use parlance::parse::Parser;
/// assert_eq!(take_until(|c| c == 'b').parse(&"aaabbb"), Ok(("aaa", "bbb")));
/// assert_eq!(take_until(|c| c == 'x').parse(&"aaabbb"), Ok(("aaabbb", "")));
/// ```
pub fn take_until<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_until(&predicate) {
            Ok((s, remaining))
        } else {
            Ok(input.take_all())
        }
    }
}

/// Takes characters until the predicate matches. Fails if predicate never matches.
/// ```
/// # use parlance::primitives::take::take_until1;
/// # use parlance::parse::Parser;
/// assert_eq!(take_until1(|c| c == 'b').parse(&"aaabbb"), Ok(("aaa", "bbb")));
/// ```
pub fn take_until1<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_until(&predicate) {
            Ok((s, remaining))
        } else {
            Err(ParserError::Error(NotFound, input.location()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn take_while_with_match() {
        assert_eq!(take_while(|c| c == 'f').parse(&"foo"), Ok(("f", "oo")));
        assert_eq!(take_while0(|c| c == 'f').parse(&"foo"), Ok(("f", "oo")));
    }

    #[test]
    fn take_while_without_match() {
        assert_eq!(
            take_while(|_| false).parse(&"foo"),
            Err(ParserError::Error(NotFound, ()))
        );
        assert_eq!(take_while0(|_| false).parse(&"foo"), Ok(("", "foo")));
    }

    #[test]
    fn take_until_with_match() {
        assert_eq!(take_until(|c| c == 'o').parse(&"foo"), Ok(("f", "oo")));
        assert_eq!(take_until1(|c| c == 'o').parse(&"foo"), Ok(("f", "oo")));
    }

    #[test]
    fn take_until_without_match() {
        assert_eq!(
            take_until1(|_| false).parse(&"foo"),
            Err(ParserError::Error(NotFound, ()))
        );
        assert_eq!(take_until(|_| false).parse(&"foo"), Ok(("foo", "")));
    }
}
