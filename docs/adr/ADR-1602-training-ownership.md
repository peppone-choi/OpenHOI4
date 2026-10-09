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
flat within an army-priority level. Every active priority level precedes training
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
