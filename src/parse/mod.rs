use std::fmt;

use crate::primitives::whitespace::whitespace;

mod choice;
mod compose;
mod err;
mod fuse;
mod sequence;
mod streaming;

pub use choice::{Choice, StreamingChoice};
pub use compose::{Compose, StreamingCompose};
pub use err::{Incomplete, Never, NotFound};
pub use fuse::{Fusable, FuseSequence};
pub use sequence::{SeparatedSequence, Sequence, StreamingSequence};
pub use streaming::{
    ErrorWasIncomplete, IntoStreamingParser, IntoStreamingResult, StreamingError, StreamingOk,
    StreamingParser, StreamingResult,
};

pub trait Parser<Input, Output, Error = NotFound, Failure = Never> {
    fn parse(&self, input: &Input) -> ParserResult<Input, Output, Error, Failure>;
    fn map<O, Func: Fn(Output) -> O>(self, f: Func) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse(input) {
            Ok((o, remaining)) => Ok((f(o), remaining)),
            Err(ParserError::Error(e)) => Err(ParserError::Error(e)),
            Err(ParserError::Failure(e)) => Err(ParserError::Failure(e)),
        }
    }
    fn map_err<E, F, Func: Fn(ParserError<Error, Failure>) -> ParserError<E, F>>(
        self,
        f: Func,
    ) -> impl Parser<Input, Output, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse(input) {
            Ok(x) => Ok(x),
            Err(e) => Err(f(e)),
        }
    }
    fn map_errors<E, F: Fn(Error) -> E>(self, f: F) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Error(f(e)),
                ParserError::Failure(e) => ParserError::Failure(e),
            })
        }
    }
    fn map_failures<F, Func: Fn(Failure) -> F>(
        self,
        f: Func,
    ) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Failure(e) => ParserError::Failure(f(e)),
                ParserError::Error(e) => ParserError::Error(e),
            })
        }
    }
    /// Utility to normalize a parser to an equivalent signature.
    fn to<O: From<Output>, E: From<Error>, F: From<Failure>>(self) -> impl Parser<Input, O, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse(input) {
            Ok((o, remaining)) => Ok((o.into(), remaining)),
            Err(ParserError::Error(e)) => Err(ParserError::Error(e.into())),
            Err(ParserError::Failure(e)) => Err(ParserError::Failure(e.into())),
        }
    }
    fn to_output<O: From<Output>>(self) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(From::from)
    }
    fn to_error<E: From<Error>>(self) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(From::from)
    }
    fn to_failure<F: From<Failure>>(self) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(From::from)
    }
    fn with_error<E: Clone>(self, err: E) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| err.clone())
    }
    fn with_error_as<E, Func: Fn() -> E>(self, f: Func) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| f())
    }
    fn with_failure<F: Clone>(self, err: F) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| err.clone())
    }
    fn with_failure_as<F, Func: Fn() -> F>(self, f: Func) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| f())
    }
    fn with_output<O: Clone>(self, output: O) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(move |_| output.clone())
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Failure: From<Error>,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Failure(e.into()),
                ParserError::Failure(e) => ParserError::Failure(e),
            })
        }
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail_with<F: Clone + From<Failure>>(self, err: F) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Failure(err.clone()),
                ParserError::Failure(e) => ParserError::Failure(e.into()),
            })
        }
    }
    /// Upgrade recoverable errors to the given permanent error.
    fn or_fail_as<F, Func: Fn() -> F>(self, f: Func) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Failure(f()),
                ParserError::Failure(e) => ParserError::Failure(f()),
            })
        }
    }
    /// Downgrade permanent failures into recoverable errors.
    /// NB: Failures inside of an `Incomplete` variant are untouched.
    fn no_fail(self) -> impl Parser<Input, Output, Error, Never>
    where
        Self: Sized,
        Error: From<Failure>,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Failure(e) => ParserError::Error(e.into()),
                ParserError::Error(e) => ParserError::Error(e),
            })
        }
    }
    /// Returns None on a recoverable error.
    fn opt(self) -> impl Parser<Input, Option<Output>, Error, Failure>
    where
        Self: Sized,
        Input: Clone,
    {
        move |input: &Input| match self.parse(input) {
            Ok((x, remaining)) => Ok((Some(x), remaining)),
            Err(ParserError::Error(_)) => Ok((None, input.clone())),
            Err(ParserError::Failure(e)) => Err(ParserError::Failure(e)),
        }
    }
    fn and<OtherOutput, Other: Parser<Input, OtherOutput, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, (Output, OtherOutput), Error, Failure>
    where
        Self: Sized,
    {
        (self, other).and()
    }
    fn or<Other: Parser<Input, Output, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, Output, NotFound, Failure>
    where
        Self: Sized,
    {
        (self, other).or()
    }
    fn then<O, Other: Parser<Output, O, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        Compose::map((self, other))
    }
}

pub type ParserResult<Input, Output, Error = NotFound, Failure = Never> =
    Result<(Output, Input), ParserError<Error, Failure>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserError<Error, Failure> {
    Error(Error),
    Failure(Failure),
}
impl<E: fmt::Debug, F: fmt::Debug> fmt::Debug for ParserError<E, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::Error(e) => {
                f.write_fmt(format_args!("Parsing error: Normal error {:?}", e))
            }
            ParserError::Failure(e) => {
                f.write_fmt(format_args!("Parsing error: Permanent failure {:?}", e))
            }
        }
    }
}

impl<
        Input,
        Output,
        Error,
        Failure,
        T: Fn(&Input) -> ParserResult<Input, Output, Error, Failure>,
    > Parser<Input, Output, Error, Failure> for T
{
    fn parse(&self, input: &Input) -> ParserResult<Input, Output, Error, Failure> {
        self(input)
    }
}
