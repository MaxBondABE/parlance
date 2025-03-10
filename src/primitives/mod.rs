/*!
# Primitive parsers

- Primitive parsers are single-order functions generic over an [`Input`][input::Input] argument and
    returning a [`ParserResult`][parse::ParserResult] of the form `Fn(&Input) -> ParserResult<Input, Output, ...>`
- Most of them return a `ParserResult<Input, Input>`.

*/

pub mod line;
pub mod numbers;
pub mod quote;
pub mod tag;
pub mod take;
pub mod whitespace;

pub use line::*;
pub use numbers::*;
pub use quote::*;
pub use tag::*;
pub use take::*;
pub use whitespace::*;
