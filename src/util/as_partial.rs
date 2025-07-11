//! Provides the `AsPartialParser` trait and the `as_partial!` macro to convert ordinary parsers
//! into partial parsers. On success, the wrapper yields `PartialOk::Complete`.

use crate::parse::{Parser, ParserError, ParserResult, PartialOk, PartialParser, PartialResult};

/// Converts a `Parser` into a `PartialParser`.
/// On success, the resulting parser always returns `PartialOk::Complete`.
pub trait AsPartialParser<Input, Output, Error, Failure> {
    fn as_partial(self) -> impl PartialParser<Input, Output, Error, Failure>;
}
impl<I, O, E, F, T: Parser<I, O, E, F>> AsPartialParser<I, O, E, F> for T {
    fn as_partial(self) -> impl PartialParser<I, O, E, F> {
        move |input: &I| match self.parse(input) {
            Ok((o, r)) => Ok(PartialOk::Complete(o, r)),
            Err(e) => Err(e.into()),
        }
    }
}

macro_rules! as_partial_inner (
    ($($x: expr)*) => {
        ( $($x.as_partial(), )* )
    }
);

/// Call `as_partial()` on each component parser.
#[macro_export]
macro_rules! as_partial (
    ($($x: expr),*) => {
        (
            as_partial_inner!($($x)*)
        )
    };
    ($($x: expr ,)+) => {
        (
            as_partial_inner!($($x)*)
        )
    };
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        input::Input,
        parse::{Missing, Parser, PartialParser, PartialSequence, Sequence},
    };

    #[test]
    fn as_partial_macro_is_insensitive_to_trailing_commas() {
        // With trailing comma
        assert_eq!(
            as_partial!("foo".into_fail::<Missing>(), "bar".into_fail(),)
                .and()
                .partial_parse(&"foobarbaz"),
            Ok((PartialOk::Complete(("foo", "bar"), "baz")))
        );

        // Without trailing comma
        assert_eq!(
            as_partial!("foo".into_fail::<Missing>(), "bar".into_fail())
                .and()
                .partial_parse(&"foobarbaz"),
            Ok((PartialOk::Complete(("foo", "bar"), "baz")))
        );
    }
}
