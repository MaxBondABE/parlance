// Concrete types
pub use crate::parse::{Never, NotFound, ParserError, ParserResult};

// Implementable traits
pub use crate::input::Input;
pub use crate::parse::Parser;

// Automatic traits - these occupy common names, so bind anonymously to
// avoid polluting namespace.
pub use crate::{
    parse::{Choice as _, Compose as _, FuseSequence as _, Sequence as _},
    util::rotate::{Rotate as _, RotateFn as _},
};
