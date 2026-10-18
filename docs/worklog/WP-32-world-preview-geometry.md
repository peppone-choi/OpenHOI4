# WP-32 M-1c world-preview geometry harness

## Scope and ownership

- taskId: t_200364e3; project: openhoi; stage: implement.
- Repository: https://github.com/peppone-choi/OpenHOI4 ; base: `092938b5aec9b50ad056205c8066987d420f1243`.
- Branch: `impl/t_200364e3`; draft PR: https://github.com/peppone-choi/OpenHOI4/pull/28 .
- Contract: lead D1–D5 and plan t_9c7e57ec §5. This is a geometry harness, not product pack.
- Current non-product D1 contract: [D1-R2 singleton fix](singleton-fix-t_f9e1528d.md) supersedes the original D1 below. D2–D5/Q1 remain undecided product policy; the existing harness technical contract is not product approval.
- Source preview, `data/**`, client, product Rust source, existing tests/goldens, manifests and other workflows are unchanged. Generated PNG/map files are hosted artifacts only. The single technical state and copied testland registries live only in a Rust-owned temporary directory, removed on success or unwinding.
- Q1 (country/ownership/era), actual terrain content, M-1/AC10, final product QA and gameplay remain undecided/NOT_RUN. This does not approve removing small islands/lakes from the product map.

## Input → output contract

`tools/maps/export_pack_geometry.py --input client/public/preview/world --out <new temporary directory> --terrain-map land=plains,sea=ocean,lake=inland_water`

The terrain map is required with no defaults. IDs must match `[a-z0-9_]+`. The test mapping is technical, not a content decision.

- `world-preview-v1` index is u16 little-endian, zero-based dense index. Restore the existing ID with +1 before processing; surviving IDs are never compacted/renumbered.
- Validate `index.bin`, `provinces.json` AND `adjacency.json` against the actual metadata.hashes schema before producing any output. Validate dimensions, extent, byte order, IDs, pixel areas and kind. A missing/mismatched/extra hash declaration fails by filename.
- Hash trust boundary: metadata declares hashes of exactly these three files, NOT itself. Its own computed SHA256 is recorded, not authenticated against a nonexistent self-reference. Self-referential/unknown declarations fail. The report records hashes of the four geometry outputs, excluding itself; CI additionally compares ALL output bytes including report.json across two runs. Coordinated metadata+payload rewriting is not a cryptographic authenticity claim.
- Freeze original singleton IDs and areas. For each singleton, choose only ORIGINAL area>=2 four-way neighbours including longitude wrap; same kind first, most shared edges, then lowest ID. No singleton chain, no iterative merge, no remote fallback. Missing eligible neighbour fails with the singleton ID. All decisions precede writes. Report each source/target ID, old/new kind and changed pixel coordinate.
- The preceding bullet is historical original D1, not the current acceptance contract. D1-R2 permits at most two simultaneous snapshot decision stages: stage 1 uses original area>=2 direct four-way neighbours; stage 2 handles only original singletons left without a stage-1 candidate, using direct neighbours with area>=2 in the post-stage-1 snapshot. Longitude wrap/latitude nonwrap and same-kind/most-shared-edges/lowest-ID ranking remain. Apply writes only after all decisions in each stage; no within-stage chain/sequential updates, third stage or distant fallback. Residual singletons after stage 2 explicitly FAIL without changing input. See the linked D1-R2 document for reporting and oracle details.
- Original provinces of area 2–8 remain unchanged in identity and pixels (they may gain absorbed pixels). Only original singleton pixels change.
- Encode ID as RGB `(id >> 8, id & 255, 0)`. Coastal means land with a four-way sea neighbour, including seam, not lake.
- Output: RGB8 static lossless `provinces.png`, `provinces.csv` (`id,r,g,b,kind,terrain,coastal,island`), header-only `adjacency_overrides.csv`, `regions.toml`, `report.json`. `island=false` does not suppress disconnected-component warnings.
- `km_per_pixel = 4.8865025493` is the exact fixed decimal lexeme published by [ADR-0702 §1](../adr/ADR-0702-equirectangular-distance.md), not a runtime float. This exporter is for the world-preview projection; tiny fixtures test policy/encoding, not physical scale. Extent `[-180,-90,180,90]`, projection `equirectangular`.

## Hosted evidence design

One new Ubuntu job in ci.yml; all existing jobs are unchanged.

1. Synthetic unittest contracts: same-kind priority, border count and ID tie-break, no-candidate failure, prohibition on singleton chains, immutable 2–8px, wrap/coastal, all u16 RGB identities, input hash tampering, explicit terrain mapping, metadata self-reference rejection, byte-identical repeated exports.
2. Two full conversions into runner temporary directories, timed individually. Report includes PNG byte count.
3. Independent oracle in `test_export_pack_geometry.py --world-check`: independently decode RGB PNG and CRC, replay every absorption against original pixels, compare every pixel, traverse columns for four-way/wrap edges, compare against remapped original adjacency, verify counts/kinds/three seam samples/full-longitude caps 6788 and 18014. Oracle JSON is outside both generated maps.
4. Opt-in Rust `world_preview_geometry` integration test assembles a temporary pack, calls the ACTUAL `oh_data::map::load_map`, requires wrap_x=true, compares IDs/counts/kind pixels/normal edges exactly, checks positive distances, three seam pairs, and every polar-cap neighbour distance against exact fixed-point dy*scale (dx=0). It records load time and actual loader error even if loading fails. No loader validation is relaxed. Without OH_WORLD_GEOMETRY it prints an explicit SKIP; this default skip is not a full-world PASS.
5. Working-tree cleanliness check plus local base diff allowlist proves the non-product boundary. Always upload times, both geometry outputs/reports, independent oracle and Rust log/metrics as hosted artifacts. No performance thresholds are inferred.

## RED and execution status

- Hosted RED commit `cd57696f116f51ddf2e4660ba6b758d9bf9b6251` failed at the new synthetic contracts: `ModuleNotFoundError: No module named 'export_pack_geometry'`.
- Run: https://github.com/peppone-choi/OpenHOI4/actions/runs/39121054220 ; exact failed job: https://github.com/peppone-choi/OpenHOI4/actions/runs/39121054220/job/117656572596 . Raw log retained in task evidence. Superseded RED workflows were cancelled after preserving failure evidence to release hosted slots.
- Local: Python py_compile and pinned rustfmt-only checks; no local build/test/full conversion. The host lacked cargo/rustfmt on PATH; official checksum-verified Rust 1.99 components were unpacked in external task evidence solely to run rustfmt. No toolchain/repository dependency changes.
- Final exact SHA, hosted run URLs/status, artifact hashes/counts/timings and any failures are recorded in external `results/t_200364e3-RESULT.json` and `evidence/t_200364e3/IMPLEMENT.md` after execution, not guessed here.
- Independent review, final product QA/M-1/AC10, actual pack/gameplay and merge are NOT_RUN by the implementer. Windows PC/HOI4/GCP/operations/deploy/reset/new paid services are NOT_RUN. productComplete=false.
- Next assignee: openhoi-lead for independent review/verification assignment. No self-approval or merge.
