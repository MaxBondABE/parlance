use crate::{
    input::Input,
    parse::{
        Missing, Never, NotFound, Parser, ParserError, PartialError, PartialOk, PartialParser,
    },
    util::splits::splits,
};

pub fn seek<I: Input, O, E, F, P: Parser<I, O, E, F>>(parser: P) -> impl Parser<I, I, NotFound, F> {
    move |input: &I| {
        for (head, tail) in splits(input) {
            match parser.parse(&tail) {
                Err(ParserError::Error(_)) => (),

                Ok(_) => return Ok(input.split_at(head.len())),
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            }
        }

        Err(ParserError::Error(NotFound))
    }
}

pub fn partial_seek<I: Input, O, E, F: From<Missing>, P: PartialParser<I, O, E, F>>(
    parser: P,
) -> impl PartialParser<I, I, E, F> {
    move |input: &I| {
        for (head, tail) in splits(input) {
            match parser.partial_parse(&tail) {
                Err(PartialError::Error(_)) => (),

                Ok(_) => {
                    let (o, r) = input.split_at(head.len());
                    return Ok(PartialOk::Complete(o, r));
                }
                Err(PartialError::Incomplete(_)) => {
                    return Err(PartialError::Incomplete(Missing.into()))
                }
                Err(PartialError::Failure(e)) => return Err(PartialError::Failure(e)),
            }
        }

        Err(PartialError::Incomplete(Missing.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::Choice;

    #[test]
    fn simple() {
        assert_eq!(seek(("a", "b").or()).parse(&"123abc"), Ok(("123", "abc")));

        assert_eq!(
            seek(("d", "e").or()).parse(&"123abc"),
            Err(ParserError::Error(NotFound))
        );
    }
}
