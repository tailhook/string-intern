//! String interning for rust
//!
//! # Example
//!
//! ```
//! use string_intern::{Validator, Symbol};
//!
//! struct UserIdSymbol;
//!
//! // This symbol may contain anything
//! impl Validator for UserIdSymbol {
//!     // Use an error from standard library to make example shorter
//!     type Err = ::std::string::ParseError;
//!     fn validate_symbol(val: &str) -> Result<(), Self::Err> {
//!         Ok(())
//!     }
//! }
//!
//! /// Actual symbol type you will use in code
//! type UserId = Symbol<UserIdSymbol>;
//!
//! // Create from const (panics on invalid input)
//! let x = UserId::from("user1");
//! // Create from user input
//! let y: UserId = format!("user{}", 1).parse().unwrap();
//! // Both point to the same bytes
//! assert!(x[..].as_bytes() as *const _ == y[..].as_bytes() as *const _);
//! ```
#[macro_use] extern crate lazy_static;
#[cfg(feature = "rustc-serialize")] extern crate rustc_serialize;
#[cfg(feature = "serde")] extern crate serde;
#[cfg(test)] extern crate serde_json;

mod base_type;
mod validator;

pub use base_type::Symbol;
pub use validator::Validator;

#[cfg(test)]
mod test {
    use super::{Validator, Symbol};

    struct AnyString;

    impl Validator for AnyString {
        // Use an error from standard library to make example shorter
        type Err = ::std::string::ParseError;
        fn validate_symbol(_: &str) -> Result<(), Self::Err> {
            Ok(())
        }
    }

    #[test]
    fn test_sync() {
        fn sync<T: Sync>(_: T) { }
        sync(Symbol::<AnyString>::from("x"))
    }

    #[test]
    fn test_send() {
        fn send<T: Send>(_: T) { }
        send(Symbol::<AnyString>::from("x"))
    }
}
