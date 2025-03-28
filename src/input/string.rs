use std::{fmt, ops::Range, sync::Arc};

use super::{transform::TransformContent, Input};

#[derive(Clone, PartialEq)]
pub struct SharedString {
    content: Arc<String>,
    range: Range<usize>,
}
impl Input for SharedString {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
    fn len(&self) -> usize {
        self.range.len()
    }
    fn empty(&self) -> Self {
        Default::default()
    }
    fn slice(&self, range: Range<usize>) -> Self {
        let start = self.range.start + range.start;
        assert!(self.range.contains(&start));
        let end = start + range.len();
        assert!(end <= self.range.end);

        Self {
            content: self.content.clone(),
            range: start..end,
        }
    }
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized,
    {
        let idx = self.range.start + mid;
        assert!(idx < self.range.end);

        (
            Self {
                content: self.content.clone(),
                range: self.range.start..idx,
            },
            Self {
                content: self.content.clone(),
                range: idx..self.range.end,
            },
        )
    }
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        let idx = self.range.start + mid;
        if idx < self.range.end {
            Some((
                Self {
                    content: self.content.clone(),
                    range: self.range.start..idx,
                },
                Self {
                    content: self.content.clone(),
                    range: idx..self.range.end,
                },
            ))
        } else {
            None
        }
    }
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        let end = self.range.start + count;
        assert!(end <= self.range.end);

        Self {
            content: self.content.clone(),
            range: self.range.start..end,
        }
    }
}
impl SharedString {
    pub fn new(s: String) -> Self {
        let range = 0..s.len();
        Self {
            content: Arc::new(s),
            range,
        }
    }
}
impl fmt::Debug for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.range.len() == self.content.len() {
            f.write_fmt(format_args!("SharedString [ {:?} ]", self.as_str()))
        } else if self.range.start == 0 {
            f.write_fmt(format_args!("SharedString [ {:?} .. ]", self.as_str()))
        } else if self.range.end == self.content.len() {
            f.write_fmt(format_args!("SharedString [ .. {:?} ]", self.as_str()))
        } else {
            f.write_fmt(format_args!("SharedString [ .. {:?} .. ]", self.as_str()))
        }
    }
}
impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        str::fmt(self.as_str(), f)
    }
}
impl AsRef<str> for SharedString {
    fn as_ref(&self) -> &str {
        &self.content[self.range.clone()]
    }
}
impl Default for SharedString {
    fn default() -> Self {
        Self {
            content: Arc::new(Default::default()),
            range: 0..0,
        }
    }
}

impl TransformContent for SharedString {
    type Transformed = Self;

    fn to_content(&self, content: String) -> Self::Transformed {
        Self::new(content)
    }

    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed {
        let mut s = String::with_capacity(self.len() + content.as_ref().len());
        s.push_str(self.as_str());
        s.push_str(content.as_ref());
        Self::new(s)
    }
}
