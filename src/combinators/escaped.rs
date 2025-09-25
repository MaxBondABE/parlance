use crate::{
    input::{transform::TransformContent, Input},
    parse::{Parser, ParserError},
};

/// Processes escape sequences in text using a provided escape sequence parser.
/// Repeatedly applies the escape parser to transform escaped content into its unescaped form.
/// ```
/// # use parlance::combinators::escaped::{escape, escape_backslash};
/// # use parlance::parse::Parser;
/// let parser = escape(escape_backslash!("n" "t"));
/// let result = parser.parse(&"hello\\nworld".to_string());
/// ```
pub fn escape<
    I: Input + TransformContent,
    E,
    F,
    U: AsRef<str> + std::fmt::Debug,
    EscSeq: Parser<I, EscapeToken<I, U>, E, F>,
>(
    esc: EscSeq,
) -> impl Parser<I, <I as TransformContent>::Transformed, E, F>
where
    I::Transformed: TransformContent<Transformed = I::Transformed>,
{
    move |input: &I| {
        let (o, mut remaining) = esc.parse(input)?;
        let mut output = input.to_content(o.as_str().to_string());
        while !remaining.is_empty() {
            match esc.parse(&remaining) {
                Ok((o, r)) => {
                    output = output.append_content(o);
                    remaining = r;
                }
                Err(ParserError::Error(_, _)) => break,
                Err(e) => return Err(e),
            }
        }

        Ok((output, remaining))
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum EscapeToken<I, U = &'static str> {
    Unescaped(I),
    Escaped(U),
}
impl<I: Input, U: AsRef<str>> EscapeToken<I, U> {
    pub fn as_str(&self) -> &str {
        match self {
            EscapeToken::Unescaped(s) => s.as_str(),
            EscapeToken::Escaped(s) => s.as_ref(),
        }
    }
}
impl<I: Input, U: AsRef<str>> AsRef<str> for EscapeToken<I, U> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidEscapeSequence;

#[macro_export]
/// Creates an escape sequence parser for a specific escape character and sequences.
/// Returns a parser that recognizes unescaped text or specific escape sequences.
/// ```
/// # use parlance::escape_character;
/// # use parlance::combinators::escaped::EscapeToken;
/// # use parlance::parse::Parser;
/// let parser = escape_character!("\\", "n" "t");
/// assert_eq!(parser.parse(&"hello"), Ok((EscapeToken::Unescaped("hello"), "")));
/// assert_eq!(parser.parse(&"\\n"), Ok((EscapeToken::Escaped("n"), "")));
/// ```
macro_rules! escape_character (
    ($esc: literal, $($seq: literal)*) => {
        {
            fn escape_impl<I: $crate::input::Input>(s: &I) -> $crate::parse::ParserResult<
                I,
                $crate::combinators::escaped::EscapeToken<I>,
                $crate::parse::Never,
                $crate::combinators::escaped::InvalidEscapeSequence,
            > {

                if let Some((_, s)) = s.pop(&$esc) {
                    $(
                        if let Some((_, remaining)) = s.pop(&$seq) {
                            return Ok((
                                $crate::combinators::escaped::EscapeToken::Escaped($seq),
                                remaining
                            ));
                        }
                    )*;

                    Err($crate::parse::ParserError::Failure(
                        $crate::combinators::escaped::InvalidEscapeSequence,
                        s.location(),
                    ))
                } else {
                    let (o, remaining) = s.as_str()
                        .find($esc)
                        .map(|idx| s.split_at(idx))
                        .unwrap_or_else(|| s.take_all());

                    Ok((
                        $crate::combinators::escaped::EscapeToken::Unescaped(o),
                        remaining
                    ))
                }
            }

            escape_impl
        }
    }
);

pub use escape_character;

#[macro_export]
/// Creates a backslash escape sequence parser for the specified sequences.
/// Convenience macro that uses "\\" as the escape character.
/// ```
/// # use parlance::escape_backslash;
/// # use parlance::combinators::escaped::EscapeToken;
/// # use parlance::parse::Parser;
/// let parser = escape_backslash!("n" "t" "r");
/// assert_eq!(parser.parse(&"\\n rest"), Ok((EscapeToken::Escaped("n"), " rest")));
/// assert_eq!(parser.parse(&"normal"), Ok((EscapeToken::Unescaped("normal"), "")));
/// ```
macro_rules! escape_backslash (
    ($($seq: literal)*) => {
        $crate::combinators::escaped::escape_character!("\\", $($seq)*)
    }
);

pub use escape_backslash;

#[cfg(test)]
mod test {
    use crate::parse::ParserResult;

    use super::*;

    #[test]
    fn simple() {
        fn never_escape<I: Input>(s: &I) -> ParserResult<I, EscapeToken<I>> {
            Ok((EscapeToken::Unescaped(s.clone()), s.take_none()))
        }
        assert_eq!(
            escape(never_escape).parse(&"foo"),
            Ok(("foo".to_string(), ""))
        );

        fn always_escape<I: Input>(s: &I) -> ParserResult<I, EscapeToken<I>> {
            Ok((EscapeToken::Escaped("bar"), s.take_none()))
        }
        assert_eq!(
            escape(always_escape).parse(&"foo"),
            Ok(("bar".to_string(), ""))
        );
    }

    #[test]
    fn backslash_parser() {
        let parser = escape_backslash!("a" "b");
        assert_eq!(
            parser.parse(&"foo \\a bar"),
            Ok((EscapeToken::Unescaped("foo "), "\\a bar"))
        );
        assert_eq!(
            parser.parse(&"foo \\b bar"),
            Ok((EscapeToken::Unescaped("foo "), "\\b bar"))
        );

        assert_eq!(
            parser.parse(&"\\a bar"),
            Ok((EscapeToken::Escaped("a"), " bar"))
        );
        assert_eq!(
            parser.parse(&"\\b bar"),
            Ok((EscapeToken::Escaped("b"), " bar"))
        );

        assert_eq!(
            parser.parse(&" bar"),
            Ok((EscapeToken::Unescaped(" bar"), ""))
        );
    }

    #[test]
    fn backslash_escape() {
        let parser = escape(escape_backslash!("a" "b"));
        assert_eq!(
            parser.parse(&"foo \\a bar"),
            Ok(("foo a bar".to_string(), ""))
        );
        assert_eq!(
            parser.parse(&"foo \\b bar"),
            Ok(("foo b bar".to_string(), ""))
        );

        assert_eq!(parser.parse(&"foo bar"), Ok(("foo bar".to_string(), "")));

        assert_eq!(
            parser.parse(&"foo \\ bar"),
            Err(ParserError::Failure(InvalidEscapeSequence, ()))
        );
    }

    // TODO test escape_character
}
