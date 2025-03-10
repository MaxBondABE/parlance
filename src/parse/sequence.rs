use crate::{primitives::whitespace::whitespace, util::tuples::implement_for_tuples};

use super::{Fusable, Never, NotFound, Parser};

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
                        let (remaining, [<output_ $first>]) = self.$first.parse(input)?;
                        $(
                            let (remaining, [<output_ $mid>]) = self.$mid.parse(&remaining)?;
                        )*
                        let (remaining, [<output_ $last>]) = self.$last.parse(&remaining)?;

                        Ok((
                            remaining,
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            )
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
                        let (remaining, [<output_ $first>]) = self.seq.$first.parse(input)?;
                        $(
                            let (remaining, _) = self.sep.parse(&remaining)?;
                            let (remaining, [<output_ $mid>]) = self.seq.$mid.parse(&remaining)?;
                        )*
                        let (remaining, _) = self.sep.parse(&remaining)?;
                        let (remaining, [<output_ $last>]) = self.seq.$last.parse(&remaining)?;

                        Ok((
                            remaining,
                            (
                                [<output_ $first>],
                                $([<output_ $mid>], )*
                                [<output_ $last>],
                            )
                        ))
                    }
                }
                fn with_sep<O, P>(self, sep: P) -> SeparatedSequence<Self, P> {
                    panic!("This sequence has already been assigned a separator")
                }
            }
        }
    }
);

implement_for_tuples!(sequence_impl);
