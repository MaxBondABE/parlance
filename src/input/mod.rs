use std::{fmt, ops::Range};

pub mod span;
pub mod string;
pub mod transform;

pub trait Input: Clone + fmt::Debug {
    fn as_str(&self) -> &str;
    fn len(&self) -> usize {
        self.as_str().len()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn empty(&self) -> Self;
    fn slice(&self, range: Range<usize>) -> Self;
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized;
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized;
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        self.slice(0..count)
    }
    fn take_checked(&self, count: usize) -> Option<Self>
    where
        Self: Sized,
    {
        if self.len() <= count {
            Some(self.take(count))
        } else {
            None
        }
    }
    fn take_while<P: Fn(char) -> bool>(&self, predicate: P) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        self.as_str()
            .char_indices()
            .take_while(|(_, c)| predicate(*c))
            .last()
            .map(|(i, c)| self.split_at(i + c.len_utf8()))
    }
    fn take_until<P: Fn(char) -> bool>(&self, predicate: P) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        self.as_str()
            .char_indices()
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| self.split_at(i))
    }
    fn skip(&self, count: usize) -> Self {
        self.slice(count..self.len())
    }
    fn pop<T: AsRef<str>>(&self, tag: &T) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        let tag = tag.as_ref();
        if let Some((s, remaining)) = self.split_at_checked(tag.len()) {
            if s.as_str() == tag {
                return Some((s, remaining));
            }
        }

        None
    }
    fn pop_no_case<T: AsRef<str>>(&self, tag: T) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        let tag = tag.as_ref();
        if let Some((s, remaining)) = self.split_at_checked(tag.len()) {
            if tag
                .chars()
                .zip(s.as_str().chars())
                .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
            {
                return Some((s, remaining));
            }
        }

        None
    }
}

impl Input for &str {
    fn as_str(&self) -> &str {
        self
    }
    fn len(&self) -> usize {
        str::len(self)
    }
    fn empty(&self) -> Self {
        Default::default()
    }
    fn slice(&self, range: Range<usize>) -> Self {
        &self[range]
    }
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized,
    {
        str::split_at(self, mid)
    }
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        str::split_at_checked(self, mid)
    }
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        &self[0..count]
    }
}

impl Input for String {
    fn as_str(&self) -> &str {
        self.as_str()
    }
    fn len(&self) -> usize {
        String::len(self)
    }
    fn empty(&self) -> Self {
        Default::default()
    }
    fn slice(&self, range: Range<usize>) -> Self {
        self[range].to_string()
    }
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized,
    {
        let (a, b) = self.as_str().split_at(mid);
        (a.to_string(), b.to_string())
    }
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        if let Some((a, b)) = self.as_str().split_at_checked(mid) {
            Some((a.to_string(), b.to_string()))
        } else {
            None
        }
    }
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        self.as_str().take(count).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_while_is_inclusive() {
        assert_eq!("xxxyyy".take_while(|c| c == 'x'), Some(("xxx", "yyy")));
        assert_eq!("xxxyyy".take_while(|c| c == 'y'), None);
    }

    #[test]
    fn take_until_is_inclusive() {
        assert_eq!("xxxyyy".take_until(|c| c == 'y'), Some(("xxx", "yyy")));
        assert_eq!("xxxyyy".take_until(|c| c == 'z'), None);
    }
}
