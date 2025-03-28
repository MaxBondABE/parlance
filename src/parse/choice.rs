use crate::util::tuples::implement_for_tuples;

use super::{Never, NotFound, Parser, StreamingParser, StreamingError, ParserError};

/// A tuple of parsers. Returns the first to succeed.
pub trait Choice<Input, Output, Error = NotFound, Failure = Never> {
    fn or(self) -> impl Parser<Input, Output, NotFound, Failure>;
}

/// A tuple of streaming parsers. Returns the first to succeed.
pub trait StreamingChoice<Input, Output, Error = NotFound, Failure = Never> {
    fn or(self) -> impl StreamingParser<Input, Output, NotFound, Failure>;
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
                $([<P $idx>]: StreamingParser<Input, Output, Error, Failure>, )*
                [<P $last>]: StreamingParser<Input, Output, Error, Failure>,
                > StreamingChoice<Input, Output, Error, Failure> for ($([<P $idx>], )* [<P $last>])
            {
                fn or(self) -> impl StreamingParser<Input, Output, NotFound, Failure> {
                    move |input: &Input| {
                        $(
                            match self.$idx.parse_stream(input) {
                                Ok(x) => return Ok(x),
                                Err(StreamingError::Error(_)) => (),
                                Err(StreamingError::Incomplete(e)) => return Err(StreamingError::Incomplete(e)),
                                Err(StreamingError::Failure(e)) => return Err(StreamingError::Failure(e)),
                            }
                        )*

                        match self.$last.parse_stream(input) {
                            Ok(x) => Ok(x),
                            Err(StreamingError::Error(_)) => Err(StreamingError::Error(NotFound)),
                            Err(StreamingError::Incomplete(e)) => return Err(StreamingError::Incomplete(e)),
                            Err(StreamingError::Failure(e)) => Err(StreamingError::Failure(e)),
                        }
                    }
                }
            }
        }
    }
);

implement_for_tuples!(choice_impl);
