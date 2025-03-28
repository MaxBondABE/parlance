use crate::{
    primitives::whitespace::{whitespace, whitespace_stream},
    util::{conditional_transforms::NoPartial, tuples::implement_for_tuples},
};

use super::{
    Fusable, Incomplete, IntoStreamingParser, Never, NotFound, Parser, StreamingError, StreamingOk,
    StreamingParser,
};

/// A tuple of parsers, applied serially.
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

/// A tuple of streaming parsers, applied serially.
pub trait StreamingSequence<Input, Output, Error = NotFound, Failure = Never> {
    fn and(self) -> impl StreamingParser<Input, Output, Error, Failure>;
    fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, P>
    where
        Self: Sized,
    {
        SeparatedSequence { seq: self, sep }
    }
    fn whitespace(self) -> SeparatedSequence<Self, impl StreamingParser<Input, Input>>
    where
        Self: Sized,
        Input: crate::input::Input,
    {
        self.with_sep::<Input, _>(whitespace_stream)
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
                [<P $first>]: StreamingParser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $mid>],
                    [<P $mid>]: StreamingParser<Input, [<Output $mid>], Error, Failure>,
                )*
                [<Output $last>],
                [<P $last>]: StreamingParser<Input, [<Output $last>], Error, Failure>,
            >
            StreamingSequence<Input, ([<Output $first>], $([<Output $mid>], )* [<Output $last>]), Error, Failure>
            for ([<P $first>],  $([<P $mid>], )* [<P $last>])
            {
                fn and(self) -> impl StreamingParser<Input, ([<Output $first>], $([<Output $mid>],)* [<Output $last>]), Error, Failure> {
                    move |input: &Input| {
                        let StreamingOk::Complete([<output_ $first>], remaining) = self.$first.parse_stream(input).no_partial()? else {
                            unreachable!()
                        };
                        $(
                            let StreamingOk::Complete([<output_ $mid>], remaining) = self.$mid.parse_stream(input).no_partial()? else {
                                unreachable!()
                            };
                        )*
                        let StreamingOk::Complete([<output_ $last>], remaining) = self.$last.parse_stream(input).no_partial()? else {
                            unreachable!()
                        };

                        Ok(StreamingOk::Complete(
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
