# Normal template Qty query and tooltip acceptance

This accepts the seven additive normal-template Qty fields only. Weighted
organization/armor/piercing, minimum Fx speed, military commands, template editing,
shortages and actual division supply consumption remain outside this feature.

Use Rust1.99, Node24/npm11 and an installed Chromium. Build the actual client before
the server; oh_server embeds dist and does not read it dynamically at runtime.
Keep generated fixtures, save files, host logs and screenshots outside Git.

```sh
cargo run -p oh_proto --example generate
npm --prefix client ci
npm --prefix client run build
cargo build -p oh_server
cargo test -p oh_proto
cargo test -p oh_server
npm --prefix client test -- --maxWorkers=1
```

Generate a **new** checkpoint directory using the current pack; the save test
refuses to overwrite an existing checkpoint. Do not substitute an old run's save
or use --force to make a PackMismatch pass.

```sh
OH_M2_MILITARY_OUTPUT_DIR="$PWD/target/wp22/checkpoint-current" \
  cargo test -p oh_save --test m2_military_pack
```

For real old-server/new-client compatibility, preserve a separate checkout of
producer baseline `bbebbe111dcea346234f1249ed9d391fac095090` (same source tree as
merge main `f94f2c0376621f5f507fff0a4bfe562e4cbc418b`). Copy the current generated
client/dist into its client/dist and build its oh_server with a **separate Cargo
target directory**. This baseline server has no ledger query dispatch, while it
serves the new client. Record source status, executable SHA256 and served JS bytes.
Using the same target directory across checkouts can retain stale package
fingerprints; do not treat its executable as current without forcing the relevant
package rebuild and verifying its source provenance.

Set absolute executable/checkpoint paths, then run from client:

```sh
cd client
OH_SERVER_EXECUTABLE=/absolute/current/oh_server \
OH_LEGACY_SERVER_EXECUTABLE=/absolute/baseline/oh_server \
OH_M2_MILITARY_OUTPUT_DIR=/absolute/current/checkpoint \
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/usr/bin/chromium \
  node_modules/.bin/playwright test -c playwright.military-ledger.config.ts
```

The worker copies the current pack into generated roots, adds a second synthetic
template for selection tests and separately restores the unchanged current pack.
It owns and gracefully stops its host PIDs and records exit/aliveness evidence.
Original packs are never edited. One browser worker is used.

| Requirement | Direct check |
|---|---|
| Actual checked Qty contributions and final normal bits | oh_proto military_normal_ledger; actual served browser native case |
| 64-character IDs, 12/4 occurrences, duplicate IDs separated, large raw Qty | shared synthetic maximum fixture; Rust codec and client guard |
| Read-only state and definition binding | projection state hash invariant; client base/hash/final-bits checks |
| Base query loss and recovery | V→null→ledger reply→V unit RED/GREEN; injected authority loss in actual served App; late reply after recovery |
| Whole MessagePack envelope budget and valid controls | server exact-byte/one-byte-short unit; actual256-byte host stays connected |
| Malformed request cannot echo past budget | native1024-digit bad serial receives a bounded invalid-message Notice, then legacy control succeeds |
| Legacy response unchanged | old query exact keys; actual baseline server/new client fallback |
| Capability versus domain errors | native unknown/invalid template then valid control; fallback lifecycle unit |
| Serial/socket/epoch/generation and pending bound | lifecycle unit; held A→B→A, future, duplicate, close/reopen, reconnect |
| Current and restored serving | exact dist JS bytes; V7 paused tick144 and same state hash |
| ko/en keyboard and mobile | seven native details tooltips and focused screenshots |
| Malformed wire handling | injected malformed contributor ID triggers existing disconnect and stale display |

Injected timing/error cases retain a real host but intentionally replace or hold
specific frames. They are distinct from native feature and compatibility checks.
This recipe and author-run results do not replace independent QA or source review,
and do not accept all of WP-22, REQ-UI-04, AC-M2-03 or M2.
