use crate::util::tuples::implement_for_tuples;

use super::{Never, NotFound, Parser};

/// A tuple of parsers. Returns the first to succeed.
pub trait Choice<Input, Output, Error=NotFound, Failure=Never> {
    fn or(self) -> impl Parser<Input, Output, Error, Failure>;
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
                fn or(self) -> impl Parser<Input, Output, Error, Failure> {
                    move |input: &Input| {
                        $(
                            match self.$idx.parse(input) {
                                Ok(x) => return Ok(x),
                                Err(crate::parse::ParserError::Error(_)) => (),
                                Err(crate::parse::ParserError::Failure(e)) => return Err(crate::parse::ParserError::Failure(e)),
                                Err(crate::parse::ParserError::Incomplete(e)) => return Err(crate::parse::ParserError::Incomplete(e)),
                            }
                        )*

                        self.$last.parse(input)
                    }
                }
            }
        }
    }
);

implement_for_tuples!(choice_impl);
