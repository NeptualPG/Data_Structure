//! `api_flow` — a tiny, in-memory simulation of a classic data-system
//! architecture:
//!
//! ```text
//! Client -> API / application logic -> cache
//!                                      -> primary database
//!                                      -> full-text index
//!                                      -> message queue -> background worker
//! ```
//!
//! Everything lives in memory and only the Rust standard library is used, so
//! the program runs with `cargo run` and no external services.
//!
//! NOTE ON MODULE NAMES: Rust identifiers cannot contain `-`, so the folders
//! keep their hyphenated names and are attached to underscore module names with
//! `#[path = "..."]`.

// A module's folder name does not have to match its module name. `#[path]`
// points at the file that holds the module, so `in-memory-cache/mod.rs` is
// available as the `in_memory_cache` module.
#[path = "in-memory-cache/mod.rs"]
mod in_memory_cache;
#[path = "primary-database/mod.rs"]
mod primary_database;
#[path = "full-text-index/mod.rs"]
mod full_text_index;
#[path = "message-queue/mod.rs"]
mod message_queue;
mod user;

use full_text_index::FullTextIndex;
use in_memory_cache::InMemoryCache;
use message_queue::{MessageQueue, Task};
use primary_database::{DatabaseError, PrimaryDatabase};
use user::User;

/// The "database change" event.
///
/// In a real system this event is produced by the database itself: a
/// transaction writes to the primary database, and a change-data-capture (CDC)
/// tool publishes a record to a log (Kafka/Debezium) or a table of outbox rows.
/// Separate consumers then update the search index, evict cache entries and
/// trigger downstream jobs.
///
/// Here, for learning, the event is created by hand and handed directly to
/// `process_database_change` — a plain, synchronous function call, because
/// there is no separate process or log to publish to. The *shape* of the
/// system is the same; only the transport is missing.
#[derive(Debug)]
enum DatabaseChange {
    /// A brand new user was inserted.
    Created(User),
    /// An existing user was overwritten. The old name is needed so the search
    /// index can delete the words that no longer apply.
    Updated { old_name: String, new: User },
}

fn main() {
    println!("=== api_flow: client -> API -> cache / database / index / queue ===");

    // Each layer is a separate value, owned by `main`. `&mut` borrows are
    // handed to the functions below just for the duration of the call.
    let mut database = PrimaryDatabase::new();
    let mut cache = InMemoryCache::new();
    let mut search_index = FullTextIndex::new();

    // Start the queue and its worker up front, as a real service would.
    let (queue, worker) = MessageQueue::start();

    // ---------------------------------------------------------------- create
    // The write path is identical for every user, so it lives in one function.
    for (id, name, email) in [
        (1, "Alice Johnson", "alice@example.com"),
        (2, "Bob Smith", "bob@example.com"),
    ] {
        // `create_user` returns a `Result`; the handler reports failures
        // instead of unwrapping them.
        match create_user(&mut database, &mut cache, &mut search_index, &queue, id, name, email) {
            Ok(_) => {}
            Err(error) => println!("  api error: {error}"),
        }
    }

    // ------------------------------------------------------------------ read
    // First read: cache miss -> primary database -> fill the cache.
    match read_user(&database, &mut cache, 1) {
        Ok(user) => println!("  api returned {user}"),
        Err(error) => println!("  api error: {error}"),
    }
    // Second read of the same id: served straight from the cache.
    match read_user(&database, &mut cache, 1) {
        Ok(user) => println!("  api returned {user}"),
        Err(error) => println!("  api error: {error}"),
    }
    // A missing user is an `Err`, not a panic.
    match read_user(&database, &mut cache, 999) {
        Ok(user) => println!("  api returned {user}"),
        Err(error) => println!("  api error: {error} (handled, no crash)"),
    }

    // ---------------------------------------------------------------- update
    match update_user(
        &mut database,
        &mut cache,
        &mut search_index,
        1,
        "Alice Johnson-Smith",
        "alice.johnson@example.com",
    ) {
        Ok(_) => println!("  api confirmed the update"),
        Err(error) => println!("  api error: {error}"),
    }
    // The old cached copy was invalidated, so this read goes back to the
    // database and observes the fresh row.
    match read_user(&database, &mut cache, 1) {
        Ok(user) => println!("  api returned the fresh {user}"),
        Err(error) => println!("  api error: {error}"),
    }

    // ---------------------------------------------------------------- search
    // Case-insensitive, prefix-friendly lookup in the search index.
    search_users(&search_index, &database, "bob");
    search_users(&search_index, &database, "ALICE");
    search_users(&search_index, &database, "nothing-matches-this");

    // ----------------------------------------------------------------- state
    println!("\n=== final state ===");
    println!("  database rows: {}, cache entries: {}, indexed words: {}",
        database.len(), cache.len(), search_index.word_count());

    // ---------------------------------------------------------- background work
    println!("\n=== background tasks ===");
    println!("  the welcome-email tasks queued above are handled by the worker thread");
    println!("  (its output may appear between any two lines above: it is a real thread)");

    // Graceful shutdown:
    // 1. Dropping every `Sender` closes the channel, which is how the worker
    //    learns there is no more work coming.
    drop(queue);
    // 2. `join` waits for the thread to finish and returns a `Result`: `Err`
    //    means the thread panicked. We report it instead of unwrapping.
    match worker.join() {
        Ok(()) => println!("  worker thread joined cleanly"),
        Err(_) => eprintln!("  the worker thread panicked"),
    }
}

/// API handler: create a user.
///
/// The order matters: write to the primary database first (it is the source of
/// truth), then react to that change, then queue the follow-up task.
fn create_user(
    database: &mut PrimaryDatabase,
    cache: &mut InMemoryCache,
    search_index: &mut FullTextIndex,
    queue: &MessageQueue,
    id: u64,
    name: &str,
    email: &str,
) -> Result<User, DatabaseError> {
    println!("\n=== create user #{id} ===");
    let user = User::new(id, name, email);
    println!("  [api]    client asked to create {user}");

    // 1. Primary database: the write goes to the system of record.
    println!("  [api]    -> primary database");
    database.save(&user)?;

    // 2. Simulated "database change" event. In this educational simulation
    //    the handler is a direct function call, not a message from a log.
    println!("  [api]    -> database-change event");
    let change = DatabaseChange::Created(user.clone());
    process_database_change(&change, cache, search_index);

    // 3. Message queue: hand the slow work to the background worker and
    //    return to the client immediately.
    println!("  [api]    -> message queue");
    let task = Task::SendWelcomeEmail {
        to_name: user.name.clone(),
        to_email: user.email.clone(),
    };
    if let Err(error) = queue.send(task) {
        // Not fatal for the user that was created; just report it.
        println!("  [queue]  could not enqueue the welcome email: {error}");
    } else {
        println!("  [queue]  welcome-email task queued");
    }

    println!("  [api]    created {user}");
    Ok(user)
}

/// API handler: read a user, cache first.
///
/// `&database` is an immutable borrow, `&mut cache` is a mutable borrow: reads
/// do not change the database but they may fill the cache.
fn read_user(
    database: &PrimaryDatabase,
    cache: &mut InMemoryCache,
    id: u64,
) -> Result<User, DatabaseError> {
    match cache.get(id) {
        Some(user) => {
            println!("  [api]    read user {id}: cache HIT");
            Ok(user)
        }
        None => {
            println!("  [api]    read user {id}: cache MISS");
            let user = database.find_by_id(id)?; // `?` returns early on Err
            println!("      [db]     read {user} from the primary database");
            // Fill the cache for the next reader. `put` takes ownership, so we
            // give it a copy and keep ours.
            cache.put(user.clone());
            Ok(user)
        }
    }
}

/// API handler: update a user.
fn update_user(
    database: &mut PrimaryDatabase,
    cache: &mut InMemoryCache,
    search_index: &mut FullTextIndex,
    id: u64,
    new_name: &str,
    new_email: &str,
) -> Result<User, DatabaseError> {
    println!("\n=== update user #{id} ===");

    // Read the current row first: the index stores words, so we need the old
    // name to clean them up. `?` stops here if the user does not exist.
    let current = database.find_by_id(id)?;
    let updated = User::new(id, new_name, new_email);
    println!("  [api]    client asked to change {} into {updated}", current);

    println!("  [api]    -> primary database");
    database.update(&updated)?;

    println!("  [api]    -> database-change event");
    let change = DatabaseChange::Updated {
        old_name: current.name.clone(),
        new: updated.clone(),
    };
    process_database_change(&change, cache, search_index);

    println!("  [api]    updated {updated}");
    Ok(updated)
}

/// The handler a change-data-capture consumer would run.
///
/// It keeps the derived stores consistent with the primary database:
/// the search index is refreshed and the cache entry is dropped, so the next
/// read repopulates it from the new row.
fn process_database_change(
    change: &DatabaseChange,
    cache: &mut InMemoryCache,
    search_index: &mut FullTextIndex,
) {
    match change {
        DatabaseChange::Created(user) => {
            // A new user cannot be cached yet, but it must be searchable.
            println!("      [event]  a user row was inserted");
            search_index.index_user(user);
            // Nothing to invalidate, but calling it keeps the flow uniform
            // and is a no-op when the entry is absent.
            cache.invalidate(user.id);
        }
        DatabaseChange::Updated { old_name, new } => {
            println!("      [event]  a user row was updated");
            // Remove the stale words, then add the new ones.
            search_index.remove_user(new.id, old_name);
            search_index.index_user(new);
            // The cached copy is now out of date: drop it.
            cache.invalidate(new.id);
        }
    }
}

/// API handler: search users by name.
///
/// The index returns ids only; the rows are loaded from the primary database,
/// which keeps the layers decoupled.
fn search_users(search_index: &FullTextIndex, database: &PrimaryDatabase, query: &str) {
    println!("\n=== search \"{query}\" ===");
    println!("  [api]    -> full-text index");
    let ids = search_index.search(query);
    if ids.is_empty() {
        println!("  [api]    no users matched");
        return;
    }
    for id in ids {
        // A user can be deleted behind the index's back, so a missing row is
        // reported rather than unwrapped.
        match database.find_by_id(id) {
            Ok(user) => println!("  [api]    match: {user}"),
            Err(error) => println!("  [api]    match {id} could not be loaded: {error}"),
        }
    }
}
