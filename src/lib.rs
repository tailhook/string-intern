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
#[macro_use] extern crate quick_error;
#[macro_use] extern crate lazy_static;
extern crate rustc_serialize;

mod base_type;
mod validator;

pub use base_type::Symbol;
pub use validator::Validator;
