# WP19 source graph/formulas and pure World adapter

The module is registered in `oh_sim/lib.rs`; its actual tests now compile and run
under ordinary Cargo checks. The accepted first standalone unit remains documented
at commit `74bc16bb4278479d2742786a6e2513c251934317`. Its historical external
`oh_core`-only harness applied to that commit; the current World adapter also uses
real `oh_data`/`oh_sim` public producers and should be checked in this workspace.

```sh
CARGO_TARGET_DIR=<existing-approved-target> CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test -p oh_sim --locked --offline
CARGO_TARGET_DIR=<existing-approved-target> CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 \
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo clippy -p oh_sim --all-targets --locked --offline -- -D warnings
cargo fmt --all --check
```

`Network` is explicit per-nation graph/source input **without division demand**.
Its `feeders()` answers current static rail connectivity only. `world_adapter::prepare`
reads actual map distance/kind/land/coast, nation's capital, current controller and
current owning-state infrastructure. It requires caller-supplied positive transport
reference speed, nonnegative distance-decay coefficient, an exact infrastructure
level→positive time-factor table and explicit static source/rail metadata. Neither
missing entries nor unsupported levels default/interpolate. Costs retain a→b and
b→a separately, using the established movement evaluation order; shared feeder
rail capacity stays a single undirected budget. Unit-specific movement contexts
and real fulfilled supply ratios are not inputs to this reference-distance graph.

Hub/port building-instance authority is absent, so those placements explicitly
fail as unsupported after basic map/coastal checks. State building counts cannot
create one. The input rail placements/levels are explicit static metadata, never
inferred construction/damage authority. Capital-only real-World inputs work.
World currently guarantees every land controller is Some; water None is copied
exactly. Pure graph tests separately cover future land None as blocked, with land
unchanged and real controller NationId0 positive controls. No restore rule is
weakened to manufacture an actual land-None World.

`Network::with_demands` is an explicit handoff into pure allocation from a trusted
future demand owner; the World adapter never calls it, fabricates divisions, or
runs `calculate` with empty/zero demand. There is no daily Supply authority,
movement/combat mutation, pack loader, save/wire/query or served UI in this unit.
The scalar effects remain formula plans, not evidence of applied division effects.

The preserved 22 algorithm tests include 1,323 quantized conservation cases and
100,620 monotonic trial points. New tests cover directed routes/reverse costs,
None versus ID0, actual World/Map terrain/river/modified infrastructure, exact
coefficient expiry, real control cuts, refs/static levels/coastal unsupported
placements, overflow/rounded-zero costs and fresh-process network metadata.
`defines.toml` remains provisional test tuning, not an authoritative pack loader.
