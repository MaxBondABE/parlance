use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserError},
};

pub fn take_while0<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_while(&predicate) {
            Ok((remaining, s))
        } else {
            Ok((input.clone(), input.empty()))
        }
    }
}

pub fn take_while<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_while(&predicate) {
            Ok((remaining, s))
        } else {
            Err(ParserError::Error(NotFound))
        }
    }
}

pub fn take_until0<I: Input, F, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_until(&predicate) {
            Ok((remaining, s))
        } else {
            Ok((input.clone(), input.empty()))
        }
    }
}

pub fn take_until<I: Input, Predicate: Fn(char) -> bool>(
    predicate: Predicate,
) -> impl Parser<I, I> {
    move |input: &I| {
        if let Some((s, remaining)) = input.take_until(&predicate) {
            Ok((remaining, s))
        } else {
            Err(ParserError::Error(NotFound))
        }
    }
}
