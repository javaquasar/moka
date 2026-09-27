# Add a lightweight post-removal observer to `future::Cache`

## Summary

Add `CacheBuilder::post_removal_observer`, a synchronous callback for callers that only need to
publish a small removal record without enabling eviction-listener futures or the listener-only
per-key lock map.

The callback receives the removed `Arc<K>`, cloned `V`, and `RemovalCause`. It runs after the cache
has logically removed/replaced the entry, may run concurrently, and is called before an eviction
listener when both APIs are configured. A panic is contained and disables subsequent observer
calls.

## Why a separate API

An eviction listener has asynchronous and serialization semantics that are valuable for removal
work, but those semantics have a measurable fixed cost even for a no-op callback. The observer is a
narrower contract: it must be fast and nonblocking, cannot wait for backpressure or reenter the same
cache, and delegates queueing, reconciliation, and shutdown barriers to the caller.

In five fresh release processes per mode/operation pair, median allocation count versus
notification-off was:

| operation | no-op listener | no-op observer |
| --- | ---: | ---: |
| insert | +46.73% | 0.00% |
| explicit remove | +784.63% | +0.03% |

Median gross allocated bytes changed by `+121.13%/+29.61%` for listener and `+0.03%/+0.01%` for
observer (insert/remove). This benchmark classifies allocation ownership; it is not an elapsed-time
performance claim.

## Contract choices in this prototype

- Observer and eviction listener may coexist; observer runs first.
- Observer panic is caught and disables future observer calls.
- `run_pending_tasks` makes pending cache removals observable, but does not wait for work that a
  callback publishes outside Moka.
- The callback may be concurrent and must not block, perform I/O, wait for backpressure, or reenter
  the same cache.
- Observer-only caches do not create the listener key-lock map.

These choices are intentionally called out for maintainer review; the API can be reshaped before
merge.

## Validation

- [x] All-feature unit, integration, and doc tests
- [x] Native tests for `Explicit`, `Replaced`, `Expired`, and `Size`
- [x] Old-value replacement and observer-before-listener ordering
- [x] Panic containment and observer disablement
- [x] Proof that observer-only does not enable listener key locks
- [x] Strict Clippy across all features and targets
- [x] Rustfmt and whitespace checks
- [x] `future` and `sync` feature checks
- [x] Rust 1.71.1 future test suite
- [x] Package construction
- [x] Reproducible raw allocation evidence with source/environment metadata
- [ ] Fork GitHub Actions matrix and cross-compile jobs
- [ ] Maintainer agreement on the public contract

## Notes for reviewers

The MSRV verification needed two additional temporary transitive pins (`encoding_rs 0.8.35` and
`actix-macros 0.2.4`) beyond Moka's current pin script because newer releases now require edition
2024/Rust 1.88. Those pins are not part of this change.
