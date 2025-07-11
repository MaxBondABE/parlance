use crate::util::tuples::implement_for_tuples;

use super::{Never, NotFound, Parser, ParserError, PartialError, PartialParser};

/// A tuple of parsers. Returns the first to succeed.
pub trait Choice<Input, Output, Error = NotFound, Failure = Never> {
    fn or(self) -> impl Parser<Input, Output, NotFound, Failure>;
}

/// A tuple of partial parsers. Returns the first to succeed.
pub trait PartialChoice<Input, Output, Error = NotFound, Failure = Never> {
    fn or(self) -> impl PartialParser<Input, Output, NotFound, Failure>;
}

macro_rules! choice_impl (
    ($($idx: literal)* . $last: literal) => {
        paste::paste! {
            impl<
                Input,
                Output,
                Error,
                Failure,
                $([<P $idx>]: Parser<Input, Output, Error, Failure>, )*
                [<P $last>]: Parser<Input, Output, Error, Failure>,
                > Choice<Input, Output, Error, Failure> for ($([<P $idx>], )* [<P $last>])
            {
                fn or(self) -> impl Parser<Input, Output, NotFound, Failure> {
                    move |input: &Input| {
                        $(
                            match self.$idx.parse(input) {
                                Ok(x) => return Ok(x),
                                Err(ParserError::Error(_)) => (),
                                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
                            }
                        )*

                        match self.$last.parse(input) {
                            Ok(x) => Ok(x),
                            Err(ParserError::Error(_)) => Err(ParserError::Error(NotFound)),
                            Err(ParserError::Failure(e)) => Err(ParserError::Failure(e)),
                        }
                    }
                }
            }

            impl<
                Input,
                Output,
                Error,
                Failure,
                $([<P $idx>]: PartialParser<Input, Output, Error, Failure>, )*
                [<P $last>]: PartialParser<Input, Output, Error, Failure>,
                > PartialChoice<Input, Output, Error, Failure> for ($([<P $idx>], )* [<P $last>])
            {
                fn or(self) -> impl PartialParser<Input, Output, NotFound, Failure> {
                    move |input: &Input| {
                        $(
                            match self.$idx.partial_parse(input) {
                                Ok(x) => return Ok(x),
                                Err(PartialError::Error(_)) => (),
                                Err(PartialError::Incomplete(e)) => return Err(PartialError::Incomplete(e)),
                                Err(PartialError::Failure(e)) => return Err(PartialError::Failure(e)),
                            }
                        )*

                        match self.$last.partial_parse(input) {
                            Ok(x) => Ok(x),
                            Err(PartialError::Error(_)) => Err(PartialError::Error(NotFound)),
                            Err(PartialError::Incomplete(e)) => return Err(PartialError::Incomplete(e)),
                            Err(PartialError::Failure(e)) => Err(PartialError::Failure(e)),
                        }
                    }
                }
            }
        }
    }
);

implement_for_tuples!(choice_impl);

#[cfg(test)]
mod test {
    use crate::{input::Input, parse::ParserResult};

    use super::*;

    #[test]
    fn first_to_succeed_takes_precedence() {
        #[derive(Eq, PartialEq, Debug)]
        enum Output {
            Foo,
            Bar,
        }
        fn foo<I: Input>(s: &I) -> ParserResult<I, Output> {
            Ok((Output::Foo, s.clone()))
        }

        struct Bar;
        fn bar<I: Input>(s: &I) -> ParserResult<I, Output> {
            Ok((Output::Bar, s.clone()))
        }

        assert_eq!((foo, bar).or().parse(&"baz"), Ok((Output::Foo, "baz")));
        assert_eq!((bar, foo).or().parse(&"baz"), Ok((Output::Bar, "baz")));
    }

    #[test]
    fn recoverable_errors_are_ignored() {
        fn always_succeeds<I: Input>(s: &I) -> ParserResult<I, ()> {
            Ok(((), s.clone()))
        }

        fn always_not_found<I: Input>(s: &I) -> ParserResult<I, ()> {
            Err(ParserError::Error(NotFound))
        }

        assert_eq!(
            (always_not_found, always_succeeds).or().parse(&"foo"),
            Ok(((), "foo"))
        );
    }

    #[test]
    fn smoke() {
        assert_eq!(("foo", "bar").or().parse(&"foo"), Ok(("foo", "")));
        assert_eq!(("foo", "bar").or().parse(&"bar"), Ok(("bar", "")));
        assert_eq!(
            ("foo", "bar").or().parse(&"not_there"),
            Err(ParserError::Error(NotFound))
        );

        assert_eq!(("bar", "foo").or().parse(&"foo"), Ok(("foo", "")));
        assert_eq!(("bar", "foo").or().parse(&"bar"), Ok(("bar", "")));
        assert_eq!(
            ("bar", "foo").or().parse(&"not_there"),
            Err(ParserError::Error(NotFound))
        );
    }
}
