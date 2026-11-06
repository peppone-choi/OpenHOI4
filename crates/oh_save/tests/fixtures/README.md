# M1 save fixture (current format)

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

Since ADR-1001 (2026-11-06) the file is in the single current save format (v9).
It was rewrapped from the original v1 bytes with the same state; a fresh native
capture reproduces it byte for byte. Any body format change regenerates it.

Provenance: original OpenHOI contributors, CC-BY-SA-4.0 (registered in ASSETS).
