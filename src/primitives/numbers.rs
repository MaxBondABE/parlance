use std::{
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};

use crate::{
    input::Input,
    parse::{Choice, Fusable, NotFound, Parser, ParserError, ParserResult, Sequence},
    primitives::tag::tag_no_case,
    strfuse,
    util::conditional_transforms::OkOrNotFound,
};

/// Parses a sign character (`+` or `-`).
/// ```
/// # use parlance::primitives::numbers::sign;
/// # use parlance::parse::Parser;
/// assert_eq!(sign.parse(&"+123"), Ok(("+", "123")));
/// assert_eq!(sign.parse(&"-456"), Ok(("-", "456")));
/// ```
pub fn sign<I: Input>(s: &I) -> ParserResult<I, I> {
    ("+", "-").or().parse(s)
}

/// Parses one or more ASCII digits.
/// ```
/// # use parlance::primitives::numbers::digits;
/// # use parlance::parse::Parser;
/// assert_eq!(digits.parse(&"123abc"), Ok(("123", "abc")));
/// ```
pub fn digits<I: Input>(s: &I) -> ParserResult<I, I> {
    s.take_while(|c| c.is_ascii_digit())
        .ok_or_not_found(s.location())
}

/// Parses digits with a decimal point (e.g., "123.456").
/// ```
/// # use parlance::primitives::numbers::digits_with_decimal;
/// # use parlance::parse::Parser;
/// assert_eq!(digits_with_decimal.parse(&"12.34"), Ok(("12.34", "")));
/// ```
pub fn digits_with_decimal<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!((digits, ".", digits)).parse(s)
}

/// Parses a signed integer (with optional sign).
/// ```
/// # use parlance::primitives::numbers::signed_integer_number;
/// # use parlance::parse::Parser;
/// assert_eq!(signed_integer_number.parse(&"123"), Ok(("123", "")));
/// assert_eq!(signed_integer_number.parse(&"-456"), Ok(("-456", "")));
/// ```
pub fn signed_integer_number<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!((sign.opt(), digits)).parse(s)
}

/// Parses a positive integer (with optional `+` sign).
/// ```
/// # use parlance::primitives::numbers::positive_integer_number;
/// # use parlance::parse::Parser;
/// assert_eq!(positive_integer_number.parse(&"123"), Ok(("123", "")));
/// assert_eq!(positive_integer_number.parse(&"+456"), Ok(("+456", "")));
/// ```
pub fn positive_integer_number<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("+".opt(), digits)).parse(s)
}

/// Parses a negative integer (requires `-` sign).
/// ```
/// # use parlance::primitives::numbers::negative_integer_number;
/// # use parlance::parse::Parser;
/// assert_eq!(negative_integer_number.parse(&"-123"), Ok(("-123", "")));
/// ```
pub fn negative_integer_number<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!(("-", digits)).parse(s)
}

/// Parses a decimal number with optional sign.
/// ```
/// # use parlance::primitives::numbers::number_with_decimal;
/// # use parlance::parse::Parser;
/// assert_eq!(number_with_decimal.parse(&"12.34"), Ok(("12.34", "")));
/// assert_eq!(number_with_decimal.parse(&"-56.78"), Ok(("-56.78", "")));
/// ```
pub fn number_with_decimal<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!((sign.opt(), digits_with_decimal)).parse(s)
}

/// Parses a number in scientific notation (e.g., "1.23e-4").
/// ```
/// # use parlance::primitives::numbers::scientific_number;
/// # use parlance::parse::Parser;
/// assert_eq!(scientific_number.parse(&"1.23e5"), Ok(("1.23e5", "")));
/// assert_eq!(scientific_number.parse(&"-4E-2"), Ok(("-4E-2", "")));
/// ```
pub fn scientific_number<I: Input>(s: &I) -> ParserResult<I, I> {
    strfuse!((
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

/// Parses special floating-point values (infinity, NaN).
/// ```
/// # use parlance::primitives::numbers::special_number;
/// # use parlance::parse::Parser;
/// assert_eq!(special_number.parse(&"inf"), Ok(("inf", "")));
/// assert_eq!(special_number.parse(&"NaN"), Ok(("NaN", "")));
/// assert_eq!(special_number.parse(&"-infinity"), Ok(("-infinity", "")));
/// ```
pub fn special_number<I: Input>(s: &I) -> ParserResult<I, I> {
    (
        strfuse!((
            sign.opt(),
            tag_no_case("inf"),
            tag_no_case("inity").opt() // Handle inf vs infinity as an optional suffix
        )),
        tag_no_case("NaN"),
    )
        .or()
        .parse(s)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberToken<I> {
    Plain(I),
    WithDecimal(I),
    Scientific(I),
    Special(I),
}
impl<I> NumberToken<I> {
    pub fn unwrap(self) -> I {
        match self {
            NumberToken::Plain(x) => x,
            NumberToken::WithDecimal(x) => x,
            NumberToken::Scientific(x) => x,
            NumberToken::Special(x) => x,
        }
    }
}
impl<I: Input> NumberToken<I> {
    pub fn parse(s: &I) -> ParserResult<I, Self> {
        (
            scientific_number.map(NumberToken::Scientific),
            number_with_decimal.map(NumberToken::WithDecimal),
            signed_integer_number.map(NumberToken::Plain),
            special_number.map(NumberToken::Special),
        )
            .or()
            .parse(s)
    }
}

/// Parses a signed integer and converts it to the specified type.
/// ```
/// # use parlance::primitives::numbers::integer;
/// # use parlance::parse::Parser;
/// assert_eq!(integer::<_, i32>.parse(&"42"), Ok((42, "")));
/// assert_eq!(integer::<_, i64>.parse(&"-123"), Ok((-123, "")));
/// ```
pub fn integer<I: Input, O: Integer>(s: &I) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((n, remaining)) = signed_integer_number.parse(s) {
        match O::from_str(n.as_str()) {
            Ok(output) => Ok((output, remaining)),
            Err(e) => Err(ParserError::Failure(e, s.location())),
        }
    } else {
        Err(ParserError::Error(NotFound, s.location()))
    }
}

/// Parses an unsigned integer and converts it to the specified type.
/// ```
/// # use parlance::primitives::numbers::unsigned_integer;
/// # use parlance::parse::Parser;
/// assert_eq!(unsigned_integer::<_, u32>.parse(&"42"), Ok((42, "")));
/// assert_eq!(unsigned_integer::<_, u64>.parse(&"+123"), Ok((123, "")));
/// ```
pub fn unsigned_integer<I: Input, O: UnsignedInteger>(
    s: &I,
) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((n, remaining)) = positive_integer_number.parse(s) {
        match O::from_str(n.as_str()) {
            Ok(output) => Ok((output, remaining)),
            Err(e) => Err(ParserError::Failure(e, s.location())),
        }
    } else {
        Err(ParserError::Error(NotFound, s.location()))
    }
}

/// Parses a real number and converts it to the specified floating-point type.
/// ```
/// # use parlance::primitives::numbers::real;
/// # use parlance::parse::Parser;
/// assert_eq!(real::<_, f32>.parse(&"3.14"), Ok((3.14, "")));
/// assert_eq!(real::<_, f64>.parse(&"1.23e-4"), Ok((1.23e-4, "")));
/// ```
pub fn real<I: Input, O: Real>(s: &I) -> ParserResult<I, O, NotFound, <O as FromStr>::Err> {
    if let Ok((n, remaining)) = NumberToken::parse(s) {
        match O::from_str(n.unwrap().as_str()) {
            Ok(output) => Ok((output, remaining)),
            Err(e) => Err(ParserError::Failure(e, s.location())),
        }
    } else {
        Err(ParserError::Error(NotFound, s.location()))
    }
}

// TODO comment + doctest
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Number {
    Unsigned(u32),
    Signed(i32),
    Real(f32),
}
impl Number {
    pub fn parse<I: Input>(s: &I) -> ParserResult<I, Self, NotFound, NumberFailure> {
        match NumberToken::parse(s) {
            Ok((NumberToken::Plain(n), remaining)) => {
                if n.as_str().starts_with("-") {
                    i32::from_str(n.as_str())
                        .map(|n| (n.into(), remaining))
                        .map_err(|e| ParserError::Failure(e.into(), s.location()))
                } else {
                    u32::from_str(n.as_str())
                        .map(|n| (n.into(), remaining))
                        .map_err(|e| ParserError::Failure(e.into(), s.location()))
                }
            }
            Ok((NumberToken::WithDecimal(n), remaining))
            | Ok((NumberToken::Scientific(n), remaining))
            | Ok((NumberToken::Special(n), remaining)) => f32::from_str(n.as_str())
                .map(|n| (n.into(), remaining))
                .map_err(|e| ParserError::Failure(e.into(), s.location())),
            Err(_) => Err(ParserError::Error(NotFound, s.location())),
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
integer_impl!(i8, i16, i32, i64, i128, isize,);

macro_rules! unsigned_integer_impl {
    ($($uint:ty, )*) => {
        $(
            impl UnsignedInteger for $uint {}
        )*
    }
}
unsigned_integer_impl!(u8, u16, u32, u64, u128, usize,);

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
        assert_eq!(signed_integer_number.parse(&"123"), Ok(("123", "")));
        assert_eq!(signed_integer_number.parse(&"+123"), Ok(("+123", "")));
    }

    #[test]
    fn with_decimal() {
        assert_eq!(number_with_decimal.parse(&"123.45"), Ok(("123.45", "")));
    }

    #[test]
    fn scientific() {
        assert_eq!(scientific_number.parse(&"1e6"), Ok(("1e6", "")));
        assert_eq!(scientific_number.parse(&"1.0e6"), Ok(("1.0e6", "")));

        assert_eq!(scientific_number.parse(&"1E6"), Ok(("1E6", "")));
        assert_eq!(scientific_number.parse(&"1.0E6"), Ok(("1.0E6", "")));
    }

    #[test]
    fn one() {
        assert_eq!(Number::parse.parse(&"1"), Ok((Number::Unsigned(1), "")));
        assert_eq!(Number::parse.parse(&"+1"), Ok((Number::Unsigned(1), "")));
        assert_eq!(Number::parse.parse(&"-1"), Ok((Number::Signed(-1), "")));
        assert_eq!(Number::parse.parse(&"1.0"), Ok((Number::Real(1.0), "")));
        assert_eq!(Number::parse.parse(&"+1.0"), Ok((Number::Real(1.0), "")));
        assert_eq!(Number::parse.parse(&"-1.0"), Ok((Number::Real(-1.0), "")));
    }

    #[test]
    fn zeroes() {
        assert_eq!(Number::parse.parse(&"0"), Ok((Number::Unsigned(0), "")));
        assert_eq!(Number::parse.parse(&"0.0"), Ok((Number::Real(0.0), "")));
        assert_eq!(Number::parse.parse(&"+0.0"), Ok((Number::Real(0.0), "")));
        assert_eq!(Number::parse.parse(&"-0.0"), Ok((Number::Real(0.0), "")));
    }

    #[test]
    fn special_floats() {
        assert_eq!(special_number.parse(&"nan"), Ok(("nan", "")));
        assert_eq!(special_number.parse(&"NaN"), Ok(("NaN", "")));
        assert_eq!(special_number.parse(&"inf"), Ok(("inf", "")));
        assert_eq!(special_number.parse(&"+inf"), Ok(("+inf", "")));
        assert_eq!(special_number.parse(&"-inf"), Ok(("-inf", "")));

        /// NaN != NaN so can't use assert_eq!()
        match Number::parse.parse(&f32::NAN.to_string()) {
            Ok((Number::Real(x), _)) => {
                assert!(x.is_nan())
            }
            Ok(x) => panic!("Wrong type of number: {:?}", x),
            Err(e) => panic!("Error: {:?}", e),
        };
        match Number::parse.parse(&"nan") {
            Ok((Number::Real(x), _)) => {
                assert!(x.is_nan())
            }
            Ok(x) => panic!("Wrong type of number: {:?}", x),
            Err(e) => panic!("Error: {:?}", e),
        };
        match Number::parse.parse(&"NaN") {
            Ok((Number::Real(x), _)) => {
                assert!(x.is_nan())
            }
            Ok(x) => panic!("Error parsing special float: Wrong type of number: {:?}", x),
            Err(e) => panic!("Error parsing special float: {:?}", e),
        };

        assert_eq!(
            Number::parse.parse(&f32::INFINITY.to_string()),
            Ok((Number::Real(f32::INFINITY), "".to_string()))
        );
        assert_eq!(
            Number::parse.parse(&"inf"),
            Ok((Number::Real(f32::INFINITY), ""))
        );
        assert_eq!(
            Number::parse.parse(&"infinity"),
            Ok((Number::Real(f32::INFINITY), ""))
        );
        assert_eq!(
            Number::parse.parse(&"+inf"),
            Ok((Number::Real(f32::INFINITY), ""))
        );
        assert_eq!(
            Number::parse.parse(&"+infinity"),
            Ok((Number::Real(f32::INFINITY), ""))
        );

        assert_eq!(
            Number::parse.parse(&f32::NEG_INFINITY.to_string()),
            Ok((Number::Real(f32::NEG_INFINITY), "".to_string()))
        );
        assert_eq!(
            Number::parse.parse(&"-inf"),
            Ok((Number::Real(f32::NEG_INFINITY), ""))
        );
        assert_eq!(
            Number::parse.parse(&"-infinity"),
            Ok((Number::Real(f32::NEG_INFINITY), ""))
        );
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn uinteger_dogfood(n in u32::MIN..u32::MAX) {
            assert_eq!(Number::parse(&n.to_string().as_str()), Ok((Number::Unsigned(n), "")))
        }
        #[test]
        fn integer_dogfood(n in i32::MIN..i32::MAX) {
            match Number::parse(&n.to_string()) {
                Ok((Number::Signed(x), _)) => assert_eq!(n, x),
                Ok((Number::Unsigned(x), _)) => assert_eq!(n as u32, x),
                Ok((Number::Real(x), _)) => panic!("Integer should not be real (parsed as {})", x),
                Err(e) => panic!("Error: {:?}", e),
            }
        }
        #[test]
        fn float_dogfood(a in i32::MIN..i32::MAX, b in 0f32..1f32) {
            let n = (a as f32) * b;
            let s = format!("{:.4}", n);
            match Number::parse(&s) {
                Ok((Number::Real(actual), _)) => assert!((n - actual).abs() < 0.1),
                // Because we used 4 decimal points when we built the string, the absolute error should
                // be less than 1 decimal point
                Ok(x) => panic!("Wrong number kind {:?}", x),
                Err(e) => panic!("Error: {:?}", e),
            };
        }
    }
}
