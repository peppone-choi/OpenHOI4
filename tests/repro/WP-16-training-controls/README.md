# Training controls feature acceptance

This accepts the served UI's existing Train/Cancel commands only. Keep exact source,
generated fixtures, binary/dist hashes, wire and owned host exits as evidence.
Author-run checks do not replace independent review/runtime QA or full game QA.

Use approved Rust1.99, Node24/npm11, Python and Chromium. Serialize heavy jobs,
Cargo jobs1 and browser worker1. Preserve worktrees and all original inputs.

```sh
export CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$PWD/target/training-current"
npm --prefix client ci
npm --prefix client test -- --maxWorkers=1
npm --prefix client run build
cargo build -p oh_server --locked
OH_M2_MILITARY_OUTPUT_DIR="$PWD/target/training-checkpoint-new" \
  cargo test -p oh_save --test m2_military_pack --locked
```

Generate a new checkpoint path; do not force a stale PackMismatch. For the actual
old-client control, preserve a separate checkout of pre-ledger source
bbebbe111dcea346234f1249ed9d391fac095090. Build its own client from that exact source:

```sh
npm --prefix /absolute/legacy-checkout/client ci
npm --prefix /absolute/legacy-checkout/client run build
```

Record clean legacy source status and each dist file's SHA256. A real localhost HTTP
proxy serves those actual old static bytes and forwards the current native server
WebSocket frames untouched. It requires no browser security-policy exception. This
is actual legacy-client execution with explicit static substitution; the native
binary does not embed the old bundle.

```sh
cd client
OH_SERVER_EXECUTABLE=/absolute/current/oh_server \
OH_M2_MILITARY_OUTPUT_DIR=/absolute/current/checkpoint \
OH_LEGACY_CLIENT_DIST=/absolute/legacy-checkout/client/dist \
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/usr/bin/chromium \
  node_modules/.bin/playwright test -c playwright.training-controls.config.ts
```

When a resource guard stops a full run, retain that stopped result and run the same
assertions as three sequential groups, with a fresh browser and owned hosts per group:

```sh
node_modules/.bin/playwright test -c playwright.training-controls.config.ts --grep 'native buttons|native restored|spectator'
node_modules/.bin/playwright test -c playwright.training-controls.config.ts --grep 'real CONNECTING|injected timing|injected close|injected missing'
node_modules/.bin/playwright test -c playwright.training-controls.config.ts --grep 'native time controls|actual pre-ledger'
```

Keep the environment variables above for each group. No assertion or worker limit
changes between full and grouped runs. If rustc embedding the bundle reaches the
same guard, `cargo rustc -p oh_server --bin oh_server --locked -- -C codegen-units=1` serializes that
package's code generation without changing product source or guard limits.

The dedicated config runs one Chromium worker and is invoked explicitly; current
general CI does not invoke it. Unit tests run in the normal client suite. The worker
copies the current military pack, owns fresh/restored native hosts and gracefully
stops both with exit0, preserving a pid ledger. Confirm each PID is absent.

| Requirement | Evidence |
|---|---|
| Actual UI Train and Pending Cancel | Native buttons and correlated authoritative job view |
| Training cancellation conserves resources | Current-pack V7 native restored job: reserved8 and equipment6 return; other job unchanged |
| Actual training progress and Ready cancel | Native TimeControls, ko/en keyboard/mobile |
| Spectator/foreign/terminal authority | UI restrictions and authority unit checks |
| send=true and shared bounded correlation | Unit failed-send, monotonic sequence,64 issued bound and unknown/duplicate/socket results |
| ACK requires a later issued query | Unit barrier and injected actual-frame timing with duplicate clicks and unknown ACK |
| CONNECTING and reconnect | Real HTTP upgrade hold; labeled close injection then manual fresh local session |
| Missing/unsupported | Labeled replacement response plus fresh correlated native recovery |
| Actual old client/current server | Legacy source/dist hashes, exact served old JS bytes and native military frames |

No automatic command retry, local gameplay calculations, deployment UI or full
WP16/M2 acceptance is included. The existing local reconnect creates a distinct
simulation; it is not saved-game recovery or shared multiplayer persistence.
