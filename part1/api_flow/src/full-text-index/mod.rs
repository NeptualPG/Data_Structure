//! The `full-text-index` module simulates a search engine such as
//! Elasticsearch, OpenSearch or a SQL `FULLTEXT` index.
//!
//! ---------------------------------------------------------------------------
//! IMPORTANT: this is a *simplified* search implementation, written to show
//! the shape of the component, not to be fast.
//!
//! What it really is: a tiny inverted index. Every word of every indexed name
//! is lowercased and stored in a `HashMap`, mapping
//! `"johnson" -> [1]` (word -> ids of matching users). A query is lowercased
//! the same way and looked up in that map, so `"ALICE"` finds `"Alice"`.
//!
//! What a real system does instead: the database or the search cluster
//! maintains that map itself, over *all* of the document's fields and body
//! text, with tokenisation rules, stemming, ranking and relevance scores. It
//! also runs out-of-process, so a write to the database does not rebuild the
//! index in the same thread (see `DatabaseChange` in `main.rs`).
//!
//! Here, the "engine" is an in-process `HashMap` updated explicitly after each
//! database change, and a query is a lookup plus a prefix scan over the
//! indexed words.
//! ---------------------------------------------------------------------------

use crate::user::User;
use std::collections::HashMap;

/// A tiny inverted index: lowercase word -> ids of the users containing it.
pub struct FullTextIndex {
    word_to_user_ids: HashMap<String, Vec<u64>>,
}

impl FullTextIndex {
    /// An index with nothing in it yet.
    pub fn new() -> Self {
        Self {
            word_to_user_ids: HashMap::new(),
        }
    }

    /// Add (or refresh) a user in the index.
    ///
    /// The user is only *borrowed*: the index copies the words it needs and
    /// never stores the `User` itself, so the caller's copy stays independent.
    pub fn index_user(&mut self, user: &User) {
        for word in words_of(&user.name) {
            let ids = self
                .word_to_user_ids
                .entry(word)
                .or_insert_with(Vec::new);
            if !ids.contains(&user.id) {
                ids.push(user.id);
            }
        }
        println!("      [index]  indexed the name of {}", user);
    }

    /// Remove a user from the index.
    ///
    /// The old name is needed because the index stores words, not user
    /// records: without it we would not know which words to clean up.
    pub fn remove_user(&mut self, id: u64, old_name: &str) {
        for word in words_of(old_name) {
            // Step 1: drop the id from that word's list.
            let mut is_empty = false;
            if let Some(ids) = self.word_to_user_ids.get_mut(&word) {
                ids.retain(|existing| *existing != id);
                is_empty = ids.is_empty();
            }
            // Step 2: drop the word itself once nothing points at it. This has
            // to happen after the mutable borrow above has ended, otherwise we
            // would hold two borrows of the map at the same time.
            if is_empty {
                self.word_to_user_ids.remove(&word);
            }
        }
        println!("      [index]  removed user {id} from the index");
    }

    /// Search users whose name contains `query`, ignoring case.
    ///
    /// Returns the matching user ids. The caller decides what to do with them
    /// (here: load the rows from the database), which keeps the index from
    /// having to know anything about the database.
    pub fn search(&self, query: &str) -> Vec<u64> {
        // `trim` + `to_lowercase` normalise the query the same way indexing
        // normalises the documents, so matching works regardless of case.
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            // Early return: an empty query matches nothing here.
            return Vec::new();
        }

        // SIMPLIFIED: we scan every indexed word looking for a prefix match.
        // A real engine would jump straight to the word in a trie or hash map.
        let mut matches: Vec<u64> = Vec::new();
        for (word, ids) in &self.word_to_user_ids {
            if word.starts_with(&needle) {
                for id in ids {
                    if !matches.contains(id) {
                        matches.push(*id);
                    }
                }
            }
        }
        // Deterministic output order for the demo.
        matches.sort_unstable();
        matches
    }

    /// How many distinct words are indexed, used by the demo output.
    pub fn word_count(&self) -> usize {
        self.word_to_user_ids.len()
    }
}

impl Default for FullTextIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Split a name into lowercased words.
///
/// Splitting on "not alphanumeric" means punctuation and extra spaces are
/// treated as separators, so `"Anne-Marie O'Neil"` becomes
/// `["anne", "marie", "o", "neil"]`.
fn words_of(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_lowercase())
        .collect()
}
