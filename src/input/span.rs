use std::{
    fmt::{self, Debug, Display},
    ops::Range,
    path::PathBuf,
    sync::Arc,
};

use super::{string::SharedString, transform::TransformContent};
use crate::{input::Input, parse::Parser, primitives::line::line};

#[derive(Clone, Debug)]
pub enum Span {
    Original {
        source: Arc<Source>,
        range: Range<usize>,
    },
    Transformed {
        id: Id,
        line: usize,
        column: usize,
        content: SharedString,
    },
}

impl Span {
    pub fn new(id: impl Into<Id>, content: impl Into<String>) -> Self {
        let id = id.into();
        let content = content.into();
        let end = content.len();

        Self::Original {
            source: Arc::new(Source::new(id, content)),
            range: 0..end,
        }
    }

    pub fn new_continued(
        id: impl Into<Id>,
        content: impl Into<String>,
        starting_line: usize,
    ) -> Self {
        // FIXME zero versus 1 indexed starting line
        let id = id.into();
        let content = content.into();
        let end = content.len();
        Self::Original {
            source: Arc::new(Source::new_continued(id, content, starting_line)),
            range: 0..end,
        }
    }

    pub fn anonymous(content: impl Into<String>) -> Self {
        Self::new(Id::default(), content)
    }

    pub fn id(&self) -> &Id {
        match self {
            Self::Original { source, .. } => &source.id,
            Self::Transformed { id, .. } => id,
        }
    }

    pub fn position(&self) -> (usize, usize) {
        match self {
            Self::Original { source, range } => match source.lines.binary_search(&range.start) {
                Ok(idx) => {
                    let line = idx + 1 + source.starting_line;
                    let column = 1;
                    (line, column)
                }
                Err(idx) => {
                    let line = idx + source.starting_line;
                    let column = range.start - source.lines[idx - 1] + 1;
                    (line, column)
                }
            },
            Self::Transformed { line, column, .. } => (*line, *column),
        }
    }

    pub fn detatch(self) -> Self {
        match self {
            Self::Original { source, range } => {
                let span = Self::Original { source, range };
                let (line, column) = span.position();
                Self::Transformed {
                    id: span.id().clone(),
                    line,
                    column,
                    content: SharedString::new(span.as_str().to_string()),
                }
            }
            transformed @ Self::Transformed { .. } => transformed,
        }
    }

    fn new_transformed(id: &Id, line: usize, column: usize, content: String) -> Self {
        Self::Transformed {
            id: id.clone(),
            line,
            column,
            content: SharedString::new(content),
        }
    }

    fn with_content(&self, content: SharedString) -> Self {
        match self {
            Self::Transformed {
                id, line, column, ..
            } => Self::Transformed {
                id: id.clone(),
                line: *line,
                column: *column,
                content,
            },
            Self::Original { .. } => {
                let (line, column) = self.position();
                Self::Transformed {
                    id: self.id().clone(),
                    line,
                    column,
                    content,
                }
            }
        }
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::Original {
            source: Default::default(),
            range: 0..0,
        }
    }
}

impl Input for Span {
    type Location = (Id, (usize, usize));

    fn as_str(&self) -> &str {
        self.as_ref()
    }

    fn location(&self) -> Self::Location {
        (self.id().clone(), self.position())
    }

    fn len(&self) -> usize {
        match self {
            Self::Original { range, .. } => range.len(),
            Self::Transformed { content, .. } => content.len(),
        }
    }

    fn take_none(&self) -> Self {
        match self {
            Self::Original { source, range } => Self::Original {
                source: source.clone(),
                range: range.start..range.start,
            },
            Self::Transformed { .. } => self.with_content(SharedString::default()),
        }
    }

    fn slice(&self, subrange: Range<usize>) -> Self {
        match self {
            Self::Original { source, range } => {
                let start = range.start + subrange.start;
                assert!(range.contains(&start));
                let end = start + subrange.len();
                assert!(end <= range.end);

                Self::Original {
                    source: source.clone(),
                    range: start..end,
                }
            }
            Self::Transformed { content, .. } => self.with_content(content.slice(subrange)),
        }
    }

    fn split_at(&self, mid: usize) -> (Self, Self)
    where
        Self: Sized,
    {
        match self {
            Self::Original { source, range } => {
                let idx = range.start + mid;
                assert!(idx <= range.end);

                (
                    Self::Original {
                        source: source.clone(),
                        range: range.start..idx,
                    },
                    Self::Original {
                        source: source.clone(),
                        range: idx..range.end,
                    },
                )
            }
            Self::Transformed { content, .. } => {
                let (a, b) = content.split_at(mid);
                (self.with_content(a), self.with_content(b))
            }
        }
    }

    fn split_at_checked(&self, mid: usize) -> Option<(Self, Self)>
    where
        Self: Sized,
    {
        match self {
            Self::Original { source, range } => {
                let abs_mid = range.start + mid;
                if self.as_str().get(abs_mid..).is_none() {
                    // Ensure that the midpoint is within bound & a valid unicode boundary
                    return None;
                }

                Some((
                    Self::Original {
                        source: source.clone(),
                        range: range.start..abs_mid,
                    },
                    Self::Original {
                        source: source.clone(),
                        range: abs_mid..range.end,
                    },
                ))
            }
            Self::Transformed { content, .. } => content
                .split_at_checked(mid)
                .map(|(a, b)| (self.with_content(a), self.with_content(b))),
        }
    }

    fn take(&self, count: usize) -> Self
    where
        Self: Sized,
    {
        match self {
            Self::Original { source, range } => {
                let end = range.start + count;
                assert!(end <= range.end);

                Self::Original {
                    source: source.clone(),
                    range: range.start..end,
                }
            }
            Self::Transformed { .. } => self.slice(0..count),
        }
    }
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        match self {
            Self::Original { source, range } => &source.content.as_str()[range.clone()],
            Self::Transformed { content, .. } => content.as_ref(),
        }
    }
}

#[derive(Debug)]
pub struct Source {
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
    String(Arc<String>),
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
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Self::String(Arc::new(s))
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

impl TransformContent for Span {
    type Transformed = Self;

    fn to_content(&self, content: String) -> <Self as TransformContent>::Transformed {
        let (line, column) = Span::position(self);
        Span::new_transformed(Span::id(self), line, column, content)
    }

    fn append_content<T: AsRef<str>>(&self, content: T) -> <Self as TransformContent>::Transformed {
        let (line, column) = Span::position(self);
        let mut s = String::with_capacity(self.len() + content.as_ref().len());
        s.push_str(Input::as_str(self));
        s.push_str(content.as_ref());

        Span::new_transformed(self.id(), line, column, s)
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
