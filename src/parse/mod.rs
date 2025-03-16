use std::fmt;

use crate::primitives::whitespace::whitespace;

mod choice;
mod compose;
mod fuse;
mod sequence;

pub use choice::Choice;
pub use compose::Compose;
pub use fuse::{Fusable, FuseSequence};
pub use sequence::{SeparatedSequence, Sequence};

pub trait Parser<Input, Output, Error = NotFound, Failure = Never> {
    fn parse(&self, input: &Input) -> ParserResult<Input, Output, Error, Failure>;
    fn to<O: From<Output>, E: From<Error>, F: From<Failure>>(self) -> impl Parser<Input, O, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse(input) {
            Ok((remaining, o)) => Ok((remaining, o.into())),
            Err(ParserError::Error(e)) => Err(ParserError::Error(e.into())),
            Err(ParserError::Incomplete(e)) => Err(ParserError::Incomplete(e.into())),
            Err(ParserError::Failure(e)) => Err(ParserError::Failure(e.into())),
        }
    }
    fn as_output<O: From<Output>>(self) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(From::from)
    }
    fn as_error<E: From<Error>>(self) -> impl Parser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(From::from)
    }
    fn as_failure<F: From<Failure>>(self) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(From::from)
    }
    fn map<O, Func: Fn(Output) -> O>(self, f: Func) -> impl Parser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| self.parse(input).map(|(r, o)| (r, f(o)))
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
                ParserError::Incomplete(e) => ParserError::Incomplete(e),
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
                ParserError::Incomplete(e) => ParserError::Incomplete(f(e)),
                ParserError::Error(e) => ParserError::Error(e),
            })
        }
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
    /// Transform incompletes into failures.
    fn complete(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Incomplete(e) | ParserError::Failure(e) => ParserError::Failure(e),
                ParserError::Error(e) => ParserError::Error(e),
            })
        }
    }
    /// Upgrade recoverable errors to permanent failures.
    fn fail(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Failure: From<Error>,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Failure(e.into()),
                ParserError::Failure(e) => ParserError::Failure(e),
                ParserError::Incomplete(e) => ParserError::Incomplete(e),
            })
        }
    }
    /// Downgrade permanent failures into recoverable errors.
    fn no_fail(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Error: From<Failure>,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Failure(e) => ParserError::Error(e.into()),
                ParserError::Error(e) => ParserError::Error(e),
                ParserError::Incomplete(e) => ParserError::Incomplete(e),
            })
        }
    }
    /// Upgrade recoverable errors to the given permanent error.
    fn fail_with<F: Clone + From<Failure>>(self, err: F) -> impl Parser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse(input).map_err(|e| match e {
                ParserError::Error(e) => ParserError::Failure(err.clone()),
                ParserError::Failure(e) => ParserError::Failure(e.into()),
                ParserError::Incomplete(e) => ParserError::Incomplete(e.into()),
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
            Ok((remaining, x)) => Ok((remaining, Some(x))),
            Err(ParserError::Error(_)) => Ok((input.clone(), None)),
            Err(e) => Err(e),
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
    ) -> impl Parser<Input, Output, Error, Failure>
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

pub type ParserResult<I, O, E = NotFound, F = Never> = Result<(I, O), ParserError<E, F>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ParserError<E, F> {
    Incomplete(F),
    Error(E),
    Failure(F),
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
            ParserError::Incomplete(e) => {
                f.write_fmt(format_args!("Parsing error: Input is incomplete {:?}", e))
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

/// A typical recoverable error
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotFound;

/// An error or failure which is returned by any code branch. A `From`
/// implementation should consist only of `unreachable!()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Never(Neverever);

/// A type which is not exported and prevents foreign crates from ever creating
/// a Never value.
/// NB: This does not prevent a programmer working within this crate from creating
/// a `Never`. This must be enforced via code review.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Neverever;
