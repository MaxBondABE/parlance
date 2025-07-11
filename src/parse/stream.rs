use super::{Never, NotFound, PartialParser};

pub trait StreamingParser<Input, Output, Error = NotFound, Failure = Never> {
    fn stream_parse(&self, input: &Input) -> impl TokenStream<Input, Output, Error, Failure>;
}

pub type StreamingResult<Input, Error, Failure> =
    Result<Input, StreamingError<Input, Error, Failure>>;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum StreamingError<Input, Error, Failure> {
    Error(Error),
    Failure(Failure),
    Incomplete(Input, Failure),
}

pub trait TokenStream<Input, Output, Error, Failure> {
    fn iter(&mut self) -> impl IntoIterator<Item = Output>;
    fn done(self) -> StreamingResult<Input, Error, Failure>;
}

/*
impl<I, O, E, F, T: PartialParser<I, O, E, F>> StreamingParser<I, O, E, F> for T {
    fn stream_parse(&self, input: &I) -> impl TokenStream<I, O, E, F> {
        todo!()
    }
}
*/
