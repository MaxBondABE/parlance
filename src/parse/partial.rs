use std::fmt;

use super::{
    Fusable, Missing, Never, NotFound, Parser, ParserError, PartialChoice, PartialPipeline,
    PartialSequence,
};

pub trait PartialParser<Input: crate::input::Input, Output, Error = NotFound, Failure = Never> {
    fn partial_parse(&self, input: &Input) -> PartialResult<Input, Output, Error, Failure>;
    fn complete(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.partial_parse(input) {
            Ok(PartialOk::Complete(o, r)) | Ok(PartialOk::Partial(o, r)) => Ok((o, r)),
            Err(PartialError::Incomplete(e, l)) => Err(ParserError::Failure(e, l)),
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
            Err(PartialError::Incomplete(e, l)) => Err(PartialError::Incomplete(e, l)),
            Err(PartialError::Error(e, l)) => Err(PartialError::Error(e, l)),
            Err(PartialError::Failure(e, l)) => Err(PartialError::Failure(e, l)),
        }
    }
    fn map_err<
        E,
        F,
        Func: Fn(PartialError<Error, Failure, Input::Location>) -> PartialError<E, F, Input::Location>,
    >(
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
                PartialError::Error(e, l) => PartialError::Error(f(e), l),

                PartialError::Incomplete(e, l) => PartialError::Incomplete(e, l),
                PartialError::Failure(e, l) => PartialError::Failure(e, l),
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
                PartialError::Incomplete(e, l) => PartialError::Incomplete(f(e), l),
                PartialError::Failure(e, l) => PartialError::Failure(f(e), l),

                PartialError::Error(e, l) => PartialError::Error(e, l),
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
            Err(PartialError::Incomplete(e, l)) => Err(PartialError::Incomplete(e.into(), l)),
            Err(PartialError::Error(e, l)) => Err(PartialError::Error(e.into(), l)),
            Err(PartialError::Failure(e, l)) => Err(PartialError::Failure(e.into(), l)),
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
                PartialError::Error(e, l) => PartialError::Failure(e.into(), l),

                PartialError::Incomplete(e, l) => PartialError::Incomplete(e, l),
                PartialError::Failure(e, l) => PartialError::Failure(e, l),
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
                PartialError::Error(e, l) => PartialError::Failure(err.clone(), l),
                PartialError::Incomplete(e, l) => PartialError::Incomplete(e.into(), l),
                PartialError::Failure(e, l) => PartialError::Failure(e.into(), l),
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
                PartialError::Error(e, l) => PartialError::Failure(f(), l),
                PartialError::Failure(e, l) => PartialError::Failure(e.into(), l),
                PartialError::Incomplete(e, l) => PartialError::Incomplete(e.into(), l),
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
                PartialError::Incomplete(e, l) => PartialError::Error(e.into(), l),
                PartialError::Failure(e, l) => PartialError::Error(e.into(), l),
                PartialError::Error(e, l) => PartialError::Error(e, l),
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
            Err(PartialError::Error(..)) => Ok(PartialOk::Complete(None, input.clone())),
            Err(PartialError::Incomplete(e, l)) => Err(PartialError::Incomplete(e, l)),
            Err(PartialError::Failure(e, l)) => Err(PartialError::Failure(e, l)),
        }
    }
    fn and<OtherOutput, Other: PartialParser<Input, OtherOutput, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, (Output, OtherOutput), Error, Failure>
    where
        Self: Sized,
        Failure: From<Missing>,
    {
        (self, other).and()
    }
    fn or<Other: PartialParser<Input, Output, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, Output, NotFound, Failure>
    where
        Self: Sized,
        Failure: From<Missing>,
    {
        (self, other).or()
    }
    fn then<O, Other: PartialParser<Output, O, Error, Failure>>(
        self,
        other: Other,
    ) -> impl PartialParser<Input, O, Error, Failure>
    where
        Self: Sized,
        Failure: From<Missing>,
        Output: crate::input::Input<Location = Input::Location>,
    {
        PartialPipeline::pipe((self, other))
    }

    /// Identity function. A helper for certain macros.
    fn as_partial(self) -> impl PartialParser<Input, Output, Error, Failure>
    where
        Self: Sized,
    {
        self
    }
}

pub type PartialResult<Input, Output, Error = NotFound, Failure = Never> = Result<
    PartialOk<Input, Output>,
    PartialError<Error, Failure, <Input as crate::input::Input>::Location>,
>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum PartialError<Error, Failure, Location> {
    Incomplete(Failure, Location),
    Error(Error, Location),
    Failure(Failure, Location),
}

impl<E, F, L> From<ParserError<E, F, L>> for PartialError<E, F, L> {
    fn from(value: ParserError<E, F, L>) -> Self {
        match value {
            ParserError::Error(e, l) => PartialError::Error(e, l),
            ParserError::Failure(e, l) => PartialError::Failure(e, l),
        }
    }
}
impl<E, F, L> TryFrom<PartialError<E, F, L>> for ParserError<E, F, L> {
    type Error = ErrorWasIncomplete;

    fn try_from(value: PartialError<E, F, L>) -> Result<Self, ErrorWasIncomplete> {
        match value {
            PartialError::Incomplete(..) => Err(ErrorWasIncomplete),
            PartialError::Error(e, l) => Ok(ParserError::Error(e, l)),
            PartialError::Failure(e, l) => Ok(ParserError::Failure(e, l)),
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

// Implements PartialParser for all functions with the correct signature.
impl<
        Input: crate::input::Input,
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
