/// A typical recoverable error
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NotFound;

impl From<Never> for NotFound {
    fn from(value: Never) -> Self {
        unreachable!()
    }
}
impl From<Incomplete> for NotFound {
    fn from(value: Incomplete) -> Self {
        Default::default()
    }
}

/// A typical incomplete failure
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Incomplete;

impl From<Never> for Incomplete {
    fn from(value: Never) -> Self {
        unreachable!()
    }
}
impl From<NotFound> for Incomplete {
    fn from(value: NotFound) -> Self {
        Default::default()
    }
}

/// An error or failure which is not returned by any code branch. A `From`
/// implementation should consist only of `unreachable!()`.
/// This is typically used as a Failure by parsers which never return irrecoverable
/// errors or incomplete.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Never(Neverever);

/// A type which is not exported and prevents foreign crates from ever creating
/// a `Never` value.
/// NB: This does not prevent a programmer working within this crate from creating
/// a `Never`. This must be enforced via code review.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Neverever;
