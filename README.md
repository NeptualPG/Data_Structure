# Foundations of Data Systems — Learning Repository

This repository accompanies a hands-on study of *Designing Data-Intensive
Applications* (DDIA) and related distributed-systems concepts. The centerpiece
is **`api_flow`**, a minimal, single-binary Rust program that simulates the
classic data-system architecture:

```
Client → API / Application Logic
              │
              ├──→ Primary Database (source of truth)
              ├──→ Cache (fast reads)
              ├──→ Full-Text Search Index
              └──→ Message Queue → Background Worker
```

All components are **in-memory**, use **only the Rust standard library**, and
run with `cargo run` — no Docker, no external services.

---

## Quick Start

```bash
# From the workspace root
cd part1/api_flow
cargo run
```

Expected output (abridged):

```
=== api_flow: client -> API -> cache / database / index / queue ===

=== create user #1 ===
  [api]    client asked to create #1 Alice Johnson <alice@example.com>
  [api]    -> primary database
      [db]     stored #1 Alice Johnson <alice@example.com> in the primary database
  [api]    -> database-change event
      [event]  a user row was inserted
      [index]  indexed the name of #1 Alice Johnson <alice@example.com>
  [api]    -> message queue
  [queue]  welcome-email task queued
  [api]    created #1 Alice Johnson <alice@example.com>

=== create user #2 ===
  ...

  [api]    read user 1: cache MISS
      [db]     read #1 Alice Johnson <alice@example.com> from the primary database
      [cache]  cached #1 Alice Johnson <alice@example.com>
  api returned #1 Alice Johnson <alice@example.com>
  [api]    read user 1: cache HIT
  api returned #1 Alice Johnson <alice@example.com>
  [api]    read user 999: cache MISS
  api error: user 999 does not exist (handled, no crash)

=== update user #1 ===
  [api]    client asked to change #1 Alice Johnson <alice@example.com> into #1 Alice Johnson-Smith <alice.johnson@example.com>
  [api]    -> primary database
      [db]     updated row for #1 Alice Johnson-Smith <alice.johnson@example.com>
  [api]    -> database-change event
      [event]  a user row was updated
      [index]  removed user 1 from the index
      [index]  indexed the name of #1 Alice Johnson-Smith <alice.johnson@example.com>
      [cache]  invalidated the cached copy of user 1
  [api]    updated #1 Alice Johnson-Smith <alice.johnson@example.com>
  api confirmed the update
  [api]    read user 1: cache MISS
      [db]     read #1 Alice Johnson-Smith <alice.johnson@example.com> from the primary database
      [cache]  cached #1 Alice Johnson-Smith <alice.johnson@example.com>
  api returned the fresh #1 Alice Johnson-Smith <alice.johnson@example.com>

=== search "bob" ===
  [api]    -> full-text index
  [api]    match: #2 Bob Smith <bob@example.com>

=== search "ALICE" ===
  [api]    -> full-text index
  [api]    match: #1 Alice Johnson-Smith <alice.johnson@example.com>

=== search "nothing-matches-this" ===
  [api]    -> full-text index
  [api]    no users matched

=== final state ===
  database rows: 2, cache entries: 1, indexed words: 4

=== background tasks ===
  the welcome-email tasks queued above are handled by the worker thread
  (its output may appear between any two lines above: it is a real thread)
      [worker] welcome email sent to Alice Johnson <alice@example.com>
      [worker] welcome email sent to Bob Smith <bob@example.com>
      [queue]  channel closed, worker thread stopping
  worker thread joined cleanly
```

---

## Project Structure

```
part1/api_flow/
├── Cargo.toml                 # name = "api_flow", edition = "2024", no deps
└── src/
    ├── main.rs                # API handlers + coordination + demo flows
    ├── user/mod.rs            # User { id, name, email }
    ├── primary-database/      # HashMap<u64, User> — system of record
    ├── in-memory-cache/       # HashMap<u64, User> — look-aside cache
    ├── full-text-index/       # Inverted index (word → user ids)
    └── message-queue/         # mpsc channel + worker thread
```

Module folders keep hyphens (`in-memory-cache`) but are declared with
`#[path = "in-memory-cache/mod.rs"] mod in_memory_cache;` so Rust identifiers
use underscores.

---

## Demonstrated Flows

| Flow | What happens | Key code |
|---|---|---|
| **Create user** | `DB.save` → `DatabaseChange::Created` → `index_user` + `cache.invalidate` → `queue.send(SendWelcomeEmail)` | `create_user` |
| **Read user** | `cache.get` → hit: return; miss: `DB.find_by_id` → `cache.put` | `read_user` |
| **Update user** | `DB.find_by_id` (old name) → `DB.update` → `DatabaseChange::Updated` → `remove_user` + `index_user` + `invalidate` | `update_user` |
| **Search** | `index.search(query)` → ids → `DB.find_by_id` for each | `search_users` |
| **Background task** | Worker thread consumes `mpsc` channel; `drop(sender)` closes channel → graceful `join()` | `MessageQueue::start`, `main` shutdown |

---

## Learning Documents

| File | Purpose |
|---|---|
| **[CONCEPTS.md](CONCEPTS.md)** | Maps each component to DDIA concepts; 8-step path to swap in-memory pieces for real infrastructure (SQLite → Redis → Tantivy → Kafka → CDC → Observability → Async → Docker). |
| **[GLOSSARY.md](GLOSSARY.md)** | 80+ alphabetical terms with one-line definitions, project links, and "dig deeper" pointers. |

---

## Rust Concepts Illustrated

| Feature | Where |
|---|---|
| Ownership / `move` | `cache.put(user)`, `queue.send(task)` |
| Borrowing (`&`, `&mut`) | `database.find_by_id(&self, id)`, `cache.get(&self, id)` |
| `Option<T>` | `cache.get` returns `Option<User>` — explicit miss handling |
| `Result<T, E>` | All fallible ops; `?` propagates cleanly |
| `mpsc` channel | Thread-safe ownership transfer between API and worker |
| `thread::spawn` + `JoinHandle` | Structured concurrency; main waits for worker |

---

## Next Steps — Replace One Piece at a Time

1. **Persist the DB** — `cargo add rusqlite` (or `sqlx` + Postgres)
2. **Real cache** — `cargo add redis` + TTL
3. **Real search** — `cargo add tantivy` (pure-Rust) or Meilisearch
4. **Real queue** — `cargo add lapin` (RabbitMQ) or `rdkafka`
5. **CDC** — Transactional outbox + Debezium/Kafka Connect
6. **Observability** — `tracing` + `prometheus` + OpenTelemetry
7. **Async** — `tokio`, make handlers `async`, run on thread pool
8. **Deploy** — Dockerfile + docker-compose for all services

Each step keeps the **architecture** (the arrows) identical; only the
implementation of each box changes.

---

## Requirements

- Rust 1.70+ (`cargo --version` to check)
- No external services needed

---

## License

MIT — use freely for learning, teaching, or as a starting point.