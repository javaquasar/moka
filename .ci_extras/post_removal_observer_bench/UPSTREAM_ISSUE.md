# Proposal: lightweight synchronous post-removal observation for `future::Cache`

## Problem

`future::Cache` currently offers synchronous and asynchronous eviction listeners. They are the right
API when removal work may be asynchronous, but enabling even a no-op listener also enables the
listener-only per-key locking path and creates listener futures. Some users only need to publish a
small accounting record into their own bounded queue after logical removal.

In a standalone allocation benchmark, the listener changes allocation behavior even when no key is
removed during the measured insert loop. A prototype synchronous observer retains the listener-off
insert path and leaves deferred work, backpressure, reconciliation, and shutdown barriers to the
caller.

## Proposed shape

```rust
Cache::builder()
    .post_removal_observer(|key: Arc<K>, value: V, cause: RemovalCause| {
        let _ = sender.try_send((key, value, cause));
    })
    .build();
```

The callback is synchronous, may be invoked concurrently, must not block or reenter the same cache,
and does not promise completion of work published outside Moka. The prototype delivers
`Explicit`, `Replaced`, `Expired`, and `Size` after logical removal. When both APIs are configured,
the prototype calls the observer before the eviction listener. A panic is caught and disables later
observer calls while leaving the cache usable.

## Prototype evidence

- Five independent release processes per mode/operation pair, with rotated order and raw CSV. Against
  notification-off, observer median allocation count changed by `0.00%` for insert and `+0.03%` for
  explicit remove. A no-op listener changed it by `+46.73%` and `+784.63%`, respectively.
- Observer median gross allocated bytes remained within `+0.03%` for insert and `+0.01%` for remove
  of notification-off. The no-op listener changed those medians by `+121.13%` and `+29.61%`.
- Native tests cover all removal causes, old-value replacement, observer/listener order, panic
  containment, and proof that observer-only caches do not allocate the listener key-lock map.
- All-feature tests, doctests, strict Clippy, feature checks, packaging, and Rust 1.71.1 future tests
  pass locally. Cross-platform CI remains the final pre-merge authority.
- A downstream bounded-queue consumer with duplicate-tolerant, version-conditional cleanup.

The allocation benchmark classifies ownership and does not claim an elapsed-time speedup. Raw data,
environment metadata, hashes, and exact commands are included with the prototype branch.

## Questions

1. Should observer and eviction listener be mutually exclusive?
2. If both are configured, what ordering and panic behavior should be guaranteed?
3. Should the callback receive `V`, `Arc<V>`, or an internal immutable entry token?
4. What should `run_pending_tasks` and cache drop guarantee about observer visibility?
5. Should same-cache reentrancy be forbidden contractually or detected?
6. Is disabling the observer after a caught panic consistent with Moka's listener policy?

This proposal is motivated by a general accounting/metrics use case rather than a request for Moka
to manage the caller's deferred work. We would prefer maintainer guidance on the contract before
opening a final implementation PR.
