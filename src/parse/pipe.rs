use super::{Missing, Never, NotFound, Parser, PartialOk, PartialParser};
use crate::util::conditional_transforms::NoPartial;

/// Enables chaining a tuple of parsers into a single parser that applies each parser in sequence,
/// passing each output as the input to the next.
pub trait Pipeline<T, Input: crate::input::Input, Output, Error = NotFound, Failure = Never> {
    fn pipe(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized;
}

/// Enables chaining a tuple of parsers into a single parser that applies each parser in sequence,
/// passing each output as the input to the next.
pub trait PartialPipeline<T, Input: crate::input::Input, Output, Error = NotFound, Failure = Never>
{
    fn pipe(self) -> impl PartialParser<Input, Output, Error, Failure>
    where
        Self: Sized;
}

macro_rules! pipeline_impl (
    ($first: literal $(($prev: literal $idx: literal))* . ($last_prev: literal $last: literal)) => {
        paste::paste! {
            impl<
                Input: crate::input::Input,
                Error,
                Failure,
                Output,
                [<Output $first>]: crate::input::Input<Location = Input::Location>,
                [<P $first>]: Parser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $idx>]: crate::input::Input<Location = Input::Location>,
                    [<P $idx>]: Parser<[<Output $prev>], [<Output $idx>], Error, Failure>,
                )*
                [<P $last>]: Parser<[<Output $last_prev>], Output, Error, Failure>,
            > Pipeline<([<Output $first>], $([<Output $idx>], )* ), Input, Output, Error, Failure> for ([<P $first>], $([<P $idx>], )* [<P $last>]) {
                fn pipe(self) -> impl Parser<Input, Output, Error, Failure> {
                    move |input: &Input| {
                        let (output, remaining) = self.$first.parse(input)?;
                        $(
                            let (output, _) = self.$idx.parse(&output)?;
                        )*
                        let (output, _) = self.$last.parse(&output)?;

                        Ok((output, remaining))
                    }

                }
            }

            impl<
                Input: crate::input::Input,
                Error,
                Failure: From<Missing>,
                Output,
                [<Output $first>]: crate::input::Input<Location = Input::Location>,
                [<P $first>]: PartialParser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $idx>]: crate::input::Input<Location = Input::Location>,
                    [<P $idx>]: PartialParser<[<Output $prev>], [<Output $idx>], Error, Failure>,
                )*
                [<P $last>]: PartialParser<[<Output $last_prev>], Output, Error, Failure>,
            > PartialPipeline<([<Output $first>], $([<Output $idx>], )* ), Input, Output, Error, Failure> for ([<P $first>], $([<P $idx>], )* [<P $last>]) {
                fn pipe(self) -> impl PartialParser<Input, Output, Error, Failure> {
                    move |input: &Input| {
                        let PartialOk::Complete(output, remaining) = self.$first.partial_parse(input).no_partial()? else {
                            unreachable!()
                        };
                        $(
                            let PartialOk::Complete(output, _) = self.$idx.partial_parse(&output).no_partial()? else {
                                unreachable!()
                            };
                        )*
                        let PartialOk::Complete(output, _) = self.$last.partial_parse(&output).no_partial()? else {
                            unreachable!()
                        };

                        Ok(PartialOk::Complete(output, remaining))
                    }

                }
            }
        }
    }
);

pipeline_impl!(0 . (0 1));
pipeline_impl!(0 (0 1) . (1 2));
pipeline_impl!(0 (0 1) (1 2) . (2 3));
pipeline_impl!(0 (0 1) (1 2) (2 3) . (3 4));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) . (4 5));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) . (5 6));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) . (6 7));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) . (7 8));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) . (8 9));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) . (9 10));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) . (10 11));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) . (11 12));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) . (12 13));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) (12 13) . (13 14));
pipeline_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) (12 13) (13 14) . (14 15));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple() {}
}
