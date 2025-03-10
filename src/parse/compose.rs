use super::{Never, NotFound, Parser};

pub trait Compose<T, Input, Output, Error = NotFound, Failure = Never> {
    fn map(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized;
}

macro_rules! compose_impl (
    ($first: literal $(($prev: literal $idx: literal))* . ($last_prev: literal $last: literal)) => {
        paste::paste! {
            impl<
                Input,
                Error,
                Failure,
                Output,
                [<Output $first>],
                [<P $first>]: Parser<Input, [<Output $first>], Error, Failure>,
                $(
                    [<Output $idx>],
                    [<P $idx>]: Parser<[<Output $prev>], [<Output $idx>], Error, Failure>,
                )*
                [<P $last>]: Parser<[<Output $last_prev>], Output, Error, Failure>,
            > Compose<([<Output $first>], $([<Output $idx>], )* ), Input, Output, Error, Failure> for ([<P $first>], $([<P $idx>], )* [<P $last>]) {
                fn map(self) -> impl Parser<Input, Output, Error, Failure> {
                    move |input: &Input| {
                        let (remaining, output) = self.$first.parse(input)?;
                        $(
                            let (_, output) = self.$idx.parse(&output)?;
                        )*
                        let (_, output) = self.$last.parse(&output)?;

                        Ok((remaining, output))
                    }

                }
            }
        }
    }
);

compose_impl!(0 . (0 1));
compose_impl!(0 (0 1) . (1 2));
compose_impl!(0 (0 1) (1 2) . (2 3));
compose_impl!(0 (0 1) (1 2) (2 3) . (3 4));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) . (4 5));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) . (5 6));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) . (6 7));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) . (7 8));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) . (8 9));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) . (9 10));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) . (10 11));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) . (11 12));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) . (12 13));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) (12 13) . (13 14));
compose_impl!(0 (0 1) (1 2) (2 3) (3 4) (4 5) (5 6) (6 7) (7 8) (8 9) (9 10) (10 11) (11 12) (12 13) (13 14) . (14 15));
