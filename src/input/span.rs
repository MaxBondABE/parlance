use std::{
    fmt::{self, Debug, Display},
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use super::{string::SharedString, transform::TransformContent};
use crate::{input::Input, parse::Parser, primitives::line::line};

#[derive(Clone, Default)]
pub struct Span {
    source: Arc<Source>,
    range: Range<usize>,
}
impl Span {
    pub fn new(id: impl Into<Id>, content: impl Into<String>) -> Self {
        let id = id.into();
        let content = content.into();
        let end = content.len();

        Self {
            source: Arc::new(Source::new(id, content)),
            range: 0..end,
        }
    }
    pub fn new_continued(
        id: impl Into<Id>,
        content: impl Into<String>,
        starting_line: usize,
    ) -> Self {
        let id = id.into();
        let content = content.into();
        let end = content.len();
        Self {
            source: Arc::new(Source::new_continued(id, content, starting_line)),
            range: 0..end,
        }
    }
    pub fn anonymous(content: impl Into<String>) -> Self {
        Self::new(Id::default(), content)
    }
    pub fn id(&self) -> &Id {
        &self.source.id
    }
    pub fn position(&self) -> (usize, usize) {
        match self.source.lines.binary_search(&self.range.start) {
            Ok(idx) => {
                let line = idx + 1 + self.source.starting_line;
                let column = 1;
                (line, column)
            }
            Err(idx) => {
                let line = idx + self.source.starting_line;
                let column = self.range.start - self.source.lines[idx - 1] + 1;
                (line, column)
            }
        }
    }
    pub fn location(&self) -> (&Id, (usize, usize)) {
        (self.id(), self.position())
    }
    pub fn detatch(self) -> TransformedSpan {
        let (line, column) = self.position();
        TransformedSpan::new(self.id(), line, column, self.as_str().to_string())
    }
}

impl Input for Span {
    fn as_str(&self) -> &str {
        self.as_ref()
    }
    fn len(&self) -> usize {
        self.range.len()
    }
    fn empty(&self) -> Self {
        Self {
            source: self.source.clone(),
            range: self.range.start..self.range.start,
        }
    }
    fn slice(&self, subrange: Range<usize>) -> Self {
        let start = self.range.start + subrange.start;
        assert!(self.range.contains(&start));
        let end = start + subrange.len();
        assert!(end <= self.range.end);

        Self {
            source: self.source.clone(),
            range: start..end,
        }
    }
    fn split_at(&self, mid: usize) -> (Self, Self) {
        let idx = self.range.start + mid;
        assert!(idx < self.range.end);

        (
            Self {
                source: self.source.clone(),
                range: self.range.start..idx,
            },
            Self {
                source: self.source.clone(),
                range: idx..self.range.end,
            },
        )
    }
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)> {
        let abs_mid = self.range.start + mid;
        if abs_mid >= self.range.end {
            return None;
        }
        if self.as_str().split_at_checked(mid).is_none() {
            return None;
        }

        Some((
            Self {
                source: self.source.clone(),
                range: self.range.start..abs_mid,
            },
            Self {
                source: self.source.clone(),
                range: abs_mid..self.range.end,
            },
        ))
    }
    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        let end = self.range.start + count;
        assert!(end <= self.range.end);

        Self {
            source: self.source.clone(),
            range: self.range.start..end,
        }
    }
}
impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        &self.source.content.as_str()[self.range.clone()]
    }
}
impl Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.position();
        if self.range.len() == self.source.content.len() {
            f.write_fmt(format_args!(
                "Span {} {}:{} [ {:?} ]",
                line,
                col,
                self.source.id.as_ref(),
                self.as_str()
            ))
        } else if self.range.start == 0 {
            f.write_fmt(format_args!(
                "Span {} {}:{} [ {:?} .. ]",
                line,
                col,
                self.source.id.as_ref(),
                self.as_str()
            ))
        } else if self.range.end == self.source.content.len() {
            f.write_fmt(format_args!(
                "Span {} {}:{} [ .. {:?} ]",
                line,
                col,
                self.source.id.as_ref(),
                self.as_str()
            ))
        } else {
            f.write_fmt(format_args!(
                "Span {} {}:{} [ .. {:?} .. ]",
                line,
                col,
                self.source.id.as_ref(),
                self.as_str()
            ))
        }
    }
}

#[derive(Debug)]
struct Source {
    pub id: Id,
    pub content: String,
    pub lines: Box<[usize]>,
    pub starting_line: usize,
}
impl Source {
    pub fn new(id: Id, content: String) -> Self {
        let lines = line_indexes(&content);
        Self {
            id,
            content,
            lines,
            starting_line: 0,
        }
    }
    pub fn new_continued(id: Id, content: String, starting_line: usize) -> Self {
        let lines = line_indexes(&content);
        Self {
            id,
            content,
            lines,
            starting_line,
        }
    }
}
impl Default for Source {
    fn default() -> Self {
        Self {
            id: Default::default(),
            content: Default::default(),
            lines: Default::default(),
            starting_line: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    Static(&'static str),
    String(String),
}
impl Id {
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }
}
impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        match self {
            Id::Static(s) => s,
            Id::String(s) => s.as_str(),
        }
    }
}
impl From<&'static str> for Id {
    fn from(v: &'static str) -> Self {
        Self::Static(v)
    }
}
impl From<String> for Id {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}
impl Default for Id {
    fn default() -> Self {
        Self::Static("")
    }
}

fn line_indexes(s: &str) -> Box<[usize]> {
    let mut lines = vec![0];
    let mut remaining = s;
    while !remaining.is_empty() {
        let (l, r) = line.parse(&remaining).unwrap();
        lines.push(lines.last().unwrap() + l.len());
        remaining = r;
    }
    lines.into_boxed_slice()
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransformedSpan {
    id: Arc<Id>,
    line: usize,
    column: usize,
    content: SharedString,
}
impl TransformedSpan {
    pub fn new(id: &Id, line: usize, column: usize, content: String) -> Self {
        Self {
            id: Arc::new(id.clone()),
            line,
            column,
            content: SharedString::new(content),
        }
    }

    fn with_content(&self, content: SharedString) -> Self {
        Self {
            id: self.id.clone(),
            line: self.line,
            column: self.column,
            content,
        }
    }
}
impl Input for TransformedSpan {
    fn as_str(&self) -> &str {
        self.content.as_str()
    }
    fn empty(&self) -> Self {
        self.with_content(self.content.empty())
    }
    fn slice(&self, range: Range<usize>) -> Self {
        self.with_content(self.content.slice(range))
    }
    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized,
    {
        let (a, b) = self.content.split_at(mid);
        (self.with_content(a), self.with_content(b))
    }
    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        self.content
            .split_at_checked(mid)
            .map(|(a, b)| (self.with_content(a), self.with_content(b)))
    }
}
impl AsRef<str> for TransformedSpan {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TransformContent for Span {
    type Transformed = TransformedSpan;

    fn to_content(&self, content: String) -> Self::Transformed {
        let (line, column) = self.position();
        TransformedSpan::new(self.id(), line, column, self.as_str().to_string())
    }

    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed {
        let (line, column) = self.position();
        let mut s = String::with_capacity(self.len() + content.as_ref().len());
        s.push_str(self.as_str());
        s.push_str(content.as_ref());

        TransformedSpan::new(self.id(), line, column, s)
    }
}
impl TransformContent for TransformedSpan {
    type Transformed = Self;

    fn to_content(&self, content: String) -> Self::Transformed {
        self.with_content(SharedString::new(content))
    }

    fn append_content<T: AsRef<str>>(&self, content: T) -> Self::Transformed {
        let mut s = String::with_capacity(self.len() + content.as_ref().len());
        s.push_str(self.as_str());
        s.push_str(content.as_ref());
        Self::new(&self.id, self.line, self.column, s)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_line_indexes() {
        assert_eq!(line_indexes(""), vec![0].into_boxed_slice());
    }

    #[test]
    fn single_line_indexes() {
        assert_eq!(line_indexes("foo"), vec![0, 3].into_boxed_slice());
        assert_eq!(line_indexes("foo\n"), vec![0, 4].into_boxed_slice());
        assert_eq!(line_indexes("foo\r\n"), vec![0, 5].into_boxed_slice());
    }

    #[test]
    fn multi_line_indexes() {
        assert_eq!(line_indexes("foo\nbar"), vec![0, 4, 7].into_boxed_slice());
        assert_eq!(line_indexes("foo\r\nbar"), vec![0, 5, 8].into_boxed_slice());
        assert_eq!(
            line_indexes("foo\nbar\r\nbaz"),
            vec![0, 4, 9, 12].into_boxed_slice()
        );
        assert_eq!(
            line_indexes("foo\n\nbar"),
            vec![0, 4, 5, 8].into_boxed_slice()
        );
    }

    #[test]
    fn start_of_line_locations() {
        let s = Span::anonymous("foo\nbar");
        let (first_line, second_line) = line.parse(&s).unwrap();

        assert_eq!(first_line.position(), (1, 1));
        assert_eq!(second_line.position(), (2, 1));
    }

    #[test]
    fn column_2_locations() {
        let s = Span::anonymous("foo\nbar");
        let (first_line, second_line) = line.parse(&s).unwrap();
        let first_line_col1 = first_line.skip(1);
        let second_line_col1 = second_line.skip(1);

        assert_eq!(first_line_col1.position(), (1, 2));
        assert_eq!(second_line_col1.position(), (2, 2));
    }
}
