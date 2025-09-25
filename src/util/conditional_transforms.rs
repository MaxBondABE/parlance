use crate::parse::{
    Missing, Never, NotFound, Parser, ParserError, ParserResult, PartialError, PartialOk,
    PartialResult,
};

pub trait OkOrNotFound<Input: crate::input::Input, Output> {
    fn ok_or_not_found(self, location: Input::Location) -> ParserResult<Input, Output>;
}
impl<I: crate::input::Input, O> OkOrNotFound<I, O> for Option<(O, I)> {
    fn ok_or_not_found(self, location: I::Location) -> ParserResult<I, O> {
        self.ok_or(ParserError::Error(NotFound, location))
    }
}

pub trait OkOrFail<Input: crate::input::Input, Output, Failure> {
    fn ok_or_fail(self, location: Input::Location) -> ParserResult<Input, Output, Never, Failure>;
}
impl<I: crate::input::Input, O, F: Default> OkOrFail<I, O, F> for Option<(O, I)> {
    fn ok_or_fail(self, location: I::Location) -> ParserResult<I, O, Never, F> {
        self.ok_or_else(|| ParserError::Failure(Default::default(), location))
    }
}

pub trait PartialOkOrNotFound<I: crate::input::Input, O> {
    fn ok_or_not_found(self, location: I::Location) -> PartialResult<I, O>;
}
impl<I: crate::input::Input, O> PartialOkOrNotFound<I, O> for Option<PartialOk<I, O>> {
    fn ok_or_not_found(self, location: I::Location) -> PartialResult<I, O> {
        self.ok_or(PartialError::Error(NotFound, location))
    }
}

pub trait OkOrIncomplete<I: crate::input::Input, O> {
    fn ok_or_incomplete(self, location: I::Location) -> PartialResult<I, O, Never, Missing>;
}
impl<I: crate::input::Input, O> OkOrIncomplete<I, O> for Option<PartialOk<I, O>> {
    fn ok_or_incomplete(self, location: I::Location) -> PartialResult<I, O, Never, Missing> {
        self.ok_or(PartialError::Incomplete(Missing, location))
    }
}

pub trait CompleteIf<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialOk<I, O>;
    fn as_complete(self) -> PartialOk<I, O>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> PartialOk<I, O>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> PartialOk<I, O>
    where
        Self: Sized,
        I: crate::input::Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I, O> CompleteIf<I, O> for (O, I) {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialOk<I, O> {
        let (output, remaining) = self;
        if f(&output, &remaining) {
            PartialOk::Complete(output, remaining)
        } else {
            PartialOk::Partial(output, remaining)
        }
    }
}
impl<I, O> CompleteIf<I, O> for PartialOk<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialOk<I, O> {
        match self {
            Self::Complete(o, r) | Self::Partial(o, r) => (o, r).as_complete_if(f),
        }
    }
}

pub trait MaybeCompleteIf<I, O> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> Option<PartialOk<I, O>>;
    fn as_complete(self) -> Option<PartialOk<I, O>>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> Option<PartialOk<I, O>>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> Option<PartialOk<I, O>>
    where
        Self: Sized,
        I: crate::input::Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I, O> MaybeCompleteIf<I, O> for Option<(O, I)> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> Option<PartialOk<I, O>> {
        self.map(|x| x.as_complete_if(f))
    }
}

pub trait EitherCompleteIf<I: crate::input::Input, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialResult<I, O, E, F>;
    fn as_complete(self) -> PartialResult<I, O, E, F>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| true)
    }
    fn as_partial(self) -> PartialResult<I, O, E, F>
    where
        Self: Sized,
    {
        self.as_complete_if(|_, _| false)
    }
    fn has_stopped(self) -> PartialResult<I, O, E, F>
    where
        Self: Sized,
        I: crate::input::Input,
    {
        self.as_complete_if(|_, remaining| !remaining.is_empty())
    }
}
impl<I: crate::input::Input, O, E, F> EitherCompleteIf<I, O, E, F> for ParserResult<I, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialResult<I, O, E, F> {
        self.map(|x| x.as_complete_if(f)).map_err(Into::into)
    }
}
impl<I: crate::input::Input, O, E, F> EitherCompleteIf<I, O, E, F> for PartialResult<I, O, E, F> {
    fn as_complete_if<Func: Fn(&O, &I) -> bool>(self, f: Func) -> PartialResult<I, O, E, F> {
        self.map(|x| x.as_complete_if(f))
    }
}

pub trait NoPartial<Input: crate::input::Input, Output, Error, Failure> {
    fn no_partial(self) -> PartialResult<Input, Output, Error, Failure>;
    fn no_partial_at(
        self,
        location: Input::Location,
    ) -> PartialResult<Input, Output, Error, Failure>;
}
impl<I: crate::input::Input, O, E, F: From<Missing>> NoPartial<I, O, E, F>
    for PartialResult<I, O, E, F>
{
    fn no_partial(self) -> PartialResult<I, O, E, F> {
        match self {
            Ok(PartialOk::Complete(o, r)) => Ok(PartialOk::Complete(o, r)),
            Ok(PartialOk::Partial(o, r)) => {
                Err(PartialError::Failure(Missing.into(), r.location()))
            }
            Err(e) => Err(e),
        }
    }
    fn no_partial_at(self, location: I::Location) -> PartialResult<I, O, E, F> {
        match self {
            Ok(PartialOk::Complete(o, r)) => Ok(PartialOk::Complete(o, r)),
            Ok(PartialOk::Partial(o, r)) => Err(PartialError::Failure(Missing.into(), location)),
            Err(e) => Err(e),
        }
    }
}
