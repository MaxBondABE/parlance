use crate::{
    input::{transform::TransformContent, Input},
    parse::{Parser, ParserError},
};

fn escape<
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
        let (mut remaining, o) = esc.parse(input)?;
        let mut output = input.to_content(o.as_str().to_string());
        while !remaining.is_empty() {
            match esc.parse(&remaining) {
                Ok((r, o)) => {
                    output = output.append_content(o);
                    remaining = r;
                }
                Err(ParserError::Error(_)) => break,
                Err(e) => return Err(e),
            }
        }

        Ok((remaining, output))
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
macro_rules! escape_character (
    ($esc: literal, $($seq: literal)*) => {
        {
            use $crate::util::rotate::Rotate;

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
                                remaining,
                                $crate::combinators::escaped::EscapeToken::Escaped($seq)
                            ));
                        }
                    )*;

                    Err($crate::parse::ParserError::Failure(
                        $crate::combinators::escaped::InvalidEscapeSequence,
                    ))
                } else {
                    let (remaining, t) = s.as_str()
                        .find($esc)
                        .map(|idx| s.split_at(idx))
                        .unwrap_or_else(|| (s.clone(), s.empty()))
                        .rot();

                    Ok((
                        remaining,
                        $crate::combinators::escaped::EscapeToken::Unescaped(t)
                    ))
                }
            }

            escape_impl
        }
    }
);

pub use escape_character;

#[macro_export]
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
            Ok((s.empty(), EscapeToken::Unescaped(s.clone())))
        }
        assert_eq!(
            escape(never_escape).parse(&"foo"),
            Ok(("", "foo".to_string()))
        );

        fn always_escape<I: Input>(s: &I) -> ParserResult<I, EscapeToken<I>> {
            Ok((s.empty(), EscapeToken::Escaped("bar")))
        }
        assert_eq!(
            escape(always_escape).parse(&"foo"),
            Ok(("", "bar".to_string()))
        );
    }

    #[test]
    fn backslash_parser() {
        let parser = escape_backslash!("a" "b");
        assert_eq!(
            parser.parse(&"foo \\a bar"),
            Ok(("\\a bar", EscapeToken::Unescaped("foo ")))
        );
        assert_eq!(
            parser.parse(&"foo \\b bar"),
            Ok(("\\b bar", EscapeToken::Unescaped("foo ")))
        );

        assert_eq!(
            parser.parse(&"\\a bar"),
            Ok((" bar", EscapeToken::Escaped("a")))
        );
        assert_eq!(
            parser.parse(&"\\b bar"),
            Ok((" bar", EscapeToken::Escaped("b")))
        );

        assert_eq!(
            parser.parse(&" bar"),
            Ok(("", EscapeToken::Unescaped(" bar")))
        );
    }

    #[test]
    fn backslash_escape() {
        let parser = escape(escape_backslash!("a" "b"));
        assert_eq!(
            parser.parse(&"foo \\a bar"),
            Ok(("", "foo a bar".to_string()))
        );
        assert_eq!(
            parser.parse(&"foo \\b bar"),
            Ok(("", "foo b bar".to_string()))
        );

        assert_eq!(parser.parse(&"foo bar"), Ok(("", "foo bar".to_string())));

        assert_eq!(
            parser.parse(&"foo \\ bar"),
            Err(ParserError::Failure(InvalidEscapeSequence))
        );
    }

    // TODO test escape_character
}
