use std::{fmt, ops::Range};

pub mod span;
pub mod string;
pub mod transform;

/// A string-like data structure which can be consumed by a `Parser`.
pub trait Input: Clone + fmt::Debug {
    type Location;

    /// Access the text as a string.
    fn as_str(&self) -> &str;
    /// Gets the current location in the input for error reporting.
    fn location(&self) -> Self::Location;
    /// Returns the length of the text.
    fn len(&self) -> usize {
        self.as_str().len()
    }
    /// Returns true if the input contains no characters.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Creates a new input containing only the characters in the given range.
    fn slice(&self, range: Range<usize>) -> Self;
    /// Splits the input at the given byte position, returning (prefix, suffix).
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized;
    /// Splits the input at the given position if valid, returning None for out-of-bounds.
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized;
    /// Returns a new input containing the first `count` characters. Panics if `count` is greater than the length of the text.
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        self.slice(0..count)
    }
    /// Returns an empty input of the same type. Location tracking is respected.
    fn take_none(&self) -> Self;
    /// Returns the first `count` characters if available, otherwise None.
    fn take_checked(&self, count: usize) -> Option<Self>
    where
        Self: Sized,
    {
        if self.as_str().get(count..).is_some() {
            // Ensure that count is within bound & a valid unicode boundary
            Some(self.take(count))
        } else {
            None
        }
    }
    /// Takes characters from the beginning while the predicate returns true, returning (taken, remaining).
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
    /// Takes characters until the predicate returns true, returning (taken, remaining).
    fn take_until<P: Fn(char) -> bool>(&self, predicate: P) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        self.as_str()
            .char_indices()
            .find(|(_, c)| predicate(*c))
            .map(|(i, _)| self.split_at(i))
    }
    /// Takes all remaining characters, returning (all_chars, empty).
    fn take_all(&self) -> (Self, Self) {
        self.split_at(self.len())
    }
    /// Returns a new input with the first `count` bytes removed.
    fn skip(&self, count: usize) -> Self {
        self.slice(count..self.len())
    }
    /// Removes and returns the tag if it matches the beginning of the input.
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
    /// Removes and returns the tag if it matches the beginning of the input, using a case-insensitive comparison.
    fn pop_no_case<T: AsRef<str>>(&self, tag: T) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        let tag = tag.as_ref();
        if let Some((s, remaining)) = self.split_at_checked(tag.len()) {
            if tag
                .chars()
                .zip(s.as_str().chars())
                .all(|(a, b)| case_insensitive_comparison(a, b))
            {
                return Some((s, remaining));
            }
        }

        None
    }
}

fn case_insensitive_comparison(a: char, b: char) -> bool {
    let mut b_lower = b.to_lowercase();
    for i in a.to_lowercase() {
        if let Some(j) = b_lower.next() {
            if i != j {
                return false;
            }
        } else {
            // a was longer than b
            return false;
        }
    }

    if b_lower.next().is_some() {
        // b was longer than a
        return false;
    }

    true
}

impl Input for &str {
    type Location = ();
    fn as_str(&self) -> &str {
        self
    }
    fn location(&self) -> Self::Location {
        ()
    }
    fn len(&self) -> usize {
        str::len(self)
    }
    fn take_none(&self) -> Self {
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
    type Location = ();

    fn as_str(&self) -> &str {
        self.as_str()
    }
    fn location(&self) -> Self::Location {
        ()
    }
    fn len(&self) -> usize {
        String::len(self)
    }
    fn take_none(&self) -> Self {
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

mod test_issues;

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

    // ISSUE: Range boundary inconsistencies in split_at implementations
    #[test]
    fn test_boundary_split_consistency() {
        let input = "hello";

        // Should be able to split at the end (creates empty suffix)
        let result = input.split_at_checked(5);
        assert!(result.is_some(), "Should allow splitting at end boundary");

        let (prefix, suffix) = result.unwrap();
        assert_eq!(prefix, "hello");
        assert_eq!(suffix, "");

        // Should fail when trying to split beyond end
        let result = input.split_at_checked(6);
        assert!(result.is_none(), "Should reject splits beyond end");
    }

    #[test]
    fn test_take_checked() {
        let s = "hello world";
        assert_eq!(s.take_checked(0), Some(""));
        assert_eq!(s.take_checked(5), Some("hello"));
        assert_eq!(s.take_checked(s.len()), Some(s));
        assert_eq!(s.take_checked(s.len() + 1), None);
    }
}
