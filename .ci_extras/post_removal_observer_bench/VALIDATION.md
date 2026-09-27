# Validation record

## Source identities

- Upstream base: `moka-rs/moka` `v0.12.16`, `a616ec19e8d4ed938caf8b2c88090331d778d5da`
- Observer implementation: `3c2b464e0b85b203f16e39f1dcd6efa272c40141`
- Benchmark runner/provenance fix: `91a31104e4b5c099b32ea6b9cf1abfa0964b4eda`
- Preserved HydraCache release fork: `352e53faa480c9997272b9c70798dd5b5c15d581`

## Passed locally

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo check --all-features`
- `cargo check --locked --no-default-features --features future`
- `cargo check --locked --no-default-features --features sync`
- `cargo test --all-features`: 167 passed, 6 intentionally ignored; all integration tests passed.
- Doc tests from the all-feature run: 67 passed, 2 intentionally ignored.
- `cargo clippy --lib --tests --all-features --all-targets -- -D warnings`
- `cargo run --example post_removal_observer_async --features future`
- `cargo check --manifest-path .ci_extras/post_removal_observer_bench/Cargo.toml`
- `cargo package --allow-dirty --features future --no-verify`
- Rust 1.71.1 `cargo test --features future`: 103 passed, 2 intentionally ignored; integration
  tests and 30 doc tests passed.

The MSRV run used Moka's `.ci_extras/pin-crate-vers-msrv.sh` dependency choices plus temporary pins
for `encoding_rs 0.8.35` and `actix-macros 0.2.4`. Current newer releases of those transitive
dependencies no longer parse/build on 1.71.1. No MSRV pin was committed to this branch.

Fork CI reproduced this pin drift before compiling Moka. The missing pins were added as a separate
prep-only tooling commit. That commit should be proposed separately or excluded when the observer
PR branch is cut so the API review stays focused.

## Expected or baseline conditions

- `cargo check --no-default-features` fails by design because Moka requires either `sync` or
  `future`.
- The repository has no `deny.toml`; bare `cargo deny check` applies an empty license allow-list and
  rejects the existing MIT/Apache dependency graph. This is not caused by the observer patch and is
  not an upstream CI check.
- Linux and cross-target builds are delegated to the fork's unchanged upstream GitHub Actions
  workflows. GitHub initially registered them as `disabled_fork`; they were explicitly enabled
  before the validation-triggering push.

## Evidence

The benchmark runner launches every sample in a fresh release process and rotates the six
mode/operation combinations. Raw values and host/toolchain metadata are in `results/`.
