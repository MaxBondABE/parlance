use crate::{input::Input, util::tuples::implement_for_tuples};

use super::{Parser, Sequence};

pub trait FuseSequence<Input, Output, Error, Failure> {
    fn output_len(self) -> impl Parser<Input, usize, Error, Failure>;
    fn fuse(self) -> impl Parser<Input, Input, Error, Failure>;
}

impl<I: Input, O: Fusable, E, F, T: Sequence<I, O, E, F>> FuseSequence<I, O, E, F> for T {
    fn output_len(self) -> impl Parser<I, usize, E, F> {
        self.and().map(|o| o.len())
    }
    fn fuse(self) -> impl Parser<I, I, E, F> {
        let parser = self.output_len();
        move |input: &I| {
            let (len, _) = parser.parse(input)?;
            Ok(input.split_at(len))
        }
    }
}

pub trait Fusable {
    fn len(&self) -> usize;
}

impl<I: crate::input::Input> Fusable for I {
    fn len(&self) -> usize {
        crate::input::Input::len(self)
    }
}
impl<I: crate::input::Input> Fusable for Option<I> {
    fn len(&self) -> usize {
        self.as_ref().map(crate::input::Input::len).unwrap_or(0)
    }
}

macro_rules! fusable_impl (
    ($($idx:literal)* . $last:literal) => (
        paste::paste! {
            impl <
                $(
                    [<Output $idx>]: Fusable,
                )*
                [<Output $last>]: Fusable,
            > Fusable for ($([<Output $idx>], )* [<Output $last>]) {
                fn len(&self) -> usize {
                    $(self.$idx.len() + )* self.$last.len()
                }
            }
        }
    )
);

implement_for_tuples!(fusable_impl);
