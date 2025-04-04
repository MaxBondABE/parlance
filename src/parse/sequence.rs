use crate::{
    primitives::whitespace::{partial_whitespace, whitespace},
    util::{conditional_transforms::NoPartial, tuples::implement_for_tuples},
};

use super::{
    AsPartialParser, Fusable, Incomplete, Never, NotFound, Parser, PartialError, PartialOk,
    PartialParser,
};

/// A set of parsers which consume the input in series, failing if any
/// one of them returns an error.
pub trait Sequence<Input, Output, Error = NotFound, Failure = Never> {
    fn and(self) -> impl Parser<Input, Output, Error, Failure>;
    fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, P>
    where
        Self: Sized,
    {
        SeparatedSequence { seq: self, sep }
    }
    fn whitespace(self) -> SeparatedSequence<Self, impl Parser<Input, Input>>
    where
        Self: Sized,
        Input: crate::input::Input,
    {
        self.with_sep::<Input, _>(whitespace)
    }
}

/// A set of partial parsers which consume the input in series, failing if any
/// one of them returns an error.
pub trait PartialSequence<Input, Output, Error = NotFound, Failure = Never> {
    fn and(self) -> impl PartialParser<Input, Output, Error, Failure>;
    fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, P>
    where
        Self: Sized,
    {
        SeparatedSequence { seq: self, sep }
    }
    fn whitespace(self) -> SeparatedSequence<Self, impl PartialParser<Input, Input>>
    where
        Self: Sized,
        Input: crate::input::Input,
    {
        self.with_sep::<Input, _>(partial_whitespace)
    }
}

pub struct SeparatedSequence<S, P> {
    seq: S,
    sep: P,
}

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
            for SeparatedSequence<([<P $first>],  $([<P $mid>], )* [<P $last>]), Sep>
            {
                fn and(self) -> impl Parser<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let ([<output_ $first>], remaining) = self.seq.$first.parse(input)?;
                        $(
                            let (_, remaining) = self.sep.parse(&remaining)?;
                            let ([<output_ $mid>], remaining) = self.seq.$mid.parse(&remaining)?;
                        )*
                        let (_, remaining) = self.sep.parse(&remaining)?;
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
                fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, P> {
                    panic!("This sequence has already been assigned a separator")
                }
            }

            impl<
                Input,
                Error,
                Failure: From<Incomplete>,
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
                            let PartialOk::Complete([<output_ $mid>], remaining) = self.$mid.partial_parse(input).no_partial()? else {
                                unreachable!()
                            };
                        )*
                        let PartialOk::Complete([<output_ $last>], remaining) = self.$last.partial_parse(input).no_partial()? else {
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
