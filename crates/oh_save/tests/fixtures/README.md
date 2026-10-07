# Original M1 save v1 compatibility fixture

This is a new, directly authored Testland fixture, not an updated existing golden.
The numeric expectations come from the save contract and independent integer
postcard/FNV/ledger reference in `crates/oh_save/tools/save_reference.py`.

`m1-v1.ohsave` fixes saved_at to Unix UTC second 0. It contains the actual M1
world at tick48 on 2000-03-01, seed7, with five pending time commands and raw-bit
infrastructure modifiers whose first contribution expires at tick49. Its
native fresh-process continuation ends at tick96 on 2000-03-03. Both original
and resumed state hashes must equal the committed expected JSON.

Run from the repository root:

```
python crates/oh_save/tools/check_save_determinism.py capture --out target/save-determinism
python crates/oh_save/tools/check_save_determinism.py compare --root <three-OS-artifacts>
```

Capture builds and executes native binaries; comparison rejects missing OS,
dirty/different HEAD, command/exit/PID, file SHA and canonical/ledger mismatch.
Synthetic labels used by the negative predicate tests are not actual OS runs.
Only an exact clean source HEAD and real CI artifacts establish cross-OS evidence.
Ownership/player permissions and browser upload/download are later milestones.

Provenance: original OpenHOI contributors, CC-BY-SA-4.0 (registered in ASSETS).

## Original WP-17 movement v2 compatibility fixture

`movement-v2-97f.ohsave` and `movement-v2-97f.expected.json` are immutable raw
producer output from source `97f688718dabed46bc477fa249798ac09e999581`, Windows
native `movement_fixture` at tick1, unit900 at province10, elapsed1h, with
pending Stop/Pause/Resume/Move. Save SHA256 is
`d3706ae77d226ad480ce24e018137de6898bd45b0bd6dbb203b1a4a4c6c9df0f`.
Split canonical hash is `18bff036731f4f26`; continuous/resumed tick24 hash is
`a145b72ad0ad25e6`. Source artifact:
`target/evidence/WP-17/exact-movement-1/` (producer identity/PIDs and SHA in
`exact-movement-result.json`). This is a new compatibility fixture preserving
an earlier implementation's bytes, not a replacement golden/expected value.
Original engine-generated synthetic test output; content is the existing
independently authored Testland pack. No original-game assets or values.

Historical input: `movement-pack-97f/` preserves all nineteen original
`data/packs/testland` Git blobs from the same source97f. Its independent
whole-pack Postcard/FNV identity is `13318001328374612931`. File-level
SHA256/source manifest is the sibling `movement-pack-97f.source.json`, outside
the pack root so it cannot change identity. The original-v2 regression uses
only this real historical input; current pack v3 roundtrip and changed-pack
rejection are separate. No --force, header rewrite or expected replacement.
The future WP24 frozen pack directory is neither assumed nor used here.
