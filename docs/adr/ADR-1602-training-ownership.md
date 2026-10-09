# ADR-1602: Explicit training and division ownership

Status: implementation checkpoint; independent integration acceptance pending.

The approved training policies require ownership beyond anonymous economic
reservation totals. An opt-in military definition supplies immutable templates,
whole-day durations and fixed initial armies/roster. Legacy committed/reserved
totals are explicitly retained as background buckets; they are not converted
into invented jobs or units.

TrainingJob owns its exact reserved manpower and held equipment. Military owns
the monotonic job/division IDs, terminal job records, and army membership. Normal
template results are immutable cached snapshots checked against the active
definition authority during restoration. An actual Division owns deployed
manpower and equipment. Deployment transfers these quantities once rather than
withdrawing them again; cancellation releases only the current job's holdings.

Per-model and per-priority allocation uses integer Hamilton largest remainders,
with ascending entity IDs for equal remainders. Existing division deficits are
flat within an army-priority level. Priority numbers ascend from zero (highest)
to 65535 (lowest). Every active priority level precedes training
jobs. Full-manpower pending admission scans IDs and permits smaller later jobs
to start when a larger job remains pending. These are the user-approved policies,
not new production tuning.

Returnable training equipment occupies inventory capacity together with free
stock. Production keeps its existing whole-output cap failure and atomic rollback;
no silent discard, clamp or invented overflow carry is introduced. Military
commands remain before clock advance, and the daily military phase follows
production/construction. A job started at tick24 has zero progress; a two-day
training job becomes Ready at tick72.

Military absence omits the state entirely and takes the existing production path.
Save/query/wire integration is a separate serial implementation. No new combat,
understrength-effect, supply, movement, editor or general-modifier formula is
derived by this state ownership contract.

## Approved active admission quota and save implications

The 2026-10-09 approval removes the global lifetime count of 4096. A country's
Pending, Training and Ready jobs together have a technical admission bound of
4096; Ready still waits for explicit deployment. Cancelled and Deployed records
release active capacity while remaining in history. This magnitude is the prior
technical bound, not a new campaign balance or smaller gameplay limit.

Admission checks the current country when Train applies, rather than reserving
quota for future queued commands. A preceding cancellation/deployment in command
order can release capacity. Semantic quota rejection consumes that command under
the existing command contract, but consumes no job ID or resources. The core reports ActiveLimit; the existing
transport maps it to invalid-message without a protocol variant or key change. Restore also
checks per-country active counts. Counts are derived from job statuses, not a
new persisted cache. The exact contiguous history, next job/division counters,
ownership, accounting and atomic resource transfer checks remain in place.

The old V7 reader/exporter keeps its total-history bound of 4096. States with
4096 or fewer records remain V7 and preserve the previous save bytes. Extended
states use a distinguishable V8 prefix/header tag with the same canonical body
layout, enum/field order, definition identity and state hash representation.
V8 validates per-country active counts. Relabelling an extended V8 body as V7
is rejected, including force mode. Existing V7 saves restore without reset,
renumbering or production-data migration. V1–V6 remain frozen.

The encoder selects V8 only when the retained total exceeds the old V7 bound.
It checks the candidate's authority and the same reader's byte/collection/
allocation bounds before returning bytes to the existing atomic writer. These
engineering changes follow the actual independent review's versioning plan;
there is no irreversible or destructive migration. Older binaries do not read
V8; the new version records that boundary explicitly.

Terminal retention is unchanged: no pruning, tombstones, renumbering or ID reuse.
General save byte/collection/allocation limits still apply independently of the
active admission quota. Unbounded history growth or unlimited save capacity is
not promised; a future retention/resource policy needs its own explicit decision.
Queued duplicate tuple rejection and existing per-connection wire sequence
rejection remain unchanged. This does not add a persistent cross-connection
request ledger or a new replay contract.

History traversal, canonical serialization, query transmission and transactional
cloning remain linear in retained records and consume memory/CPU/bandwidth.
Pagination, archives and retention are separate future scope. Existing read bounds
include one million entries, 256 MiB body/allocation and 64 MiB file; nested
allocation charges can reach the allocation budget before the entry bound.
