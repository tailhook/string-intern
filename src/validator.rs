use std::fmt;
use std::error::Error;

use Symbol;


/// This is validator trait you should implement for your own symbols
///
/// In reality this trait serves three purposes:
///
/// 1. Validates that atom contains only value you expect it to contain
/// 2. Identifies the type i.e. `type S1 = Symbol<V1>` and
///    `type S2 = Symbol<V2>` are different and incompatible types
/// 3. Allows to override `Display` trait for your own symbol
pub trait Validator {
    type Err: Error;
    fn validate_symbol(&str) -> Result<(), Self::Err>;
    fn display(value: &Symbol<Self>, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "i{:?}", value.as_ref())
    }
}
