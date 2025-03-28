use crate::{input::Input, parse::{
    Incomplete, Never, NotFound, ParserError, ParserResult, StreamingError, StreamingOk,
    StreamingResult,
}};

pub trait OrNotFound<I, O> {
    fn ok_or_not_found(self) -> ParserResult<I, O>;
}
impl<I, O> OrNotFound<I, O> for Option<(O, I)> {
    fn ok_or_not_found(self) -> ParserResult<I, O> {
        self.ok_or(ParserError::Error(NotFound))
    }
}

pub trait OrFail<I, O, F> {
    fn ok_or_fail(self) -> ParserResult<I, O, Never, F>;
}
impl<I, O, F: Default> OrFail<I, O, F> for Option<(O, I)> {
    fn ok_or_fail(self) -> ParserResult<I, O, Never, F> {
        self.ok_or_else(|| ParserError::Failure(Default::default()))
    }
}

pub trait StreamingOrNotFound<I, O> {
    fn ok_or_not_found(self) -> StreamingResult<I, O>;
}
impl<I, O> StreamingOrNotFound<I, O> for Option<StreamingOk<I, O>> {
    fn ok_or_not_found(self) -> StreamingResult<I, O> {
        self.ok_or(StreamingError::Error(NotFound))
    }
}

pub trait OrIncomplete<I, O> {
    fn ok_or_incomplete(self) -> StreamingResult<I, O, Never, Incomplete>;
}
impl<I, O> OrIncomplete<I, O> for Option<StreamingOk<I, O>> {
    fn ok_or_incomplete(self) -> StreamingResult<I, O, Never, Incomplete> {
        self.ok_or(StreamingError::Incomplete(Incomplete))
    }
}

pub trait CompleteIf<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingOk<I, O>;
    fn as_complete(self) -> StreamingOk<I, O>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> StreamingOk<I, O>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> StreamingOk<I, O>
    where
        Self: Sized,
        I: Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I, O> CompleteIf<I, O> for (O, I) {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingOk<I, O> {
        let (output, remaining) = self;
        if f(&output, &remaining) {
            StreamingOk::Complete(output, remaining)
        } else {
            StreamingOk::Partial(output, remaining)
        }
    }
}
impl<I, O> CompleteIf<I, O> for StreamingOk<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingOk<I, O> {
        match self {
            Self::Complete(o, r) | Self::Partial(o, r) => (o, r).as_complete_if(f),
        }
    }
}

pub trait MaybeCompleteIf<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> Option<StreamingOk<I, O>>;
    fn as_complete(self) -> Option<StreamingOk<I, O>>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> Option<StreamingOk<I, O>>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> Option<StreamingOk<I, O>>
    where
        Self: Sized,
        I: Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I, O> MaybeCompleteIf<I, O> for Option<(O, I)> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> Option<StreamingOk<I, O>> {
        self.map(|x| x.as_complete_if(f))
    }
}

pub trait EitherCompleteIf<I, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingResult<I, O, E, F>;
    fn as_complete(self) -> StreamingResult<I, O, E, F>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> StreamingResult<I, O, E, F>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> StreamingResult<I, O, E, F>
    where
        Self: Sized,
        I: Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I, O, E, F> EitherCompleteIf<I, O, E, F> for ParserResult<I, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingResult<I, O, E, F> {
        self.map(|x| x.as_complete_if(f)).map_err(Into::into)
    }
}
impl<I, O, E, F> EitherCompleteIf<I, O, E, F> for StreamingResult<I, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> StreamingResult<I, O, E, F> {
        self.map(|x| x.as_complete_if(f))
    }
}

pub trait NoPartial<Input, Output, Error, Failure> {
    fn no_partial(self) -> StreamingResult<Input, Output, Error, Failure>;
}
impl<I, O, E, F: From<Incomplete>> NoPartial<I, O, E, F> for StreamingResult<I, O, E, F> {
    fn no_partial(self) -> StreamingResult<I, O, E, F> {
        match self {
            Ok(StreamingOk::Complete(o, r)) => Ok(StreamingOk::Complete(o, r)),
            Ok(StreamingOk::Partial(o, r)) => Err(StreamingError::Failure(Incomplete.into())),
            Err(e) => Err(e),
        }
    }
}
