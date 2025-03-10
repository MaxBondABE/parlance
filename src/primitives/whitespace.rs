use crate::{
    input::Input,
    parse::{NotFound, Parser, ParserResult},
};

use super::take::take_while;

pub fn whitespace<I: Input>(s: &I) -> ParserResult<I, I> {
    take_while(|c| c.is_whitespace()).parse(s)
}
