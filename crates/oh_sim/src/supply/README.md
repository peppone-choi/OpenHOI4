# WP19 standalone source unit

This module is deliberately not registered in `oh_sim/lib.rs` yet. It computes a
new immutable daily ledger from explicit per-nation graph/source/demand inputs.
It does not read packs, supply fake division demand, apply division effects,
change movement, write saves or implement a server query. Module registration and
shared contracts must follow accepted WP15 integration in the serialized slot.

For independent algorithm verification, create a temporary Cargo project outside
the repository with `oh_core` as its only direct dependency. Set these two paths
to the actual checkout (do not copy the implementation):

```toml
[package]
name = "wp19_algorithm_harness"
version = "0.0.0"
edition = "2024"
[dependencies]
oh_core = { path = "<checkout>/crates/oh_core" }
[lib]
path = "<checkout>/crates/oh_sim/src/supply/mod.rs"
```

Use installed Rust1.99.0 and the existing offline cache, then run:

```sh
CARGO_TARGET_DIR=<temporary-target> CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test --offline --manifest-path <temporary-harness>/Cargo.toml
CARGO_TARGET_DIR=<temporary-target> CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo clippy --offline --manifest-path <temporary-harness>/Cargo.toml --all-targets -- -D warnings
rustfmt --edition 2024 --check crates/oh_sim/src/supply/mod.rs
```

The 22 tests include 1,323 quantized two-demand conservation cases and 100,620
monotonic trial points, graph cuts/route ties/widest-route prefix correctness,
stranded fixed weights versus a feasible adaptive vector, source/rail/loss raw
ledgers, large Qty/tiny attenuation and cost/demand overflow, shortage scalar
validations and starvation equality/grace/reset semantics. The separate scalar
effect plan is not evidence that any real division's strength/org/attack or
committed movement leg changes.

`defines.toml` contains provisional tunables for future strict pack-loader
integration; it is consumed by sensitivity tests. The small test reader is not
an authoritative TOML loader. `Input.decay`, source capacities, rail capacities,
and already-computed positive travel-hours remain explicit call inputs. The
future loader must validate source kind/building/coast/nation placements, real
map geometry, rail levels and owning-state infrastructure using actual producers.
