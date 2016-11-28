use std::fmt;
use std::ops::{Deref, Drop};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::marker::PhantomData;
use std::borrow::Borrow;
use std::sync::{Arc, RwLock, Weak};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};

use rustc_serialize::{Decoder, Decodable, Encoder, Encodable};
use {Validator};

lazy_static! {
    static ref ATOMS: RwLock<HashMap<Buf, Weak<Value>>> =
        RwLock::new(HashMap::new());
}

/// Base symbol type
///
/// To use this type you should define your own type of symbol:
///
/// ```ignore
/// type MySymbol = Symbol<MyValidator>;
/// ```
// TODO(tailhook) optimize Eq to compare pointers
pub struct Symbol<V: Validator + ?Sized>(Arc<Value>, PhantomData<V>);

#[derive(PartialEq, Eq, Hash)]
struct Buf(Arc<String>);

#[derive(Clone, PartialEq, Eq, Hash)]
struct Value(Arc<String>);

impl<V: Validator + ?Sized> Clone for Symbol<V> {
    fn clone(&self) -> Symbol<V> {
        Symbol(self.0.clone(), PhantomData)
    }
}

impl<V: Validator + ?Sized> PartialEq for Symbol<V> {
    fn eq(&self, other: &Symbol<V>) -> bool {
        self.0.eq(&other.0)
    }
}
impl<V: Validator + ?Sized> Eq for Symbol<V> {}

impl<V: Validator + ?Sized> Hash for Symbol<V> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher)
    }
}


impl<V: Validator + ?Sized> FromStr for Symbol<V> {
    type Err = V::Err;
    fn from_str(s: &str) -> Result<Symbol<V>, Self::Err> {
        V::validate_symbol(s)?;
        if let Some(a) = ATOMS.read().expect("atoms locked").get(s) {
            return Ok(Symbol(a.upgrade().unwrap().clone(), PhantomData));
        }
        let buf = Arc::new(String::from(s));
        let mut atoms = ATOMS.write().expect("atoms locked");
        let val = match atoms.entry(Buf(buf.clone())) {
            Occupied(e) => e.get().upgrade().unwrap(),
            Vacant(e) => {
                let result = Arc::new(Value(buf));
                e.insert(Arc::downgrade(&result));
                result
            }
        };
        Ok(Symbol(val, PhantomData))
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        let mut atoms = ATOMS.write().expect("atoms locked");
        atoms.remove(&self.0[..]);
    }
}

impl<V: Validator + ?Sized> AsRef<str> for Symbol<V> {
    fn as_ref(&self) -> &str {
        &(self.0).0[..]
    }
}

impl<V: Validator + ?Sized> Borrow<str> for Symbol<V> {
    fn borrow(&self) -> &str {
        &(self.0).0[..]
    }
}

impl<V: Validator + ?Sized> Borrow<String> for Symbol<V> {
    fn borrow(&self) -> &String {
        &(self.0).0
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
        (self.0).0.fmt(fmt)
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

impl<V: Validator> Encodable for Symbol<V> {
    fn encode<E: Encoder>(&self, d: &mut E) -> Result<(), E::Error> {
        d.emit_str(&(self.0).0)
    }
}

impl<V: Validator + ?Sized> Deref for Symbol<V> {
    type Target = str;
    fn deref(&self) -> &str {
        &(self.0).0
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
        .expect("static string used as atom is invalid")
    }
}

#[cfg(test)]
mod test {
    use std::io;
    use rustc_serialize::json;
    use {Validator, Symbol};

    #[allow(dead_code)]
    struct AnyString;
    #[allow(dead_code)]
    struct AlphaNumString;
    type Atom = Symbol<AnyString>;
    type AlphaNum = Symbol<AlphaNumString>;

    impl Validator for AnyString {
        // Use an error from standard library to make example shorter
        type Err = ::std::string::ParseError;
        fn validate_symbol(_: &str) -> Result<(), Self::Err> {
            Ok(())
        }
    }

    impl Validator for AlphaNumString {
        // Use an error from standard library to make example shorter
        type Err = io::Error;
        fn validate_symbol(s: &str) -> Result<(), Self::Err> {
            if s.chars().any(|c| !c.is_alphanumeric()) {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    "Character is not alphanumeric"));
            }
            Ok(())
        }
    }

    #[test]
    fn eq() {
        assert_eq!(Atom::from("x"), Atom::from("x"));
    }

    #[test]
    fn clone() {
        assert_eq!(Atom::from("x").clone(), Atom::from("x"));
    }

    #[test]
    fn hash() {
        use std::collections::HashMap;
        let mut h = HashMap::new();
        h.insert(Atom::from("x"), 123);
        assert_eq!(h.get("x"), Some(&123));
        assert_eq!(h.get(&Atom::from("x")), Some(&123));
        assert_eq!(h.get("y"), None);
        assert_eq!(h.get(&Atom::from("y")), None);
    }

    #[test]
    fn encode() {
        assert_eq!(json::encode(&Atom::from("xyz")).unwrap(),
                   r#""xyz""#);
    }
    #[test]
    fn decode() {
        assert_eq!(json::decode::<Atom>(r#""xyz""#).unwrap(),
                   Atom::from("xyz"));
    }

    #[test]
    #[should_panic(message="static strings used as atom is invalid")]
    fn distinct_validators() {
        let _xa = Atom::from("x");
        let _xn = AlphaNum::from("x");
        let _ya = Atom::from("a-b");
        // This should fail on invalid value, but didn't fail in <= v0.1.2
        let _yn = AlphaNum::from("a-b");
    }
}
