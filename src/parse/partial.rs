use std::fmt;

use super::{
    Fusable, Incomplete, Never, NotFound, Parser, ParserError, PartialChoice, PartialCompose,
    PartialSequence,
};

pub trait PartialParser<Input, Output, Error = NotFound, Failure = Never> {
    fn partial_parse(&self, input: &Input) -> PartialResult<Input, Output, Error, Failure>;
    fn complete(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(PartialOk::Complete(o, r)) | Ok(PartialOk::Partial(o, r)) => Ok((o, r)),
            Err(PartialError::Incomplete(e)) => Err(ParserError::Failure(e)),
            Err(e) => Err(e.try_into().unwrap()),
        }
    }
    fn map<O, Func: Fn(Output) -> O>(self, f: Func) -> impl PartialParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(PartialOk::Complete(o, remaining)) => Ok(PartialOk::Complete(f(o), remaining)),
            Ok(PartialOk::Partial(o, remaining)) => Ok(PartialOk::Partial(f(o), remaining)),
            Err(PartialError::Incomplete(e)) => Err(PartialError::Incomplete(e)),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e)),
            Err(PartialError::Failure(e)) => Err(PartialError::Failure(e)),
        }
    }
    fn map_err<E, F, Func: Fn(PartialError<Error, Failure>) -> PartialError<E, F>>(
        self,
        f: Func,
    ) -> impl PartialParser<Input, Output, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(x) => Ok(x),
            Err(e) => Err(f(e)),
        }
    }
    fn map_errors<E, F: Fn(Error) -> E>(self, f: F) -> impl PartialParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Error(e) => PartialError::Error(f(e)),

                PartialError::Incomplete(e) => PartialError::Incomplete(e),
                PartialError::Failure(e) => PartialError::Failure(e),
            })
        }
    }
    fn map_failures<F, Func: Fn(Failure) -> F>(
        self,
        f: Func,
    ) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Incomplete(e) => PartialError::Incomplete(f(e)),
                PartialError::Failure(e) => PartialError::Failure(f(e)),

                PartialError::Error(e) => PartialError::Error(e),
            })
        }
    }
    /// Utility to normalize a parser to an equivalent signature.
    fn to<O: From<Output>, E: From<Error>, F: From<Failure>>(
        self,
    ) -> impl PartialParser<Input, O, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(PartialOk::Complete(o, remaining)) => Ok(PartialOk::Complete(o.into(), remaining)),
            Ok(PartialOk::Partial(o, remaining)) => Ok(PartialOk::Partial(o.into(), remaining)),
            Err(PartialError::Incomplete(e)) => Err(PartialError::Incomplete(e.into())),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e.into())),
            Err(PartialError::Failure(e)) => Err(PartialError::Failure(e.into())),
        }
    }
    fn to_output<O: From<Output>>(self) -> impl PartialParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(From::from)
    }
    fn to_error<E: From<Error>>(self) -> impl PartialParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(From::from)
    }
    fn to_failure<F: From<Failure>>(self) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(From::from)
    }
    fn with_error<E: Clone>(self, err: E) -> impl PartialParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| err.clone())
    }
    fn with_error_as<E, Func: Fn() -> E>(
        self,
        f: Func,
    ) -> impl PartialParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| f())
    }
    fn with_failure<F: Clone>(self, err: F) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| err.clone())
    }
    fn with_failure_as<F, Func: Fn() -> F>(
        self,
        f: Func,
    ) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| f())
    }
    fn with_output<O: Clone>(self, output: O) -> impl PartialParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(move |_| output.clone())
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail(self) -> impl PartialParser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Failure: From<Error>,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Error(e) => PartialError::Failure(e.into()),

                PartialError::Incomplete(e) => PartialError::Incomplete(e),
                PartialError::Failure(e) => PartialError::Failure(e),
            })
        }
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail_with<F: Clone + From<Failure>>(
        self,
        err: F,
    ) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Error(e) => PartialError::Failure(err.clone()),
                PartialError::Incomplete(e) => PartialError::Incomplete(e.into()),
                PartialError::Failure(e) => PartialError::Failure(e.into()),
            })
        }
    }
    /// Upgrade recoverable errors to the given permanent error.
    fn or_fail_as<F: From<Failure>, Func: Fn() -> F>(
        self,
        f: Func,
    ) -> impl PartialParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Error(e) => PartialError::Failure(f()),
                PartialError::Failure(e) => PartialError::Failure(e.into()),
                PartialError::Incomplete(e) => PartialError::Incomplete(e.into()),
            })
        }
    }
    /// Downgrade permanent failures into recoverable errors.
    /// NB: Failures inside of an `Incomplete` variant are untouched.
    fn no_fail(self) -> impl PartialParser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Error: From<Failure>,
    {
        move |input: &Input| {
            self.partial_parse(input).map_err(|e| match e {
                PartialError::Incomplete(e) => PartialError::Error(e.into()),
                PartialError::Failure(e) => PartialError::Error(e.into()),
                PartialError::Error(e) => PartialError::Error(e),
            })
        }
    }
    /// Returns None on a recoverable error.
    fn opt(self) -> impl PartialParser<Input, Option<Output>, Error, Failure>
    where
        Self: Sized,
        Input: Clone,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(PartialOk::Complete(o, remaining)) => Ok(PartialOk::Complete(Some(o), remaining)),
            Ok(PartialOk::Partial(o, remaining)) => Ok(PartialOk::Partial(Some(o), remaining)),
            Err(PartialError::Error(_)) => Ok(PartialOk::Complete(None, input.clone())),
            Err(PartialError::Incomplete(e)) => Err(PartialError::Incomplete(e)),
            Err(PartialError::Failure(e)) => Err(PartialError::Failure(e)),
        }
    }
    fn and<OtherOutput, Other: PartialParser<Input, OtherOutput, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, (Output, OtherOutput), Error, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        (self, other).and()
    }
    fn or<Other: PartialParser<Input, Output, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, Output, NotFound, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        (self, other).or()
    }
    fn then<O, Other: PartialParser<Output, O, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, O, Error, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        PartialCompose::map((self, other))
    }
}

pub type PartialResult<Input, Output, Error = NotFound, Failure = Never> =
    Result<PartialOk<Input, Output>, PartialError<Error, Failure>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PartialOk<Input, Output> {
    Complete(Output, Input),
    Partial(Output, Input),
}
impl<I, O> PartialOk<I, O> {
    pub fn is_partial(&self) -> bool {
        match self {
            PartialOk::Partial(_, _) => true,
            _ => false,
        }
    }
    pub fn peek(&self) -> (&O, &I) {
        match self {
            PartialOk::Complete(o, r) => (&o, &r),
            PartialOk::Partial(o, r) => (&o, &r),
        }
    }
}
impl<I, O: Fusable> Fusable for PartialOk<I, O> {
    fn len(&self) -> usize {
        self.peek().0.len()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum PartialError<Error, Failure> {
    Incomplete(Failure),
    Error(Error),
    Failure(Failure),
}

impl<E, F> From<ParserError<E, F>> for PartialError<E, F> {
    fn from(value: ParserError<E, F>) -> Self {
        match value {
            ParserError::Error(e) => PartialError::Error(e),
            ParserError::Failure(e) => PartialError::Failure(e),
        }
    }
}
impl<E, F> TryFrom<PartialError<E, F>> for ParserError<E, F> {
    type Error = ErrorWasIncomplete;

    fn try_from(value: PartialError<E, F>) -> Result<Self, ErrorWasIncomplete> {
        match value {
            PartialError::Incomplete(_) => Err(ErrorWasIncomplete),
            PartialError::Error(e) => Ok(ParserError::Error(e)),
            PartialError::Failure(e) => Ok(ParserError::Failure(e)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ErrorWasIncomplete;
impl fmt::Debug for ErrorWasIncomplete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cannot convert Incomplete to a non-streaming error.")
    }
}
impl fmt::Display for ErrorWasIncomplete {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

impl<
        Input,
        Output,
        Error,
        Failure,
        T: Fn(&Input) -> PartialResult<Input, Output, Error, Failure>,
    > PartialParser<Input, Output, Error, Failure> for T
{
    fn partial_parse(&self, input: &Input) -> PartialResult<Input, Output, Error, Failure> {
        self(input)
    }
}

pub trait AsPartialParser<Input, Output, Error, Failure> {
    fn as_partial(self) -> impl PartialParser<Input, Output, Error, Failure>;
}
impl<I, O, E, F, T: Parser<I, O, E, F>> AsPartialParser<I, O, E, F> for T {
    fn as_partial(self) -> impl PartialParser<I, O, E, F> {
        move |input: &I| match self.parse(input) {
            Ok((o, r)) => Ok(PartialOk::Complete(o, r)),
            Err(e) => Err(e.into()),
        }
    }
}

pub trait AsPartialResult<Input, Output, Error, Failure> {
    fn as_partial(self) -> PartialResult<Input, Output, Error, Failure>;
}
impl<I, O, E, F> AsPartialResult<I, O, E, F> for Result<PartialOk<I, O>, ParserError<E, F>> {
    fn as_partial(self) -> PartialResult<I, O, E, F> {
        self.map_err(Into::into)
    }
}
