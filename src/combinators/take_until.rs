use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

pub fn take_until<I: Input, O, P: Parser<I, O>>(parser: P) -> impl Parser<I, I> {
    move |input: &I| {
        for (a, b) in crate::util::splits::splits(input) {
            match parser.parse(&b) {
                Err(ParserError::Error(_)) => (),

                Ok(_) => return Ok(input.split_at(a.len())),
                Err(ParserError::Failure(e)) => return Err(ParserError::Failure(e)),
            }
        }

        Err(ParserError::Error(NotFound))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::Choice;

    #[test]
    fn simple() {
        assert_eq!(
            take_until(("a", "b").or()).parse(&"123abc"),
            Ok(("123", "abc"))
        );

        assert_eq!(
            take_until(("d", "e").or()).parse(&"123abc"),
            Err(ParserError::Error(NotFound))
        );
    }
}
