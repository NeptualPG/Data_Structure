//! The `in-memory-cache` module simulates a cache such as Redis or Memcached.
//!
//! The cache is a *separate* `HashMap` from the primary database. Keeping the
//! two apart is the whole point: the cache is a fast, disposable copy that can
//! be dropped and refilled at any time without losing data.

use crate::user::User;
use std::collections::HashMap;

/// A very small cache: user id -> cached copy of the user.
pub struct InMemoryCache {
    entries: HashMap<u64, User>,
}

impl InMemoryCache {
    /// An empty cache. Every lookup starts as a miss.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Look a user up in the cache.
    ///
    /// Returns `Option<User>`: `Some(user)` on a hit, `None` on a miss. An
    /// `Option` is the idiomatic way to say "there might be nothing here".
    pub fn get(&self, id: u64) -> Option<User> {
        // The map is only borrowed (`&self`), so we clone the stored user to
        // return an owned value.
        match self.entries.get(&id) {
            Some(user) => Some(user.clone()),
            None => None,
        }
    }

    /// Add (or refresh) an entry.
    ///
    /// The user is *moved* into the cache, so no copy is made.
    pub fn put(&mut self, user: User) {
        println!("      [cache]  cached {}", user);
        self.entries.insert(user.id, user);
    }

    /// Drop one entry. Returns `true` if something was actually removed.
    ///
    /// This is what the "database change" handler calls after an update,
    /// because the cached copy would otherwise be stale.
    pub fn invalidate(&mut self, id: u64) -> bool {
        let removed = self.entries.remove(&id).is_some();
        if removed {
            println!("      [cache]  invalidated the cached copy of user {id}");
        }
        removed
    }

    /// Number of entries, used by the demo output.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}
