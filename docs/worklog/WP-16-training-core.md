# WP-16 training core implementation checkpoint

This is an implementation checkpoint, not an independent acceptance result or a
WP-16/M2 completion claim. The user approved the four training/reinforcement
policies in the current session; the orchestrator owns their design-document
record and the final integration.

## Implemented scope

- Explicit `scenario.military` selects `common/military/<id>.toml`. The immutable
  authority contains resolved normal-template definitions, positive whole-day
  training durations, explicit anonymous background manpower, authored armies
  with one general identifier and data capacity, and an optional initial roster.
  Actual economy and production registries are required. Existing absent-mode
  packs are untouched.
- Military actions register pending training, cancel actual owned reservations,
  deploy a Ready job into an owned army on controlled land with explicit
  understrength consent, and set an army's reinforcement priority.
- Daily processing follows production/construction. Existing divisions receive
  manpower/equipment before training, with flat per-priority Hamilton allocation
  and ascending-ID ties. Pending jobs start only with their full manpower;
  their first progress day is the following midnight. Equipment shortage does
  not stop progress. Ready jobs continue receiving equipment and wait for a
  deployment command.
- Cancel refunds only actual reservation and held equipment. Deploy transfers
  reservation to committed manpower and equipment into a real Division exactly
  once. Production checks free stock plus returnable held equipment against its
  existing inventory cap and preserves whole-step rollback.
- State validation checks immutable normal snapshots, IDs, status/timing,
  initial roster preservation, exact background-plus-attributed economy totals,
  army membership/capacity, and returnable inventory occupancy. Legacy state
  omits military from serialization and keeps its existing producer path.

## Actual checks and history

Commands use locked offline dependencies and the existing shared build target.
`cargo test -p oh_sim --lib --locked --offline military::tests -- --nocapture`
passed one proportional allocation test, including ID ties and large counts.

The first behavior-test preparation exited 101 because the test attempted to
use an unavailable direct `toml` dependency. It was corrected to use the existing
synthetic production fixture constructor; no dependency was added.

Before the daily military hook was connected,
`cargo test -p oh_sim --test military --locked --offline -- --nocapture` exited
101 with one pass and six failures. Pending-versus-Training, unchanged stock
versus equipment reservation, and absence of the expected inventory-cap failure
were actual behavior assertions. Two initial-roster cases and the early-deploy
case also exposed the test author's invalid province ID; those preparation
errors are not represented as product requirements failing.

After the hook/cap connection and correction to valid province 10, the same
command gave six passes and one failure. The remaining test incorrectly expected
the first whole produced item at tick96. With initial efficiency 1/4 and the
existing daily growth, this fixture first produces a whole item at tick120.
The expected boundary was corrected without dropping the cap/rollback assertion.
Cancellation after the rejected midnight is processed while paused, preserving
the already-full inventory cap.

The final invocation passed all seven behavior tests (0 failures): pending and
start-day timing, gear-independent completion, explicit zero-gear deployment,
exact cancellation and terminal guards, active-before-training priority,
same-priority/training proportions, inventory return space with atomic rollback,
command-before-midnight completion/cancel ordering, and paused-clock behavior.
`cargo fmt --all` was run. No whole-workspace, clippy, schema regeneration,
save/server/client integration, or independent acceptance result is claimed here.

## Handoff and remaining checks

The legacy save bootstrap is a preceding commit authored by the separate save
writer and imported here without modification. This core commit does not own
save helpers or `military_save.rs`. The latter module is registered only when the
save writer's actual implementation is integrated.

The next implementation session should extend targeted tests for insufficient
manpower backfill, capacity shrink while Training/Ready, fresh army/province
validation and army-full preservation, multiple model isolation, malformed
restored ownership/status/normal caches, and initial accounting rejection.
Add the military structural schema and regenerate the additive scenario schema;
connect the separate V7/query/server/CLI work serially and then run required
whole-workspace and client/server checks. Technical resource guards and the
single shared compiler slot remain in force.

There is no new game-policy decision request. Combat, supply consumers, actual
movement effects, editor rules, casualties/disband, and general combat modifiers
remain outside this unit.

## Continuation checkpoint

The structural military schema and additive scenario schema were regenerated
using the existing `oh_data` schema example. Before generation the new snapshot
test failed because `military.schema.json` was absent; the strict malformed-input
and actual opt-in loader tests passed. The loader tests retain duplicate-ID,
unknown-field, domain, reference and missing-producer rejection cases, and an
accepted 4,096-division input with rejection at 4,097.

`cargo test -p oh_core -p oh_data -p oh_sim --locked --offline` subsequently
passed 253 tests including doctests. `cargo clippy -p oh_core -p oh_data -p oh_sim
--all-targets --locked --offline -- -D warnings`, formatting, whitespace, and
document checks passed. These are implementation checks, not independent QA or
the requested model-specific review.

A later restored-state test adds altered ownership, status, timing, reservation,
duration, equipment model and cached normal checks with a valid control. Its
first preparation attempt failed because `oh_sim` had no direct `serde_json`
test dependency; the existing pinned 1.0.151 dependency was added. This failure
is not a game-rule RED result. Runtime validation of that later test remains
pending at this checkpoint because the shared build disk guard paused Cargo.

The owned module registration now references the actual separate writer's
`military_save.rs`. Cargo metadata used unchanged borrowed writer/native files
to generate the additive lock references (`oh_cli` to `oh_proto`, and the
`oh_sim` test dependency); those borrowed files are not included in this core
commit. Registration compilation requires integration of the writer's source,
and the CLI lock reference requires the native manifest. No full workspace,
server/client/native integration, main CI, Claude review, or selector execution
is claimed. The orchestrator retains the remaining integration and acceptance
work.
