//! The `user` module holds the one piece of data this project stores.
//!
//! In a real system this type would live in a shared crate that the API,
//! the workers and the storage layers all depend on. Here it is deliberately
//! tiny so the focus stays on how the layers talk to each other.

// `std::fmt` is the standard formatting module. We implement `fmt::Display`
// for `User` so `println!("{}", user)` works.
use std::fmt;

/// A single user record.
///
/// Every field is public so the other modules can read them directly. Each
/// field *owns* its data: `String` is a growable, owned piece of text, so the
/// `User` stays valid even after the value that was used to build it is gone.
/// That is why `User` has no lifetime parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    /// Build a user from borrowed strings.
    ///
    /// `name: &str` is a *borrowed* view of text owned by the caller. Calling
    /// `to_string()` copies it into an owned `String` that lives inside the
    /// `User`, so nothing has to stay borrowed.
    pub fn new(id: u64, name: &str, email: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            email: email.to_string(),
        }
    }
}

/// Lets a `User` be printed with `{}`.
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `write!` returns a `fmt::Result` (a `Result` of `Ok`/`Err`), so we
        // forward it with `?` instead of unwrapping it.
        write!(f, "#{} {} <{}>", self.id, self.name, self.email)
    }
}
