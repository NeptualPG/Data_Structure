# Data Systems Concepts — Learning Notes

This document ties together the architecture demonstrated by **api_flow** with the
core ideas from *Designing Data-Intensive Applications* (DDIA) and general
distributed-systems thinking. Each section maps a project component to the
concept it illustrates, then suggests what to explore next.

---

## 1. System of Record — Primary Database

**In the project:** `primary_database::PrimaryDatabase` — a private `HashMap`
that only the API layer touches.

**Concept:** The *source of truth*. All other stores (cache, index, queue) are
derived from it. In production this is Postgres, MySQL, Cassandra, etc.
Durability, transactions, and replication live here.

**Key properties**
| Property | What it means | Where to learn more |
|---|---|---|
| **ACID** | Atomicity, Consistency, Isolation, Durability — the classic guarantees. | DDIA ch. 2, 7 |
| **Write path** | Client → API → primary DB → (async) derived stores. | This project's `create_user` / `update_user` |
| **Read path** | Client → API → cache → (on miss) primary DB. | This project's `read_user` |
| **Schema** | Even schemaless DBs have an implicit schema; migrations matter. | DDIA ch. 2 |

**Exercises**
- Swap `HashMap` for `sled` or `rusqlite` to get persistence + ACID.
- Add a transaction that inserts a user *and* an audit row atomically.

---

## 2. Cache — Speeding Up Reads

**In the project:** `in_memory_cache::InMemoryCache` — a second `HashMap`
kept in sync by explicit invalidation.

**Concept:** A fast, disposable copy of hot data. The cache *does not* own the
data; the primary DB does. Cache misses fall back to the DB and repopulate.

**Patterns**
| Pattern | How it works | Trade-off |
|---|---|---|
| **Look-aside** (this project) | On miss, app reads DB then writes cache. Simple; extra round-trip on miss. | Miss latency = DB latency |
| **Write-through** | App writes cache *and* DB together. No miss on subsequent reads. | Write latency = max(cache, DB) |
| **Write-behind** | App writes cache; background flushes to DB. Fast writes; risk of loss. | Durability vs. speed |
| **Refresh-ahead** | Background job pre-warms cache before TTL expires. | Complexity; wasted work |

**Invalidation strategies**
- **Explicit** (this project): `cache.invalidate(id)` on every DB write.
- **TTL**: entries auto-expire; simple but serves stale data until expiry.
- **Versioned keys**: key includes `vN`; writers bump version, old keys
  become orphaned and are evicted later.
- **CDC / log tailing**: a separate process reads the DB log and invalidates.

**Exercises**
- Add a TTL field and a background sweeper thread.
- Instrument hit/miss counters and print a ratio at shutdown.

---

## 3. Search Index — Full-Text Search

**In the project:** `full_text_index::FullTextIndex` — a tiny inverted index
(word → user ids) with case-insensitive prefix matching.

**Concept:** A data structure optimised for *lookup by term*, not by primary
key. Real engines (Elasticsearch, OpenSearch, Tantivy, Meilisearch, PostgreSQL
`tsvector`) build this *asynchronously* from the primary DB.

**What a real engine adds**
| Feature | Why it matters |
|---|---|
| **Tokenisation** | Splits text into terms (stemming, synonyms, n-grams, CJK). |
| **Ranking / scoring** | TF-IDF, BM25, learning-to-rank — not just "matches". |
| **Distributed sharding** | Index splits across nodes; queries fan out and merge. |
| **Near-real-time** | Changes visible in ~1 s, not after batch rebuild. |
| **Facets / filters** | Combine full-text with structured filters (price < 100, category = X). |

**The project's simplification**
- Index update is a *synchronous function call* (`process_database_change`).
- In reality: DB → WAL → CDC (Debezium) → Kafka → indexer consumer → ES bulk API.
- The index lives in a *separate process/cluster*, so a DB crash doesn't
  corrupt the index and vice-versa.

**Exercises**
- Replace the `HashMap` with `tantivy` (pure-Rust search library).
- Add a second field (`email`) and a combined query.
- Simulate CDC: write changes to a `Vec<DatabaseChange>` "log" and have a
  separate thread consume it and update the index.

---

## 4. Message Queue — Async Work & Decoupling

**In the project:** `message_queue::MessageQueue` — `std::sync::mpsc` channel
+ a worker thread that sleeps 150 ms per email.

**Concept:** Decouple *latency-sensitive* request paths from *throughput-*
or *reliability-sensitive* background work. The API returns immediately; the
worker processes at its own pace.

**Queue semantics**
| Semantic | Description | Typical use |
|---|---|---|
| **At-least-once** | Message redelivered on ack failure; idempotent consumers needed. | Email, webhooks |
| **At-most-once** | Fire-and-forget; okay to lose. | Metrics, logs |
| **Exactly-once** | Hard; requires transactional outbox + idempotent consumer. | Payments, inventory |

**What the project shows**
- `mpsc` is *in-process*; real queues (Kafka, RabbitMQ, SQS, Redis Streams)
  are network services with persistence, replication, and consumer groups.
- Graceful shutdown: drop sender → channel closed → worker `recv` returns
  `Err` → loop exits → `join`.

**Exercises**
- Add a `Retry` task variant with exponential back-off.
- Replace `mpsc` with `crossbeam-channel` (bounded, multi-consumer).
- Persist tasks to a file (WAL) so a crash doesn't lose queued emails.
- Add a second worker and observe round-robin consumption.

---

## 5. Change Data Capture (CDC) — Keeping Derived Stores in Sync

**In the project:** `DatabaseChange` enum + `process_database_change` function.

**Concept:** Capture every write to the primary DB and propagate it to
caches, indexes, analytics, etc. — *without* coupling the writer to the
consumers.

**Implementation approaches**
| Approach | Latency | Complexity | Notes |
|---|---|---|---|
| **Dual write** (app writes DB + cache + index) | Low | Low | Fragile; partial failures leave inconsistency. |
| **Transactional outbox** | Low | Medium | Write events to an `outbox` table in same TX; poller publishes. |
| **Log-based CDC** (Debezium, pgoutput) | ~ms | Higher | Reads DB replication log; no app changes. Gold standard. |
| **Event sourcing** | Low | High | Events *are* the source of truth; state is a projection. |

**The project's `DatabaseChange`** is what a CDC system would emit.
`Created` / `Updated` carry enough information for the index to delete old
words and insert new ones, and for the cache to evict.

**Exercises**
- Implement a transactional outbox: `database.save(user); outbox.insert(event)`
  in one transaction (needs a real DB).
- Write a tiny poller that reads `outbox`, publishes to `mpsc`, marks `published`.
- Simulate a crash between DB write and outbox insert; observe lost events.

---

## 6. Concurrency & Ownership — Rust-Specific Lessons

| Rust feature | Where it appears | Why it matters for data systems |
|---|---|---|
| **Ownership / `move`** | `cache.put(user)` takes ownership; `queue.send(task)` moves task into channel. | Guarantees no use-after-free across threads. |
| **Borrowing (`&`, `&mut`)** | `database.find_by_id(&self, id)`, `cache.get(&self, id)`. | Multiple readers OR one writer — prevents data races at compile time. |
| **`Option<T>`** | `cache.get` returns `Option<User>` — explicit "maybe nothing". | Forces handling of cache miss; no null pointers. |
| **`Result<T, E>`** | All fallible operations (`save`, `update`, `find_by_id`, `queue.send`). | Errors are values; `?` propagates cleanly, no exceptions. |
| **`mpsc` channel** | `MessageQueue::start` splits `(Sender, Receiver)` across threads. | Thread-safe communication without mutexes; ownership transfer. |
| **`thread::spawn` + `JoinHandle`** | Worker thread; `join()` in `main`. | Structured concurrency — main waits for worker, catches panics. |
| **`Clone` on `User`** | Cheap because `String` is reference-counted internally (heap). | Needed because `HashMap::get` returns `&User` but API returns owned `User`. |

**Exercises**
- Remove `Clone` from `User` and use `Arc<User>` everywhere; compare ergonomics.
- Replace `mpsc` with `tokio::sync::mpsc` + `async/await`; measure throughput.
- Add a `RwLock<HashMap>` cache and allow concurrent readers.

---

## 7. Observability — What You Can't See, You Can't Operate

**In the project:** `println!` lines prefixed with `[api]`, `[db]`, `[cache]`,
`[index]`, `[queue]`, `[worker]`, `[event]`.

**Real systems need**
| Signal | Tooling | Example |
|---|---|---|
| **Logs** | Structured JSON (tracing, slog) + Loki / Elasticsearch | `"level":"info", "component":"cache", "op":"hit", "user_id":1` |
| **Metrics** | Prometheus + Grafana | `cache_hits_total`, `db_latency_seconds_bucket` |
| **Traces** | OpenTelemetry + Jaeger / Tempo | One HTTP request → spans for cache, DB, index, queue |
| **Alerts** | Prometheus rules / PagerDuty | `cache_hit_ratio < 0.8 for 5m` |

**Exercises**
- Add `tracing` crate; emit `span!` for each API call.
- Export a `/metrics` endpoint with `prometheus` crate.
- Correlate a trace: one create-user request = 5 spans (API, DB, index, cache, queue).

---

## 8. Data Modelling — The `User` Type

```rust
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}
```

**Decisions implicit here**
- `u64` ID: assumes single-leader or UUIDv7; not a distributed ID (Snowflake, ULID).
- `String` for name/email: no validation, no normalisation, no separate
  `Email` newtype.
- No version / `updated_at` — optimistic locking would need it.
- No soft-delete flag — `DatabaseError::NotFound` is the only deletion signal.

**Exercises**
- Add `version: u64` and make `update` fail on mismatch (optimistic locking).
- Make `Email` a newtype with `FromStr` validation.
- Add `deleted_at: Option<DateTime<Utc>>` and filter reads.

---

## 9. Testing Strategies for Data Systems

| Layer | Technique |
|---|---|
| **Unit** | Test `InMemoryCache`, `FullTextIndex`, `PrimaryDatabase` in isolation with property-based tests (`proptest`). |
| **Integration** | Spin up real Postgres + Redis + Elasticsearch in `testcontainers`; run the same API flows. |
| **Chaos** | Kill the DB mid-write; verify cache invalidation still happens via outbox. |
| **Load** | `cargo bench` + `criterion` for latency percentiles; `drill` / `wrk` for end-to-end. |
| **Contract** | Define `User` schema in Protobuf/Avro; test serialization round-trip. |

---

## 10. Next Steps — A Learning Path

1. **Persist the DB**  
   `cargo add rusqlite` → `PrimaryDatabase` wraps a SQLite file.

2. **Real cache**  
   `cargo add redis` → `InMemoryCache` becomes `RedisCache` with TTL.

3. **Real search**  
   `cargo add tantivy` → `FullTextIndex` uses a persistent index directory.

4. **Real queue**  
   `cargo add lapin` (RabbitMQ) or `rdkafka` → `MessageQueue` talks to broker.

5. **CDC**  
   Run Debezium + Kafka Connect locally; tail the outbox table.

6. **Observability**  
   `cargo add tracing tracing-subscriber prometheus`; add metrics + traces.

7. **Async**  
   `cargo add tokio`; make all handlers `async`, run on a thread pool.

8. **Deployment**  
   Dockerfile + docker-compose for DB, cache, index, queue, worker, API.

Each step replaces one in-memory component with a production-grade piece while
keeping the *architecture* (the arrows between boxes) exactly the same.

---

## Glossary

| Term | One-line definition |
|---|---|
| **ACID** | Atomicity, Consistency, Isolation, Durability — transaction guarantees. |
| **CDC** | Change Data Capture — streaming DB writes to downstream systems. |
| **Inverted index** | Map from term → document ids; the core of full-text search. |
| **Look-aside cache** | App checks cache, on miss reads DB then populates cache. |
| **Outbox pattern** | Write events to a table in the same DB transaction as the data. |
| **mpsc** | Multi-producer, single-consumer channel (Rust std). |
| **Structured logging** | Log lines as JSON with fixed fields, not printf strings. |
| **TTL** | Time-to-live — auto-expiry for cache entries. |
| **Write path / read path** | The sequence of hops a write or read takes through the system. |

---

*Generated from the `api_flow` teaching project — each component is intentionally
minimal so the *shape* of the system stays visible. Replace one piece at a time
with real infrastructure to see how the concepts survive contact with reality.*