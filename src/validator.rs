use std::error::Error;


pub trait Validator {
    type Err: Error;
    fn validate_symbol(&str) -> Result<(), Self::Err>;
}
