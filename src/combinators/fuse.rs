#[macro_export]
macro_rules! strfuse (
    ($x: expr) => {
        {
            {
                use $crate::parse::{Sequence, FuseSequence};
                fn fuse_impl<I: $crate::input::Input>(s: &I) -> $crate::parse::ParserResult<I, I> {
                    if let Ok((len, _)) = $x.output_len().parse(&s.as_str()) {
                        Ok(s.split_at(len))
                    } else {
                        Err($crate::parse::ParserError::Error($crate::parse::NotFound))
                    }
                }

                fuse_impl
            }
        }
    }
);

pub use strfuse;

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::{
        Fusable, FuseSequence, Parser, ParserResult, PartialOk, PartialParser, Sequence,
    };

    #[test]
    fn simple() {
        assert_eq!(strfuse!(("a", "b")).parse(&"abc"), Ok(("ab", "c")));

        fn foo<'a, 'b>(s: &'a &'b str) -> ParserResult<&'b str, &'b str> {
            return Ok(("", *s));
        }
        assert_eq!(strfuse!(("a", "b", foo)).parse(&"abc"), Ok(("ab", "c")))
    }
}
