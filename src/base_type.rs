use std::fmt;
use std::ops::Deref;
use std::str::FromStr;
use std::marker::PhantomData;
use std::borrow::Borrow;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

use rustc_serialize::{Decoder, Decodable};
use {Validator};

lazy_static! {
    static ref ATOMS: RwLock<HashSet<Buf>> = RwLock::new(HashSet::new());
}

/// Base symbol type
///
/// To use this type you should define your own type of symbol:
///
/// ```ignore
/// type MySymbol = Symbol<MyValidator>;
/// ```
// TODO(tailhook) optimize Eq to compare pointers
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Symbol<V: Validator + ?Sized>(Arc<String>, PhantomData<*const V>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Buf(Arc<String>);


impl<V: Validator + ?Sized> FromStr for Symbol<V> {
    type Err = V::Err;
    fn from_str(s: &str) -> Result<Symbol<V>, Self::Err> {
        if let Some(a) = ATOMS.read().expect("atoms locked").get(s) {
            return Ok(Symbol(a.0.clone(), PhantomData));
        }
        V::validate_symbol(s)?;
        let newsymbol = Arc::new(String::from(s));
        let mut atoms = ATOMS.write().expect("atoms locked");
        if !atoms.insert(Buf(newsymbol.clone())) {
            // Race condition happened, but now we are still holding lock
            // so it's safe to unwrap
            return Ok(Symbol(atoms.get(s).unwrap().0.clone(), PhantomData));
        } else {
            return Ok(Symbol(newsymbol, PhantomData));
        }
    }
}

impl<V: Validator + ?Sized> AsRef<str> for Symbol<V> {
    fn as_ref(&self) -> &str {
        &self.0[..]
    }
}

impl<V: Validator + ?Sized> Borrow<str> for Symbol<V> {
    fn borrow(&self) -> &str {
        &self.0[..]
    }
}

impl<V: Validator + ?Sized> Borrow<String> for Symbol<V> {
    fn borrow(&self) -> &String {
        &self.0
    }
}

impl Borrow<str> for Buf {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Borrow<String> for Buf {
    fn borrow(&self) -> &String {
        &self.0
    }
}

impl<V: Validator + ?Sized> fmt::Debug for Symbol<V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        V::display(self, fmt)
    }
}

impl<V: Validator + ?Sized> fmt::Display for Symbol<V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(fmt)
    }
}

impl<V: Validator> Decodable for Symbol<V> {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        use std::error::Error;
        d.read_str()?
        .parse::<Symbol<V>>()
        .map_err(|e| d.error(e.description()))
    }
}

impl<V: Validator + ?Sized> Deref for Symbol<V> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl<V: Validator + ?Sized> Symbol<V> {
    /// Create a symbol from a static string
    ///
    /// # Panics
    ///
    /// When symbol is of invalid format. We assume that this is used for
    /// constant strings in source code, so we assert that they are valid.
    ///
    /// Use `FromStr::from_str(x)` or `x.parse()` to parse user input
    pub fn from(s: &'static str) -> Symbol<V> {
        FromStr::from_str(s)
        .expect("static strings used as atom is invalid")
    }
}
