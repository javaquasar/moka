# Post-removal observer allocation benchmark

This standalone package compares `future::Cache` with notification disabled, a no-op synchronous
eviction listener, and a no-op post-removal observer. It records gross allocated bytes and allocation
count for unique inserts and explicit removals. No benchmark dependency or global allocator is added
to the published `moka` crate.

Each sample is a fresh process. The runners rotate the six mode/operation combinations between
repetitions to reduce fixed-order bias, build the release binary only once, and retain raw CSV.

On PowerShell:

```text
./.ci_extras/post_removal_observer_bench/run.ps1 -Operations 20000 -Repetitions 5
```

On a POSIX shell:

```text
./.ci_extras/post_removal_observer_bench/run.sh 20000 5
```

The insert row isolates the mutation-path cost of merely enabling notification because none of the
inserted keys is removed during the measured loop. The removal row includes delivery cost. These
results classify allocation ownership; elapsed time should be measured separately with a sampling
or benchmarking framework appropriate for the review host.

The runners also save source revision, dirty-state, compiler, host, and invocation metadata beside
the CSV. Results from a dirty source tree should be treated as exploratory only.
