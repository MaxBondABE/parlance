use std::fmt;

use super::{
    Fusable, Incomplete, Never, NotFound, Parser, ParserError, StreamingChoice, StreamingCompose,
    StreamingSequence,
};

pub trait StreamingParser<Input, Output, Error = NotFound, Failure = Never> {
    fn parse_stream(&self, input: &Input) -> StreamingResult<Input, Output, Error, Failure>;
    fn complete(self) -> impl Parser<Input, Output, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse_stream(input) {
            Ok(StreamingOk::Complete(o, r)) | Ok(StreamingOk::Partial(o, r)) => Ok((o, r)),
            Err(StreamingError::Incomplete(e)) => Err(ParserError::Failure(e)),
            Err(e) => Err(e.try_into().unwrap()),
        }
    }
    fn map<O, Func: Fn(Output) -> O>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse_stream(input) {
            Ok(StreamingOk::Complete(o, remaining)) => Ok(StreamingOk::Complete(f(o), remaining)),
            Ok(StreamingOk::Partial(o, remaining)) => Ok(StreamingOk::Partial(f(o), remaining)),
            Err(StreamingError::Incomplete(e)) => Err(StreamingError::Incomplete(e)),
            Err(StreamingError::Error(e)) => Err(StreamingError::Error(e)),
            Err(StreamingError::Failure(e)) => Err(StreamingError::Failure(e)),
        }
    }
    fn map_err<E, F, Func: Fn(StreamingError<Error, Failure>) -> StreamingError<E, F>>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, Output, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse_stream(input) {
            Ok(x) => Ok(x),
            Err(e) => Err(f(e)),
        }
    }
    fn map_errors<E, F: Fn(Error) -> E>(
        self,
        f: F,
    ) -> impl StreamingParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Error(e) => StreamingError::Error(f(e)),

                StreamingError::Incomplete(e) => StreamingError::Incomplete(e),
                StreamingError::Failure(e) => StreamingError::Failure(e),
            })
        }
    }
    fn map_failures<F, Func: Fn(Failure) -> F>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Incomplete(e) => StreamingError::Incomplete(f(e)),
                StreamingError::Failure(e) => StreamingError::Failure(f(e)),

                StreamingError::Error(e) => StreamingError::Error(e),
            })
        }
    }
    /// Utility to normalize a parser to an equivalent signature.
    fn to<O: From<Output>, E: From<Error>, F: From<Failure>>(
        self,
    ) -> impl StreamingParser<Input, O, E, F>
    where
        Self: Sized,
    {
        move |input: &Input| match self.parse_stream(input) {
            Ok(StreamingOk::Complete(o, remaining)) => {
                Ok(StreamingOk::Complete(o.into(), remaining))
            }
            Ok(StreamingOk::Partial(o, remaining)) => Ok(StreamingOk::Partial(o.into(), remaining)),
            Err(StreamingError::Incomplete(e)) => Err(StreamingError::Incomplete(e.into())),
            Err(StreamingError::Error(e)) => Err(StreamingError::Error(e.into())),
            Err(StreamingError::Failure(e)) => Err(StreamingError::Failure(e.into())),
        }
    }
    fn to_output<O: From<Output>>(self) -> impl StreamingParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(From::from)
    }
    fn to_error<E: From<Error>>(self) -> impl StreamingParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(From::from)
    }
    fn to_failure<F: From<Failure>>(self) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(From::from)
    }
    fn with_error<E: Clone>(self, err: E) -> impl StreamingParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| err.clone())
    }
    fn with_error_as<E, Func: Fn() -> E>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, Output, E, Failure>
    where
        Self: Sized,
    {
        self.map_errors(move |_| f())
    }
    fn with_failure<F: Clone>(self, err: F) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| err.clone())
    }
    fn with_failure_as<F, Func: Fn() -> F>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        self.map_failures(move |_| f())
    }
    fn with_output<O: Clone>(self, output: O) -> impl StreamingParser<Input, O, Error, Failure>
    where
        Self: Sized,
    {
        self.map(move |_| output.clone())
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail(self) -> impl StreamingParser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Failure: From<Error>,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Error(e) => StreamingError::Failure(e.into()),

                StreamingError::Incomplete(e) => StreamingError::Incomplete(e),
                StreamingError::Failure(e) => StreamingError::Failure(e),
            })
        }
    }
    /// Upgrade recoverable errors to permanent failures.
    fn or_fail_with<F: Clone + From<Failure>>(
        self,
        err: F,
    ) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Error(e) => StreamingError::Failure(err.clone()),
                StreamingError::Incomplete(e) => StreamingError::Incomplete(e.into()),
                StreamingError::Failure(e) => StreamingError::Failure(e.into()),
            })
        }
    }
    /// Upgrade recoverable errors to the given permanent error.
    fn or_fail_as<F: From<Failure>, Func: Fn() -> F>(
        self,
        f: Func,
    ) -> impl StreamingParser<Input, Output, Error, F>
    where
        Self: Sized,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Error(e) => StreamingError::Failure(f()),
                StreamingError::Failure(e) => StreamingError::Failure(e.into()),
                StreamingError::Incomplete(e) => StreamingError::Incomplete(e.into()),
            })
        }
    }
    /// Downgrade permanent failures into recoverable errors.
    /// NB: Failures inside of an `Incomplete` variant are untouched.
    fn no_fail(self) -> impl StreamingParser<Input, Output, Error, Failure>
    where
        Self: Sized,
        Error: From<Failure>,
    {
        move |input: &Input| {
            self.parse_stream(input).map_err(|e| match e {
                StreamingError::Incomplete(e) => StreamingError::Error(e.into()),
                StreamingError::Failure(e) => StreamingError::Error(e.into()),
                StreamingError::Error(e) => StreamingError::Error(e),
            })
        }
    }
    /// Returns None on a recoverable error.
    fn opt(self) -> impl StreamingParser<Input, Option<Output>, Error, Failure>
    where
        Self: Sized,
        Input: Clone,
    {
        move |input: &Input| match self.parse_stream(input) {
            Ok(StreamingOk::Complete(o, remaining)) => {
                Ok(StreamingOk::Complete(Some(o), remaining))
            }
            Ok(StreamingOk::Partial(o, remaining)) => Ok(StreamingOk::Partial(Some(o), remaining)),
            Err(StreamingError::Error(_)) => Ok(StreamingOk::Complete(None, input.clone())),
            Err(StreamingError::Incomplete(e)) => Err(StreamingError::Incomplete(e)),
            Err(StreamingError::Failure(e)) => Err(StreamingError::Failure(e)),
        }
    }
    fn and<OtherOutput, Other: StreamingParser<Input, OtherOutput, Error, Failure>>(
        self,
        other: Other,
    ) -> impl StreamingParser<Input, (Output, OtherOutput), Error, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        (self, other).and()
    }
    fn or<Other: StreamingParser<Input, Output, Error, Failure>>(
        self,
        other: Other,
    ) -> impl StreamingParser<Input, Output, NotFound, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        (self, other).or()
    }
    fn then<O, Other: StreamingParser<Output, O, Error, Failure>>(
        self,
        other: Other,
    ) -> impl StreamingParser<Input, O, Error, Failure>
    where
        Self: Sized,
        Failure: From<Incomplete>,
    {
        StreamingCompose::map((self, other))
    }
}

pub type StreamingResult<Input, Output, Error = NotFound, Failure = Never> =
    Result<StreamingOk<Input, Output>, StreamingError<Error, Failure>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamingOk<Input, Output> {
    Complete(Output, Input),
    Partial(Output, Input),
}
impl<I, O> StreamingOk<I, O> {
    pub fn is_partial(&self) -> bool {
        match self {
            StreamingOk::Partial(_, _) => true,
            _ => false,
        }
    }
    pub fn peek(&self) -> (&O, &I) {
        match self {
            StreamingOk::Complete(o, r) => (&o, &r),
            StreamingOk::Partial(o, r) => (&o, &r),
        }
    }
}
impl<I, O: Fusable> Fusable for StreamingOk<I, O> {
    fn len(&self) -> usize {
        self.peek().0.len()
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum StreamingError<Error, Failure> {
    Incomplete(Failure),
    Error(Error),
    Failure(Failure),
}

impl<E, F> From<ParserError<E, F>> for StreamingError<E, F> {
    fn from(value: ParserError<E, F>) -> Self {
        match value {
            ParserError::Error(e) => StreamingError::Error(e),
            ParserError::Failure(e) => StreamingError::Failure(e),
        }
    }
}
impl<E, F> TryFrom<StreamingError<E, F>> for ParserError<E, F> {
    type Error = ErrorWasIncomplete;

    fn try_from(value: StreamingError<E, F>) -> Result<Self, ErrorWasIncomplete> {
        match value {
            StreamingError::Incomplete(_) => Err(ErrorWasIncomplete),
            StreamingError::Error(e) => Ok(ParserError::Error(e)),
            StreamingError::Failure(e) => Ok(ParserError::Failure(e)),
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
        T: Fn(&Input) -> StreamingResult<Input, Output, Error, Failure>,
    > StreamingParser<Input, Output, Error, Failure> for T
{
    fn parse_stream(&self, input: &Input) -> StreamingResult<Input, Output, Error, Failure> {
        self(input)
    }
}

pub trait IntoStreamingParser<Input, Output, Error, Failure> {
    fn stream(self) -> impl StreamingParser<Input, Output, Error, Failure>;
}
impl<I, O, E, F, T: Parser<I, O, E, F>> IntoStreamingParser<I, O, E, F> for T {
    fn stream(self) -> impl StreamingParser<I, O, E, F> {
        move |input: &I| match self.parse(input) {
            Ok((o, r)) => Ok(StreamingOk::Complete(o, r)),
            Err(e) => Err(e.into()),
        }
    }
}

pub trait IntoStreamingResult<Input, Output, Error, Failure> {
    fn as_streaming(self) -> StreamingResult<Input, Output, Error, Failure>;
}
impl<I, O, E, F> IntoStreamingResult<I, O, E, F> for Result<StreamingOk<I, O>, ParserError<E, F>> {
    fn as_streaming(self) -> StreamingResult<I, O, E, F> {
        self.map_err(Into::into)
    }
}
