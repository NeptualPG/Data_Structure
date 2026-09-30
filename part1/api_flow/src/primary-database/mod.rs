//! The `primary-database` module simulates the system of record.
//!
//! In production this would be Postgres, MySQL, etcd... Here it is nothing
//! more than a `HashMap` keyed by user id, which is enough to show *where*
//! the data lives and *who* owns it.

use crate::user::User;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Everything that can go wrong when we touch the database.
///
/// A `Result<T, E>` is either `Ok(value)` or `Err(error)`. Using an error
/// enum instead of panicking means a missing row is a normal, expected
/// outcome that the caller can handle.
#[derive(Debug)]
pub enum DatabaseError {
    /// Somebody tried to insert a user with an id that already exists.
    DuplicateId(u64),
    /// No row exists for the requested id.
    NotFound(u64),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::DuplicateId(id) => write!(f, "user {id} already exists"),
            DatabaseError::NotFound(id) => write!(f, "user {id} does not exist"),
        }
    }
}

// Implementing `Error` lets this type be used with the standard error
// reporting helpers (`Box<dyn Error>`, `?` in `main`, ...).
impl Error for DatabaseError {}

/// The simulated primary database.
///
/// The `HashMap` field is *private* to this module. That is deliberate: no
/// other module may read the map directly, so every access goes through the
/// methods below, exactly like a real database would force you to.
pub struct PrimaryDatabase {
    users: HashMap<u64, User>,
}

impl PrimaryDatabase {
    /// An empty database.
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Insert a new user.
    ///
    /// The user is *borrowed* (`&User`) and cloned into the map, so the
    /// caller keeps its own copy and can keep using it afterwards.
    pub fn save(&mut self, user: &User) -> Result<(), DatabaseError> {
        if self.users.contains_key(&user.id) {
            return Err(DatabaseError::DuplicateId(user.id));
        }

        // `insert` overwrites any previous value for the key; because we just
        // checked that the key is free, this is a pure insert.
        self.users.insert(user.id, user.clone());
        println!("      [db]     stored {} in the primary database", user);
        Ok(())
    }

    /// Overwrite an existing user.
    pub fn update(&mut self, user: &User) -> Result<(), DatabaseError> {
        if !self.users.contains_key(&user.id) {
            return Err(DatabaseError::NotFound(user.id));
        }

        self.users.insert(user.id, user.clone());
        println!("      [db]     updated row for {}", user);
        Ok(())
    }

    /// Read one user by id.
    ///
    /// `HashMap::get` borrows the stored `User` immutably, so we clone it to
    /// hand an owned value back to the caller. (`?` propagates the error, so
    /// the caller receives `Result<User, DatabaseError>`.)
    pub fn find_by_id(&self, id: u64) -> Result<User, DatabaseError> {
        match self.users.get(&id) {
            Some(user) => Ok(user.clone()),
            None => Err(DatabaseError::NotFound(id)),
        }
    }

    /// How many rows are stored. Used by the demo output.
    pub fn len(&self) -> usize {
        self.users.len()
    }
}

// `Default` lets callers write `PrimaryDatabase::default()` too.
impl Default for PrimaryDatabase {
    fn default() -> Self {
        Self::new()
    }
}
