use std::fmt;

use crate::{combinators::sandwich, primitives::whitespace::whitespace};

mod choice;
mod err;
mod fuse;
mod partial;
mod pipe;
mod sequence;
mod stream;

pub use choice::{Choice, PartialChoice};
pub use err::{Missing, Never, NotFound};
pub use fuse::{Fusable, FuseSequence, PartialFuseSequence};
pub use partial::{ErrorWasIncomplete, PartialError, PartialOk, PartialParser, PartialResult};
pub use pipe::{PartialPipeline, Pipeline};
pub use sequence::{PartialSequence, SeparatedSequence, Sequence};

/// A parsing operation.
///
/// A parsing operation is any function implementing `Fn(&Input) -> ParserResult<Input, Output,
/// Error, Failure>`, for any values of these generics. In practice, the `Input` generic should
/// also implement the `Input` trait, which supplies low level text parsing utilities.
///
/// Parsers can be composed together, allowing you to create large and complex parsers from simpler
/// parts. Parsers can be composed in the following ways:
/// - `Sequence` allows you to
/// - `Choice`
/// - `Pipeline`
/// - `Fuse`
pub trait Parser<Input, Output, Error = NotFound, Failure = Never> {
    /// Attempt to consume the provided input, returning the parsed token
    /// and the remaining input, or a `ParserError`.
    fn parse(&self, input: &Input) -> ParserResult<Input, Output, Error, Failure>;
    /// Map a function over the parser's output.
    fn map<O, Func: Fn(Output) -> O>(self, f: Func) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse(input) {
            Ok((o, remaining)) => Ok((f(o), remaining)),
            Err(e) => Err(e),
        }
    }
    /// Map a function over the parser's error cases.
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
    /// Map a function over `ParserError::Error` cases.
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
    /// Map a function over `ParserError::Failure` cases.
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
    /// Normalize all type parameters (except `Input`) using `From`.
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
    /// Normalize the `Output` type of a parser using `From`.
    fn into_output<O: From<Output>>(self) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(From::from)
    }
    /// Normalize the `Error` type of a parser using `From`.
    fn into_err<E: From<Error>>(self) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(From::from)
    }
    /// Normalize the `Failure` type of a parser using `From`.
    fn into_fail<F: From<Failure>>(self) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(From::from)
    }
    /// Substite any `ParserError::Error` values returned by the parser with the one provided.
    fn with_error<E: Clone>(self, err: E) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| err.clone())
    }
    /// Substite any `ParserError::Error` values returned by the parser with the output of the
    /// function provided.
    fn with_error_as<E, Func: Fn() -> E>(self, f: Func) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| f())
    }
    /// Substite any `ParserError::Failure` values returned by the parser with the one provided.
    fn with_failure<F: Clone>(self, err: F) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| err.clone())
    }
    /// Substite any `ParserError::Failure` values returned by the parser with the output of the
    /// function provided.
    fn with_failure_as<F, Func: Fn() -> F>(self, f: Func) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| f())
    }
    /// Substite the output value returned by the parser with the one provided.
    fn with_output<O: Clone>(self, output: O) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(move |_| output.clone())
    }
    /// Promote recoverable `ParserError::Error` cases to permanent `ParserError::Failure` cases
    /// using `From`.
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
    /// Promote recoverable `ParserError::Error` cases to permanent `ParserError::Failure` cases
    /// by substituting the provided value. `ParserError::Failure` cases are converted using
    /// `From`.
    fn or_fail_with<F: Clone + From<Failure>>(self, err: F) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(_) => ParserError::Failure(err.clone()),
                ParserError::Failure(e) => ParserError::Failure(e.into()),
            })
        }
    }
    /// Promote recoverable `ParserError::Error` cases to permanent `ParserError::Failure` cases
    /// using the provided function. `ParserError::Failure` cases are converted using
    /// `From`.
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
    /// Demote `ParserError::Failure` cases into `ParserError::Error` cases.
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
    /// Demote `ParserError::Failure` cases into `NotFound` errors
    fn or_not_found(self) -> impl Parser<Input, Output, Error, Never>
    where
        Self: Sized,
        Error: From<NotFound>,
    {
        self.or_fail_as(|| NotFound).no_fail()
    }
    /// Promotes `ParserError::Error` cases to a success with a value of `None`.
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
    /// Concatenate two parsers together using `Sequence`.
    fn and<OtherOutput, Other: Parser<Input, OtherOutput, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, (Output, OtherOutput), Error, Failure>
    where
        Self: Sized,
    {
        (self, other).and()
    }
    /// Branch between two parsers using `Choice`.
    fn or<Other: Parser<Input, Output, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, Output, NotFound, Failure>
    where
        Self: Sized,
    {
        (self, other).or()
    }
    /// Apply the parser `other` to the output of this parser using `Pipe`.
    fn then<O, Other: Parser<Output, O, Error, Failure>>(
        self,
        other: Other,
    ) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        Pipeline::pipe((self, other))
    }
}

/// The result of a parsing operation. A successful parsing operation consumes
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

// Implements `Parser` for all functions with the correct signature.
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
