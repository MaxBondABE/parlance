use std::{
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use crate::{
    fuse,
    input::Input,
    parse::{Choice, Fusable, NotFound, Parser, ParserError, ParserResult, Sequence},
    primitives::tag::tag_no_case,
    util::rotate::Rotate,
};

pub fn sign<I: Input>(s: &I) -> ParserResult<I, I> {
    ("+", "-").or().parse(s)
}

pub fn digits<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_ascii_digit())
        .rot()
        .ok_or(ParserError::Error(NotFound))
}

pub fn digits_with_decimal<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!((digits, ".", digits)).parse(s)
}

pub fn plain_number<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!((sign.opt(), digits)).parse(s)
}

pub fn positive_number<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!(("+".opt(), digits)).parse(s)
}

pub fn negative_number<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!(("-", digits)).parse(s)
}

pub fn number_with_decimal<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!((sign.opt(), digits_with_decimal)).parse(s)
}

pub fn scientific_number<I: Input>(s: &I) -> ParserResult<I, I> {
    fuse!((
        sign.opt(),
        digits,
        ".".opt(),
        digits.opt(),
        tag_no_case("e"),
        sign.opt(),
        digits
    ))
    .parse(s)
}

pub fn special<I: Input>(s: &I) -> ParserResult<I, I> {
    (
        fuse!((sign.opt(), "infinity")),
        fuse!((sign.opt(), "inf")),
        "NaN",
        "nan",
    )
        .or()
        .parse(s)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberToken<I> {
    Plain(I),
    WithDecimal(I),
    Scientific(I),
    // TODO special
}
impl<I> NumberToken<I> {
    pub fn unwrap(self) -> I {
        match self {
            NumberToken::Plain(x) => x,
            NumberToken::WithDecimal(x) => x,
            NumberToken::Scientific(x) => x,
        }
    }
}
impl<I: Input> NumberToken<I> {
    pub fn parse(s: &I) -> ParserResult<I, Self> {
        (
            scientific_number.map(NumberToken::Scientific),
            number_with_decimal.map(NumberToken::WithDecimal),
            plain_number.map(NumberToken::Plain),
        )
            .or()
            .parse(s)
    }
}

pub fn integer<I: Input, O: Integer>(s: &I) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((remaining, n)) = plain_number.parse(s) {
        match O::from_str(n.as_str()) {
            Ok(output) => Ok((remaining, output)),
            Err(e) => Err(ParserError::Failure(e)),
        }
    } else {
        Err(ParserError::Error(NotFound))
    }
}

pub fn unsigned_integer<I: Input, O: UnsignedInteger>(
    s: &I,
) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((remaining, n)) = positive_number.parse(s) {
        match O::from_str(n.as_str()) {
            Ok(output) => Ok((remaining, output)),
            Err(e) => Err(ParserError::Failure(e)),
        }
    } else {
        Err(ParserError::Error(NotFound))
    }
}

pub fn real<I: Input, O: Real>(s: &I) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((remaining, n)) = NumberToken::parse(s) {
        match O::from_str(n.unwrap().as_str()) {
            Ok(output) => Ok((remaining, output)),
            Err(e) => Err(ParserError::Failure(e)),
        }
    } else {
        Err(ParserError::Error(NotFound))
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Number {
    Unsigned(u32),
    Signed(i32),
    Real(f32),
}
impl Number {
    pub fn parse<I: Input>(s: &I) -> ParserResult<I, Self, NotFound, NumberFailure> {
        match NumberToken::parse(s) {
            Ok((remaining, NumberToken::Plain(n))) => {
                if n.as_str().starts_with("-") {
                    i32::from_str(n.as_str())
                        .map(|n| (remaining, n.into()))
                        .map_err(|e| ParserError::Failure(e.into()))
                } else {
                    u32::from_str(n.as_str())
                        .map(|n| (remaining, n.into()))
                        .map_err(|e| ParserError::Failure(e.into()))
                }
            }
            Ok((remaining, NumberToken::WithDecimal(n)))
            | Ok((remaining, NumberToken::Scientific(n))) => f32::from_str(n.as_str())
                .map(|n| (remaining, n.into()))
                .map_err(|e| ParserError::Failure(e.into())),
            Err(_) => Err(ParserError::Error(NotFound)),
        }
    }
}
impl From<u32> for Number {
    fn from(v: u32) -> Self {
        Self::Unsigned(v)
    }
}
impl From<i32> for Number {
    fn from(v: i32) -> Self {
        Self::Signed(v)
    }
}
impl From<f32> for Number {
    fn from(v: f32) -> Self {
        Self::Real(v)
    }
}
impl<I: Input> Parser<I, Number, NotFound, NumberFailure> for Number {
    fn parse(&self, input: &I) -> ParserResult<I, Number, NotFound, NumberFailure> {
        Self::parse(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NumberFailure {
    Int(ParseIntError),
    Real(ParseFloatError),
}

impl From<ParseFloatError> for NumberFailure {
    fn from(v: ParseFloatError) -> Self {
        Self::Real(v)
    }
}
impl From<ParseIntError> for NumberFailure {
    fn from(v: ParseIntError) -> Self {
        Self::Int(v)
    }
}

pub trait Integer: FromStr {}
pub trait UnsignedInteger: FromStr {}
pub trait Real: FromStr {}

macro_rules! integer_impl {
    ($($int:ty, )*) => {
        $(
            impl Integer for $int {}
        )*
    }
}
integer_impl!(i8, i16, i32, i64, i128,);

macro_rules! unsigned_integer_impl {
    ($($uint:ty, )*) => {
        $(
            impl UnsignedInteger for $uint {}
        )*
    }
}
unsigned_integer_impl!(u8, u16, u32, u64, u128,);

macro_rules! float_impl {
    ($($real:ty, )*) => {
        $(
            impl Real for $real {}
        )*
    }
}
float_impl!(f32, f64,);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn plain() {
        assert_eq!(plain_number.parse(&"123"), Ok(("", "123")));
    }

    #[test]
    fn with_decimal() {
        assert_eq!(number_with_decimal.parse(&"123.45"), Ok(("", "123.45")));
    }

    #[test]
    fn scientific() {
        assert_eq!(scientific_number.parse(&"1e6"), Ok(("", "1e6")));
        assert_eq!(scientific_number.parse(&"1.0e6"), Ok(("", "1.0e6")));

        assert_eq!(scientific_number.parse(&"1E6"), Ok(("", "1E6")));
        assert_eq!(scientific_number.parse(&"1.0E6"), Ok(("", "1.0E6")));
    }

    #[test]
    fn one() {
        assert_eq!(Number::parse.parse(&"1"), Ok(("", Number::Unsigned(1))));
        assert_eq!(Number::parse.parse(&"-1"), Ok(("", Number::Signed(-1))));
        assert_eq!(Number::parse.parse(&"1.0"), Ok(("", Number::Real(1.0))));
        assert_eq!(Number::parse.parse(&"-1.0"), Ok(("", Number::Real(-1.0))));
    }

    #[test]
    fn zeroes() {
        assert_eq!(Number::parse.parse(&"0"), Ok(("", Number::Unsigned(0))));
        assert_eq!(Number::parse.parse(&"0.0"), Ok(("", Number::Real(0.0))));
        assert_eq!(Number::parse.parse(&"-0.0"), Ok(("", Number::Real(0.0))));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn uinteger_dogfood(n in u32::MIN..u32::MAX) {
            assert_eq!(Number::parse(&n.to_string().as_str()), Ok(("", Number::Unsigned(n))))
        }
        #[test]
        fn integer_dogfood(n in i32::MIN..i32::MAX) {
            match Number::parse(&n.to_string()) {
                Ok((_, Number::Signed(x))) => assert_eq!(n, x),
                Ok((_, Number::Unsigned(x))) => assert_eq!(n as u32, x),
                Ok((_, Number::Real(x))) => panic!("Integer should not be real (parsed as {})", x),
                Err(e) => panic!("Error: {:?}", e),
            }
        }
        #[test]
        fn float_dogfood(a in i32::MIN..i32::MAX, b in 0f32..1f32) {
            let n = (a as f32) * b;
            let s = format!("{:.4}", n);
            match Number::parse(&s) {
                Ok((_, Number::Real(actual))) => assert!((n - actual).abs() < 0.1),
                Ok(x) => panic!("Wrong number kind {:?}", x),
                Err(e) => panic!("Error: {:?}", e),
            };
        }
    }
}
