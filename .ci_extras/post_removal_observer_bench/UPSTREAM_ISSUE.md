# Proposal: lightweight synchronous post-removal observation for `future::Cache`

## Summary

Would the project be open to a lightweight synchronous post-removal observer for
`moka::future::Cache`?

The existing eviction-listener APIs are the right choice when removal work needs asynchronous
execution or per-key serialization. Some applications, however, only need to publish a small
accounting record into their own bounded nonblocking queue after an entry has been logically
removed. For that narrower use case, enabling an eviction listener also enables listener-only
per-key locking and creates listener futures.

I have a tested prototype of a separate `CacheBuilder::post_removal_observer` API and would like
maintainer feedback on the contract before opening an implementation PR.

## Motivating use case

A cache user maintains external accounting derived from removal events. The callback itself does
not perform cleanup, I/O, or backpressure-sensitive work. It only attempts a nonblocking send into
a caller-owned bounded queue:

```rust
let cache = Cache::builder()
    .post_removal_observer(|key: Arc<K>, value: V, cause: RemovalCause| {
        if sender.try_send((key, value, cause)).is_err() {
            dropped_events.fetch_add(1, Ordering::Relaxed);
        }
    })
    .build();
```

The caller owns queue capacity, overflow handling, reconciliation, draining, and shutdown. Moka
would only deliver the removal observation.

## Proposed contract

The prototype is intentionally narrower than an eviction listener:

- The API is initially scoped to `future::Cache`, where avoiding listener futures and the
  listener-only `KeyLockMap` matters.
- The callback receives the removed `Arc<K>`, cloned `V`, and `RemovalCause`.
- It observes `Explicit`, `Replaced`, `Expired`, and `Size` removals after logical removal.
- It is synchronous and may be invoked concurrently for different removals.
- It must finish quickly and must not block, perform I/O, wait for backpressure, or reenter the same
  cache.
- When an observer and eviction listener are both configured, the prototype invokes the observer
  first.
- A panic is caught and disables subsequent observer calls, matching the eviction listener's
  existing circuit-breaker policy. An observer call already running concurrently may still finish.
- Observer-only caches do not enable listener-only per-key locks.
- Cache drop does not synthesize removal observations for remaining entries, matching the existing
  listener lifecycle.
- `run_pending_tasks` makes cache-maintenance removals observable, but does not wait for work that
  the observer publishes outside Moka.

This is not intended to replace the eviction listener. Applications that need asynchronous work,
per-key serialization, or Moka-managed cancellation recovery should continue to use the existing
listener API.

## Prototype allocation evidence

The benchmark starts every sample in a fresh release process, rotates the mode/operation order,
and runs five samples for each pair. Relative to notification-off, the median allocation counts
were:

| Operation | No-op eviction listener | No-op observer |
| --- | ---: | ---: |
| Insert | +46.73% | 0.00% |
| Explicit remove | +784.63% | +0.03% |

Median gross allocated bytes changed by:

| Operation | No-op eviction listener | No-op observer |
| --- | ---: | ---: |
| Insert | +121.13% | +0.03% |
| Explicit remove | +29.61% | +0.01% |

This benchmark is evidence about allocation ownership, not a claim that elapsed time improves by a
corresponding amount.

The prototype, benchmark source, raw CSV, environment metadata, and checksums are available in the
fork branch:

https://github.com/javaquasar/moka/compare/v0.12.16...hydracache/post-removal-observer-upstream-prep

## Validation completed

The prototype currently has 18 focused observer tests covering:

- all four `RemovalCause` variants;
- `remove`, single/all/predicate invalidation, TTL, TTI, and weighted capacity eviction;
- replacement and removal of already-expired entries;
- owned and borrowed Entry/compute operations;
- default and custom-hasher builders;
- observer-before-listener ordering;
- cancellation without duplicate delivery;
- concurrent callback execution and concurrent panic;
- panic containment, observer disablement, and listener survival;
- bounded nonblocking queue overflow; and
- proof that observer-only caches do not enable listener key locks.

All-feature, future-only, integration, documentation, strict Clippy, formatting, packaging, MSRV,
cross-compile, Loom, Kani, and Trybuild checks pass on the fork. The Miri workflow reproduces an
existing sync-only `timer_wheel_panic_test`/`parking_lot` futex failure; the failing stack does not
contain the observer or `future::Cache` code.

## Questions for maintainers

1. Does a separate narrow observer API fit Moka, or would you prefer a different extension of the
   existing eviction-listener API?
2. Is `post_removal_observer` an appropriate name, and should the first version remain scoped to
   `future::Cache`?
3. If observer and listener coexist, should observer-before-listener ordering be contractual, or
   should coexistence be disallowed?
4. Are the proposed concurrency, panic, drop, and `run_pending_tasks` semantics consistent with the
   guarantees you want to preserve?

I am happy to reshape the API and split the implementation into the form maintainers prefer before
opening the PR.
