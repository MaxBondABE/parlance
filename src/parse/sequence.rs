use std::marker::PhantomData;

use crate::{
    primitives::whitespace::{partial_whitespace, whitespace},
    util::{conditional_transforms::NoPartial, tuples::implement_for_tuples},
};

use super::{Fusable, Missing, Never, NotFound, Parser, PartialError, PartialOk, PartialParser};

/// A set of parsers which consume the input in series, failing if any
/// one of them returns an error.
pub trait Sequence<Input, Output, Error = NotFound, Failure = Never> {
    fn and(self) -> impl Parser<Input, Output, Error, Failure>;
    fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, impl Parser<Input, ()>>
    where
        Self: Sized,
        P: Parser<Input, O>,
    {
        SeparatedSequence {
            seq: self,
            sep: sep.with_output(()),
            _marker: PhantomData,
        }
    }
    fn whitespace(self) -> SeparatedSequence<Self, impl Parser<Input, ()>, ApplySeperatorIfNotEmpty>
    where
        Self: Sized,
        Input: crate::input::Input,
    {
        self.with_sep(whitespace).seperate_if_not_empty()
    }
}

/// A set of partial parsers which consume the input in series, failing if any
/// one of them returns an error.
pub trait PartialSequence<Input, Output, Error = NotFound, Failure = Never> {
    fn and(self) -> impl PartialParser<Input, Output, Error, Failure>;
    fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, impl PartialParser<Input, ()>>
    where
        Self: Sized,
        P: PartialParser<Input, O>,
    {
        SeparatedSequence {
            seq: self,
            sep: sep.with_output(()),
            _marker: PhantomData,
        }
    }
    fn whitespace(
        self,
    ) -> SeparatedSequence<Self, impl PartialParser<Input, ()>, ApplySeperatorIfNotEmpty>
    where
        Self: Sized,
        Input: crate::input::Input,
    {
        self.with_sep(partial_whitespace).seperate_if_not_empty()
    }
}

/// A `Sequence` where each component (taken from `S`) is separated by elements
/// identified by parser `P`. May be a standard or `Partial` parser depending on
/// usage.
pub struct SeparatedSequence<S, P, ApplyBehavior = ApplySeperatorAlways> {
    seq: S,
    sep: P,
    _marker: PhantomData<ApplyBehavior>,
}
impl<S, P> SeparatedSequence<S, P, ApplySeperatorAlways> {
    pub fn seperate_if_not_empty(self) -> SeparatedSequence<S, P, ApplySeperatorIfNotEmpty> {
        let Self { seq, sep, .. } = self;
        SeparatedSequence {
            seq,
            sep,
            _marker: PhantomData,
        }
    }
}
impl<S, P> SeparatedSequence<S, P, ApplySeperatorIfNotEmpty> {
    pub fn always_seperate(self) -> SeparatedSequence<S, P, ApplySeperatorAlways> {
        let Self { seq, sep, .. } = self;
        SeparatedSequence {
            seq,
            sep,
            _marker: PhantomData,
        }
    }
}

/// Sentinel. Configures SeparatedSequence to apply the seperator parser in between
/// each child parser in the sequence, even if the child parser returns an empty string.
///
/// This would be desirable in a format like CSV, where there is always a comma after
/// each column, even if it's empty.
pub struct ApplySeperatorAlways;

/// Sentinel. Configures SeparatedSequence to not apply the seperator parser after
/// child parsers returning an empty string.
///
/// This would be useful in cases like programming languages, where all available
/// whitespace is consumed and so there is no way to delimit a missing optional
/// keyword.
pub struct ApplySeperatorIfNotEmpty;

macro_rules! sequence_impl (
    ($first: literal $($mid: literal)* . $last: literal) => {
        paste::paste! {
            impl<
                Input,
                Error,
                Failure,
                [<Output $first>],
                [<P $first>]: Parser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>],
                    [<P $mid>]: Parser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: Parser<Input, [<Output $last>], Error, Failure>,
            >
            Sequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for ([<P $first>],  $([<P $mid>], )* [<P $last>])
            {
                fn and(self) -> impl Parser<Input, ([<Output $first>], $([<Output $mid>],)* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let ([<output_ $first>], remaining) = self.$first.parse(input)?;
                        $(
                            let ([<output_ $mid>], remaining) = self.$mid.parse(&remaining)?;
                        )*
                        let ([<output_ $last>], remaining) = self.$last.parse(&remaining)?;

                        Ok((
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

            impl<
                Input,
                Error,
                Failure,
                Sep: Parser<Input, (), Error, Failure>,
                [<Output $first>],
                [<P $first>]: Parser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>],
                    [<P $mid>]: Parser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: Parser<Input, [<Output $last>], Error, Failure>,
            >
            Sequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for SeparatedSequence<([<P $first>],  $([<P $mid>], )* [<P $last>]), Sep, ApplySeperatorAlways>
            {
                fn and(self) -> impl Parser<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let ([<output_ $first>], remaining) = self.seq.$first.parse(input)?;
                        let (_, remaining) = self.sep.parse(&remaining)?;
                        $(
                            let ([<output_ $mid>], remaining) = self.seq.$mid.parse(&remaining)?;
                            let (_, remaining) = self.sep.parse(&remaining)?;
                        )*
                        let ([<output_ $last>], remaining) = self.seq.$last.parse(&remaining)?;

                        Ok((
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

            impl<
                Input,
                Error,
                Failure,
                Sep: Parser<Input, (), Error, Failure>,
                [<Output $first>]: Fusable,
                [<P $first>]: Parser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>]: Fusable,
                    [<P $mid>]: Parser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: Parser<Input, [<Output $last>], Error, Failure>,
            >
            Sequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for SeparatedSequence<([<P $first>],  $([<P $mid>], )* [<P $last>]), Sep, ApplySeperatorIfNotEmpty>
            {
                fn and(self) -> impl Parser<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let ([<output_ $first>], mut remaining) = self.seq.$first.parse(input)?;
                        let last_len = [<output_ $first>].len();
                        $(
                            if last_len > 0 {
                                let (_, r) = self.sep.parse(&remaining)?;
                                remaining = r;
                            }
                            let ([<output_ $mid>], r) = self.seq.$mid.parse(&remaining)?;
                            remaining = r;
                            let last_len = [<output_ $mid>].len();
                        )*
                        if last_len > 0 {
                            let (_, r) = self.sep.parse(&remaining)?;
                            remaining = r;
                        }
                        let ([<output_ $last>], remaining) = self.seq.$last.parse(&remaining)?;

                        Ok((
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

            impl<
                Input,
                Error,
                Failure: From<Missing>,
                [<Output $first>],
                [<P $first>]: PartialParser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>],
                    [<P $mid>]: PartialParser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: PartialParser<Input, [<Output $last>], Error, Failure>,
            >
            PartialSequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for ([<P $first>],  $([<P $mid>], )* [<P $last>])
            {
                fn and(self) -> impl PartialParser<Input, ([<Output $first>], $([<Output $mid>],)* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let PartialOk::Complete([<output_ $first>], remaining) = self.$first.partial_parse(input).no_partial()? else {
                            unreachable!()
                        };
                        $(
                            let PartialOk::Complete([<output_ $mid>], remaining) = self.$mid.partial_parse(&remaining).no_partial()? else {
                                unreachable!()
                            };
                        )*
                        let PartialOk::Complete([<output_ $last>], remaining) = self.$last.partial_parse(&remaining).no_partial()? else {
                            unreachable!()
                        };

                        Ok(PartialOk::Complete(
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

            impl<
                Input,
                Error,
                Failure: From<Missing>,
                Sep: PartialParser<Input, (), Error, Failure>,
                [<Output $first>],
                [<P $first>]: PartialParser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>],
                    [<P $mid>]: PartialParser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: PartialParser<Input, [<Output $last>], Error, Failure>,
            >
            PartialSequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for SeparatedSequence<([<P $first>],  $([<P $mid>], )* [<P $last>]), Sep, ApplySeperatorAlways>
            {
                fn and(self) -> impl PartialParser<Input, ([<Output $first>], $([<Output $mid>],)* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let PartialOk::Complete([<output_ $first>], remaining) = self.seq.$first.partial_parse(input).no_partial()? else {
                            unreachable!()
                        };
                        let PartialOk::Complete(_, remaining) = self.sep.partial_parse(input).no_partial()? else {
                            unreachable!()
                        };
                        $(
                            let PartialOk::Complete([<output_ $mid>], remaining) = self.seq.$mid.partial_parse(&remaining).no_partial()? else {
                                unreachable!()
                            };
                            let PartialOk::Complete(_, remaining) = self.sep.partial_parse(input).no_partial()? else {
                                unreachable!()
                            };
                        )*
                        let PartialOk::Complete([<output_ $last>], remaining) = self.seq.$last.partial_parse(&remaining).no_partial()? else {
                            unreachable!()
                        };

                        Ok(PartialOk::Complete(
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

            impl<
                Input,
                Error,
                Failure: From<Missing>,
                Sep: PartialParser<Input, (), Error, Failure>,
                [<Output $first>]: Fusable,
                [<P $first>]: PartialParser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>]: Fusable,
                    [<P $mid>]: PartialParser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: PartialParser<Input, [<Output $last>], Error, Failure>,
            >
            PartialSequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for SeparatedSequence<([<P $first>],  $([<P $mid>], )* [<P $last>]), Sep, ApplySeperatorIfNotEmpty>
            {
                fn and(self) -> impl PartialParser<Input, ([<Output $first>], $([<Output $mid>],)* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let PartialOk::Complete([<output_ $first>], remaining) = self.seq.$first.partial_parse(input).no_partial()? else {
                            unreachable!()
                        };
                        let last_len = [<output_ $first>].len();
                        if last_len > 0 {
                            let PartialOk::Complete(_, remaining) = self.sep.partial_parse(input).no_partial()? else {
                                unreachable!()
                            };
                        }
                        $(
                            let PartialOk::Complete([<output_ $mid>], remaining) = self.seq.$mid.partial_parse(&remaining).no_partial()? else {
                                unreachable!()
                            };
                            let last_len = [<output_ $mid>].len();
                            if last_len > 0 {
                                let PartialOk::Complete(_, remaining) = self.sep.partial_parse(input).no_partial()? else {
                                    unreachable!()
                                };
                            }
                        )*
                        let PartialOk::Complete([<output_ $last>], remaining) = self.seq.$last.partial_parse(&remaining).no_partial()? else {
                            unreachable!()
                        };

                        Ok(PartialOk::Complete(
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            ),
                            remaining
                        ))
                    }
                }
            }

        }
    }
);

implement_for_tuples!(sequence_impl);

#[cfg(test)]
mod test {
    use crate::{
        input::Input,
        parse::{ParserError, ParserResult},
        primitives::tag,
        util::as_partial::AsPartialParser,
    };

    use super::*;

    #[test]
    fn simple() {
        let parser = ("foo", "bar").and();
        assert_eq!(parser.parse(&"foobar"), Ok((("foo", "bar"), "")));
        assert_eq!(parser.parse(&"foobar123"), Ok((("foo", "bar"), "123")));
        assert_eq!(parser.parse(&"foo"), Err(ParserError::Error(NotFound)));
        assert_eq!(parser.parse(&"bar"), Err(ParserError::Error(NotFound)));
    }

    #[test]
    fn separated_always() {
        let parser = ("foo", "bar").with_sep("x").and();
        assert_eq!(parser.parse(&"foobar"), Err(ParserError::Error(NotFound)));
        assert_eq!(parser.parse(&"fooxbar"), Ok((("foo", "bar"), "")));
        assert_eq!(parser.parse(&"fooxbarx"), Ok((("foo", "bar"), "x")));
        assert_eq!(parser.parse(&"fooxbar123"), Ok((("foo", "bar"), "123")));

        assert_eq!(parser.parse(&"foo"), Err(ParserError::Error(NotFound)));
        assert_eq!(parser.parse(&"bar"), Err(ParserError::Error(NotFound)));
    }

    #[test]
    fn separated_if_not_empty() {
        let parser = ("foo", "bar".opt(), "baz")
            .with_sep("x")
            .seperate_if_not_empty()
            .and();
        assert_eq!(
            parser.parse(&"fooxbarxbaz"),
            Ok((("foo", Some("bar"), "baz"), ""))
        );
        assert_eq!(parser.parse(&"fooxbaz"), Ok((("foo", None, "baz"), "")));
    }

    #[test]
    fn whitespace() {
        let parser = ("foo", "bar").whitespace().and();
        assert_eq!(parser.parse(&"foobar"), Err(ParserError::Error(NotFound)));
        for s in ["foo bar", "foo    bar", "foo\nbar", "foo\tbar"] {
            assert_eq!(parser.parse(&s), Ok((("foo", "bar"), "")));
        }
        assert_eq!(parser.parse(&"foo bar\t"), Ok((("foo", "bar"), "\t")));

        assert_eq!(parser.parse(&"foo"), Err(ParserError::Error(NotFound)));
        assert_eq!(parser.parse(&"bar"), Err(ParserError::Error(NotFound)));
    }

    #[test]
    fn whitespace_with_optionals() {
        let parser = ("foo", "bar".opt(), "baz").whitespace().and();
        assert_eq!(
            parser.parse(&"foobarbaz"),
            Err(ParserError::Error(NotFound))
        );
        for s in [
            "foo bar baz",
            "foo    bar baz",
            "foo\nbar baz",
            "foo\tbar baz",
            "foo bar    baz",
            "foo bar\t baz",
            "foo bar\nbaz",
        ] {
            assert_eq!(parser.parse(&s), Ok((("foo", Some("bar"), "baz"), "")));
        }
        for s in ["foo  baz", "foo     baz", "foo\n baz", "foo\t baz"] {
            assert_eq!(parser.parse(&s), Ok((("foo", None, "baz"), "")));
        }

        assert_eq!(
            parser.parse(&"foo bar baz\t"),
            Ok((("foo", Some("bar"), "baz"), "\t"))
        );

        assert_eq!(parser.parse(&"foo"), Err(ParserError::Error(NotFound)));
        assert_eq!(parser.parse(&"bar"), Err(ParserError::Error(NotFound)));
    }

    #[test]
    fn partial_simple() {
        let parser = (
            "foo".as_partial().with_failure(Missing),
            "bar".as_partial().with_failure(Missing),
        )
            .and();
        assert_eq!(
            parser.partial_parse(&"foobar"),
            Ok(PartialOk::Complete(("foo", "bar"), ""))
        );
        assert_eq!(
            parser.partial_parse(&"foobar123"),
            Ok(PartialOk::Complete(("foo", "bar"), "123"))
        );
        assert_eq!(
            parser.partial_parse(&"foo"),
            Err(PartialError::Error(NotFound))
        );
        assert_eq!(
            parser.partial_parse(&"bar"),
            Err(PartialError::Error(NotFound))
        );
    }

    // TODO
    #[test]
    fn partial_separated() {}

    #[test]
    fn partial_whitespace() {}
}
