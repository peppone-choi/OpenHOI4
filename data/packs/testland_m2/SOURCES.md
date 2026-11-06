# Synthetic content provenance

Checked 2026-10-08. Source baseline: accepted main `0776e6b5a67951821860490964c402e518310ca5` (product producer source `4ae0446a01487e9d6d1415e7191a78407a8e0b6f`). All new pack data and the original bitmap are CC-BY-SA-4.0, authored by OpenHOI contributors. No external/history/game files or image service were consulted. New content balance is **밸런스 잠정**, not user-final balancing or permanent canon. Definitions and initial values use existing producer schema; they do not add game rules.

| Content | Values / provenance | Status |
|---|---|---|
| Six numbered fictional nations, 120 provinces, 12 states, neutral names/colors | WP-23 initial delegated scope; original synthetic layout recipe in README | 밸런스 잠정; no lore |
| Projection/start epoch | `data/packs/testland/maps/testland/regions.toml`:2 km/pixel; `scenarios/m1/scenario.toml`:2000-01-01 | Existing synthetic test values |
| State population/industry/infrastructure/steel | Per state1000/industry1/infrastructure1/steel2, from M1 north state | 밸런스 잠정 |
| Resource registry | oil/steel/aluminium/rubber/rare_metals, five design types from01 §4.3; only steel has output, all others absent/zero flow | Content IDs, no scarcity/market consumer |
| National color palette, state lower-half color +25 per channel | Original presentation values, capped255; no game modifiers | Presentation only |
| Capitals/VP | Top first province per strip:1/3/5/7/9/11; VP1 each | 밸런스 잠정; no victory rule |
| Government/ideology labels | Original fictional test labels; support0.75/0.25 from M1 NTH | 밸런스 잠정; no added political effects |
| Economy industry definition | `crates/oh_data/tests/fixtures/economy/valid.toml`:costs3/20/40, slots1/1/1, cap6, IC10/level; infrastructure0/1/1.125→factor1 | Existing synthetic producer baseline |
| Civil/war law fields/conditions | Same fixture:category economy, steps0/1, costs3/5, multiplier1/2, ratio0.125/0.25, consumer_base0/0.5, slope0; original condition AST unchanged | Existing rules, 밸런스 잠정 values |
| Law category | economy and conscription both refer to fixture's economy category, as its explicit supported input; no new split-law system | Limited synthetic baseline, not final law content |
| Nation economy initial values | Same fixture:PC0/cap100/daily3, stability0.5/mobilization0.5, four allocations0.25, ICmultiplier1. committed/reserved 0/0 (armies start with no divisions) | 밸런스 잠정 |
| State slots | Same fixture:4 each | 밸런스 잠정 |
| Time/network/map display | Existing `data/packs/testland/scenarios/m1/defines.toml` values copied unchanged; comments replaced with provenance | Technical/test configuration, no new mechanics |
| ko/en localisation and metadata | Original bilingual test labels, manifest id testland_m2/version0.1.0, no dependencies | Original content |

All reused values are cited by relative paths at the baseline above; frozen sources were not edited. Localisation `resource-steel`, `building-industry` and `plains` comes from the existing embedded client catalogs; the pack does not duplicate those keys. Zero-flow resources are not displayed as fake implemented consumers. No rail/OOB/research/agenda/victory files or rules are supplied. Final balance, permanent fiction, downstream producer schema and full WP-23 acceptance remain future work.

## Production and military (merged 2026-11-06)

Merged from the former `testland_m2_production` (WP-23 production pack) and `testland_m2_military` (WP-23 military pack, 2026-10-10, base main `0d50a83f862bf575e8051a1c7cf12da7f0ea0428`); their git history holds the original records. Code and data assembly used Codex AI assistance; no image was generated.

| Content | Values / provenance | Status |
|---|---|---|
| Production models/family, nine production defines | m2_equipment_1/2, m2_equipment; N03/N06 allowlist and zero stock extended to model1 | 밸런스 잠정 |
| Template/component | `crates/oh_cli/tests/fixtures/military/training.toml` small: manpower 8, equipment 6, 2 days; IDs m2_small/m2_small_component | 잠정 합성 입력 |
| Eleven normal stats | Same small: strength/soft_fire/defense/breakthrough/frontage/supply_use/organization/speed_kmh 1, hard_fire/armor/piercing 0 | Fixture values, not war balance |
| Armies/background | Six synthetic armies, commanders synthetic_n01..06, capacity 2/priority 0, committed/reserved 0 | Minimal runtime contract input |
| ko/en production strings | Two equipment names in `production.ftl` | Original content |
