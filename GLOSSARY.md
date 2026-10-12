# Data Systems Glossary

Alphabetical reference for terms used in the `api_flow` project and the
`CONCEPTS.md` learning notes. Each entry has a one-line definition, the
project component that demonstrates it, and a "dig deeper" pointer.

---

## A

**ACID** — Atomicity, Consistency, Isolation, Durability. The four guarantees
a transaction provides.  
*Project:* `primary_database` (simulated with `HashMap`; real ACID needs a
DB engine).  
*Dig deeper:* DDIA ch. 7; `rusqlite` / `sqlx` for real transactions.

**Actor model** — Concurrency via isolated state + message passing (no shared
memory).  
*Project:* `message_queue` worker thread is a tiny actor.  
*Dig deeper:* `actix`, `tokio` actors, Erlang/OTP.

**Append-only log** — Immutable sequence of records; foundation of Kafka,
Write-Ahead Logs (WAL), event sourcing.  
*Project:* not shown; `DatabaseChange` enum is the *shape* of one log record.  
*Dig deeper:* DDIA ch. 3; Kafka, Redpanda, `tracing` event layers.

**Async/await** — Cooperative multitasking; tasks yield at `.await` points.  
*Project:* deliberately *not* used (blocking `std::thread` only).  
*Dig deeper:* `tokio` tutorial; `async-std`; `futures` crate.

**At-least-once delivery** — Message may be redelivered; consumer must be
idempotent.  
*Project:* not guaranteed (in-process `mpsc` drops on panic).  
*Dig deeper:* Kafka acks=all + idempotent consumer; transactional outbox.

---

## B

**Back-pressure** — Slow consumer signals upstream to slow down (bounded
channel, TCP window, reactive streams).  
*Project:* `mpsc` is *unbounded* → no back-pressure; producer never blocks.  
*Dig deeper:* `crossbeam-channel::bounded`, `tokio::sync::mpsc`, Reactive
Streams spec.

**Batch processing** — High-throughput, high-latency jobs over bounded data
(MapReduce, Spark, Flink batch).  
*Project:* not shown.  
*Dig deeper:* DDIA ch. 10; `polars`, `datafusion`, `arrow-rs`.

**Bloom filter** — Probabilistic set membership test; "definitely not" or
"maybe yes". Saves disk reads in LSM trees, caches, routers.  
*Project:* not used.  
*Dig deeper:* `bloomfilter` crate; Cassandra, RocksDB internals.

**Bounded context** — DDD term: a model applies only inside a clear boundary.  
*Project:* each module (`cache`, `index`, `queue`) is a bounded context.  
*Dig deeper:* Domain-Driven Design (Evans); context mapping.

---

## C

**Cache aside (look-aside)** — App checks cache; on miss reads DB, then
populates cache.  
*Project:* `read_user` in `main.rs`.  
*Dig deeper:* Redis, Memcached, `moka` (Rust async cache).

**CAP theorem** — Of Consistency, Availability, Partition tolerance, pick two.
*Project:* single-node → no partitions → not exercised.  
*Dig deeper:* DDIA ch. 9; PACELC extension.

**CDC (Change Data Capture)** — Capture DB writes and stream them downstream.  
*Project:* `DatabaseChange` + `process_database_change` (synchronous stub).  
*Dig deeper:* Debezium, `pgoutput`, `mysql-binlog`; transactional outbox
pattern.

**Columnar storage** — Data stored by column, not row; great for analytics.  
*Project:* row-oriented `HashMap`.  
*Dig deeper:* Parquet, ORC, Apache Arrow, `parquet` crate.

**Compaction** — Merge + deduplicate sorted files (LSM trees, Kafka log
compaction).  
*Project:* not shown.  
*Dig deeper:* RocksDB, Cassandra, `sled` compaction.

**Consistent hashing** — Hash ring maps keys to nodes; minimal reshuffle on
scale-in/out.  
*Project:* single node.  
*Dig deeper:* `hash_ring` crate; Dynamo, Cassandra, Redis Cluster.

**Correlation ID** — Unique ID propagated across service boundaries to stitch
traces.  
*Project:* not used.  
*Dig deeper:* OpenTelemetry `traceparent` header; `tracing-opentelemetry`.

---

## D

**Dead letter queue (DLQ)** — Queue for messages that failed processing
repeatedly.  
*Project:* not implemented.  
*Dig deeper:* RabbitMQ DLX, SQS redrive policy, Kafka retry topics.

**Denormalisation** — Duplicate data to optimise read paths.  
*Project:* cache + index + queue all denormalise the primary `User`.  
*Dig deeper:* DDIA ch. 3; materialised views.

**Durability** — Once ack'd, data survives power loss.  
*Project:* in-memory only → *not durable*.  
*Dig deeper:* WAL, fsync, `sled`, `redb`, replicated consensus (Raft).

---

## E

**Event sourcing** — State = fold of immutable event log; events are source of
truth.  
*Project:* `DatabaseChange` is an event, but state lives in `HashMap`, not
the log.  
*Dig deeper:* `eventstore` crate; Greg Young's ES book; `cqrs-es` pattern.

**Exactly-once semantics** — End-to-end duplicate elimination (hard).  
*Project:* not achieved.  
*Dig deeper:* Idempotent keys + transactional outbox + deduplication.

---

## F

**Fan-out** — One write triggers multiple downstream actions (index, cache,
queue, analytics).  
*Project:* `create_user` → DB + index + cache-invalidate + queue.  
*Dig deeper:* Kafka topics per consumer; SNS fan-out; CDC routers.

**Full-text search** — Tokenise, index, rank documents by textual relevance.  
*Project:* `full_text_index` (tiny inverted index, prefix match only).  
*Dig deeper:* Tantivy, Meilisearch, Elasticsearch, PostgreSQL `tsvector`.

---

## G

**Graceful shutdown** — Stop accepting work, finish in-flight, release
resources.  
*Project:* `drop(queue)` closes channel → worker `recv` returns `Err` → loop
exits → `join()`.  
*Dig deeper:* `tokio::signal::ctrl_c`, `axum::Server::with_graceful_shutdown`.

---

## H

**High availability (HA)** — System stays operational despite failures.  
*Project:* single process → *no HA*.  
*Dig deeper:* Raft/Paxos, multi-AZ deployment, leader election.

**Hot/cold data** — Frequently accessed vs. archival; tiered storage.  
*Project:* cache = hot, DB = warm, index = warm.  
*Dig deeper:* S3 Glacier, ClickHouse tiered storage, `object_store` crate.

---

## I

**Idempotency** — Safe to apply the same operation multiple times.  
*Project:* `cache.put(user)` overwrites; `index_user` deduplicates ids.  
*Dig deeper:* HTTP idempotency keys; Kafka exactly-once; `PUT` vs `POST`.

**Inverted index** — Map term → document IDs; core of search.  
*Project:* `FullTextIndex::word_to_user_ids`.  
*Dig deeper:* Tantivy source; Lucene internals; `roaring` bitmaps for
postings lists.

**Isolation level** — How transactions see each other's writes (Read
Committed, Repeatable Read, Serializable).  
*Project:* none (single-threaded).  
*Dig deeper:* DDIA ch. 7; `sqlx::Transaction::set_isolation_level`.

---

## J

**Join (SQL)** — Combine rows from two relations on a key.  
*Project:* `search_users` does a manual join: index → ids → DB rows.  
*Dig deeper:* Hash join, merge join, `sqlx::query!` macros.

---

## K

**Key-value store** — Simplest data model: `GET key`, `PUT key value`.  
*Project:* `HashMap` in DB and cache.  
*Dig deeper:* RocksDB, Redis, etcd, `sled`, `redb`.

---

## L

**Latency percentiles** — p50, p95, p99, p999; averages hide tail latency.  
*Project:* not measured.  
*Dig deeper:* `criterion` benchmarks; HdrHistogram; `metrics` crate.

**Leader election** — Choose one node to coordinate; Raft, etcd, ZooKeeper.  
*Project:* single node.  
*Dig deeper:* `raft` crate; `etcd-client`; DDIA ch. 9.

**LSM tree (Log-Structured Merge-tree)** — Write to memtable, flush to SST,
compact.  
*Project:* not used.  
*Dig deeper:* RocksDB, Cassandra, `sled`, `rocksdb` crate.

---

## M

**Materialised view** — Pre-computed query result kept in sync with base data.  
*Project:* cache + index are materialised views of the `User` table.  
*Dig deeper:* PostgreSQL materialised views; Kafka Streams / Flink SQL.

**Message queue** — Durable buffer between producer and consumer.  
*Project:* `message_queue` (`mpsc` channel + worker thread).  
*Dig deeper:* RabbitMQ, Kafka, SQS, Redis Streams, `lapin`, `rdkafka`.

**Microservice** — Small, independently deployable service owning one
bounded context.  
*Project:* all in one binary; modules *could* be services.  
*Dig deeper:* Service mesh (Linkerd, Istio); gRPC, `tonic`.

**Monotonic clock** — Clock that never jumps backward; for timeouts/intervals.  
*Project:* `std::thread::sleep` uses it implicitly.  
*Dig deeper:* `std::time::Instant`; `tokio::time::Instant`.

---

## N

**Normalisation** — Eliminate redundancy via foreign keys (3NF, BCNF).  
*Project:* `User` is flat; no relations yet.  
*Dig deeper:* DDIA ch. 2; `sea-orm` migrations.

---

## O

**Observability** — Logs, metrics, traces; ability to ask "why?".  
*Project:* `println!` with component prefixes.  
*Dig deeper:* OpenTelemetry, `tracing`, Prometheus, Grafana, Loki, Jaeger.

**Optimistic locking** — Version column; update fails if version changed.  
*Project:* not implemented.  
*Dig deeper:* `version: u64` on `User`; `WHERE version = ?` in SQL.

**Outbox pattern** — Write events to an `outbox` table in the same DB
transaction as the data; poller publishes.  
*Project:* `DatabaseChange` is the event; outbox not shown.  
*Dig deeper:* Debezium outbox connector; `transactional-outbox` crate.

---

## P

**Partitioning (sharding)** — Split data by key across nodes.  
*Project:* single `HashMap`.  
*Dig deeper:* Consistent hashing, range partitioning, `vitess`, `citus`.

**Pipeline** — Series of stages; each stage transforms data and passes on.  
*Project:* create → DB → index → cache-invalidate → queue.  
*Dig deeper:* Kafka Streams, Flink, `tokio::sync::mpsc` pipelines.

**Primary key** — Unique identifier for a row.  
*Project:* `User.id: u64`.  
*Dig deeper:* UUIDv7, Snowflake, ULID for distributed IDs.

**Projection** — Derived view (cache, index, read model) from event log.  
*Project:* `process_database_change` builds index + invalidates cache.  
*Dig deeper:* Event sourcing projections; CQRS read models.

---

## Q

**Quorum** — Majority of replicas (R + W > N) for strong consistency.  
*Project:* single replica.  
*Dig deeper:* Dynamo-style quorum; Raft leader commit.

---

## R

**Read path** — Sequence of hops a read request takes.  
*Project:* Client → API → cache → (miss) DB → cache.  
*Dig deeper:* DDIA ch. 3; trace a read in Jaeger.

**Replication** — Copy data to multiple nodes for durability/availability.  
*Project:* none.  
*Dig deeper:* Single-leader, multi-leader, leaderless; DDIA ch. 5.

**Retry with back-off** — Wait longer between retries (exponential + jitter).  
*Project:* not implemented.  
*Dig deeper:* `backoff` crate; circuit breaker (`governor`, `tower`).

**RPC (Remote Procedure Call)** — Call a function over the network (gRPC,
Thrift, JSON-RPC).  
*Project:* in-process function calls only.  
*Dig deeper:* `tonic` (gRPC), `tarpc`, `jsonrpsee`.

---

## S

**Saga** — Choreographed or orchestrated long-running transaction across
services; compensating actions on failure.  
*Project:* not shown.  
*Dig deeper:* `temporal`, `cadence`, `conductor`; Saga pattern (Hellas).

**Schema registry** — Central store for Avro/Protobuf schemas; compatibility
checks.  
*Project:* Rust structs = schema; no registry.  
*Dig deeper:* Confluent Schema Registry, `schemars`, `prost`.

**Serialization** — Convert in-memory struct to bytes (JSON, CBOR, Avro,
Protobuf, MessagePack).  
*Project:* not needed (in-process).  
*Dig deeper:* `serde`, `bincode`, `rmp-serde`, `prost`.

**Snapshot isolation** — Read from a consistent snapshot; writers don't block
readers.  
*Project:* not applicable.  
*Dig deeper:* PostgreSQL MVCC; `sqlx` `REPEATABLE READ`.

**Stream processing** — Continuous computation over unbounded data (Flink,
Kafka Streams, RisingWave).  
*Project:* `message_queue` worker is a 1-task stream processor.  
*Dig deeper:* DDIA ch. 11; `fluvio`, `materialize`, `risingwave`.

**Structured logging** — Log as JSON with fixed fields, not `printf`.  
*Project:* `println!` with manual prefixes.  
*Dig deeper:* `tracing` + `tracing-subscriber` `JsonStorageLayer`.

---

## T

**Transactional outbox** — See **Outbox pattern**.

**TTL (Time-To-Live)** — Auto-expiry for cache entries, DNS records, queue
messages.  
*Project:* not implemented.  
*Dig deeper:* Redis `EXPIRE`, Kafka `retention.ms`, `moka` TTL.

**Two-phase commit (2PC)** — Coordinator ensures all participants commit or
abort.  
*Project:* not shown.  
*Dig deeper:* XA transactions; saga is usually preferred.

---

## V

**Vector clock** — Partial ordering of events in distributed system.  
*Project:* not used.  
*Dig deeper:* Dynamo paper; `versionstamp` in FoundationDB.

---

## W

**Write path** — Sequence of hops a write request takes.  
*Project:* Client → API → DB → index → cache-invalidate → queue.  
*Dig deeper:* DDIA ch. 3; trace a write in Jaeger.

**Write-ahead log (WAL)** — Append-only log of changes before applying to
data structure; enables crash recovery.  
*Project:* not present.  
*Dig deeper:* SQLite WAL mode; RocksDB WAL; Kafka *is* a distributed WAL.

---

## Quick index by project component

| Component | Key glossary terms |
|---|---|
| `primary_database` | ACID, primary key, durability, isolation level, WAL |
| `in_memory_cache` | Cache aside, TTL, invalidation, hit/miss, denormalisation |
| `full_text_index` | Inverted index, full-text search, tokenisation, ranking |
| `message_queue` | Message queue, at-least-once, back-pressure, graceful shutdown, worker |
| `DatabaseChange` | CDC, event sourcing, outbox pattern, fan-out, projection |
| `main.rs` coordination | Read path, write path, saga (future), observability |

---

*Keep this file next to `CONCEPTS.md`; add entries as you replace in-memory
pieces with real infrastructure.*