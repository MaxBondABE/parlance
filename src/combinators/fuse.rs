#[macro_export]
macro_rules! fuse (
    ($x: expr) => {
        {
            {
                use $crate::parse::{Sequence, FuseSequence};
                fn fuse_impl<I: $crate::input::Input>(s: &I) -> $crate::parse::ParserResult<I, I> {
                    if let Ok((_, len)) = $x.output_len().parse(&s.as_str()) {
                        let (output, remaining) = s.split_at(len);
                        Ok((remaining, output))
                    } else {
                        Err($crate::parse::ParserError::Error($crate::parse::NotFound))
                    }
                }

                fuse_impl
            }
        }
    }
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::{Fusable, FuseSequence, Parser, ParserResult, Sequence};

    #[test]
    fn simple() {
        assert_eq!(fuse!(("a", "b")).parse(&"abc"), Ok(("c", "ab")));

        fn foo<'a, 'b>(s: &'a &'b str) -> ParserResult<&'b str, &'b str> {
            return Ok((*s, ""));
        }
        assert_eq!(fuse!(("a", "b", foo)).parse(&"abc"), Ok(("c", "ab")))
    }

}
