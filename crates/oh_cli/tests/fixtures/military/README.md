# Authored WP-16 military acceptance input

These values are synthetic test data, not campaign tuning. The temporary pack
generator copies the existing two-country M1 input, adds real economy/production
authority, and opts scenario `m1` into `common/military/synthetic.toml`. Shipped
M1/M2 packs are never rewritten. `normal_templates` is part of this actual pack
authority; the older external `military-template inspect --definitions` identity
remains a diagnostic command with no simulation state.

The input has three templates: `small` requires 8 people/6 items, `tiny` requires
2 people/2 items, and `large` requires 100 people/6 items. Each declares two whole
training days. Country 1 starts with capacity 125 and legacy anonymous committed
60/reserved 10, so availability is 55. Those anonymous totals are explicitly
background, not invented jobs or divisions. Stock starts at four items of model
`test_model_1`; model 2 exists but is never substituted. Armies 0 and 2 belong to
country 1; army 1 belongs to country 2. Each has one authored general and two
slots. Province 10 is controlled land of country 1, while province 20 is controlled
by country 2 despite country 1 ownership; 50 is sea and 60 is lake.

Generate the temporary pack with `cargo run -p oh_cli --example military_fixture`.
Use the emitted directory as `--pack` below. Commands have the same types and
decimal identifier strings as the actual server protocol:

```text
oh_cli military run --pack <emitted-root> --scenario m1 --seed 1
{"op":"enqueue","nation":1,"sequence":"1","tick":"0","command":{"type":"Train","template":"small"}}
{"op":"step","count":24}
{"op":"step","count":48}
{"op":"query"}
{"op":"save","path":"/outside/pack/training.ohsave"}
```

Admission is at tick 24 with progress zero, the following daily boundary adds
day one, and tick 72 publishes Ready. No automatic deployment occurs. Explicit
under-equipped deployment uses the Ready job and transfers actual held resources:

```json
{"op":"enqueue","nation":1,"sequence":"2","tick":"72","command":{"type":"Deploy","job":"0","army":"0","province":10,"allow_understrength":true}}
{"op":"step","count":1}
```

`military resume --pack <same-root> --load <checkpoint>` restores the real save
format and queue in a fresh process. `query` reports actual state/hash and the
same military, economy and production projections used by the server. `step`
returns semantic command errors without concealing the successful step; a daily
phase error reports the uncommitted state and leaves its queue intact. A multi-step
request retains earlier successful steps and stops at its first failed step.

Native host limits are 1 MiB per input line and one million steps per request.
They bound host requests and do not define training balance. An input error stops
the host. There is no force/migration/default-definition option. Saves use a
fixed diagnostic timestamp for byte comparisons, reject destinations inside the
pack, and cannot overwrite the resumed input checkpoint. Training, deployment,
priority and cancellation rules are calculated only by actual Simulation.

`crates/oh_cli/tests/military.rs` drives new native processes for pending
backfill, next-day progress, Ready, partial/zero-equipment explicit deployment,
fresh owner/control/army-slot validation, cancellation/terminal guards, paused
commands, active-before-training flat Hamilton allocation across armies,
capacity shrink, joint stock/returnable holding cap atomicity, and real V7
fresh-process queue/state/byte preservation. No combat, movement, supply,
experience, editor legality or general modifier formula is inferred here.
The host boundary checks also cover integer overflow, noncanonical identifiers,
unknown JSON fields, and checkpoint/pack overwrite guards with byte preservation.
