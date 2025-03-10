use std::str::CharIndices;

use crate::input::Input;

pub fn splits<I: Input>(s: &I) -> Splits<'_, I> {
    Splits {
        content: s,
        iter: s.as_str().char_indices(),
    }
}

pub struct Splits<'a, I> {
    content: &'a I,
    iter: CharIndices<'a>,
}
impl<'a, I: Input> Iterator for Splits<'a, I> {
    type Item = (I, I);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((offset, _)) = self.iter.next() {
            Some(self.content.split_at(offset))
        } else {
            None
        }
    }
}
