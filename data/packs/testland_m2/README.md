# Synthetic Testland M2 initial content

Standalone content slice: six fictional test nations, 120 land provinces and 12 states, with the existing economy producer. This is not full WP-23 or a playable M2 campaign. No equipment, rail, OOB, supply, combat, diplomacy, AI, agenda or victory rules are included. Neutral numbered labels have no permanent lore. Existing M1 and golden/repro inputs remain separate.

Validate and run from the repository root:

```sh
cargo run -p oh_cli --locked -- validate --deny-warnings data/packs/testland_m2
cargo run -p oh_cli --locked -- run --pack data/packs/testland_m2 --scenario m2_initial --days 365 --seed 1 --hash-out
```

The current server selects a `testland` child folder. To view this separate pack without changing engine routing, copy it byte-for-byte to a fresh temporary `<host>/testland`, verify sorted relative lengths/SHA256, and launch `oh_server --pack-root <host> --scenario m2_initial`. The manifest remains `testland_m2`. This staging is required; direct arbitrary-pack server selection is not implemented. The existing client displays nation/state/map data; economy is verified with its supported server query, not a production UI.

## Deterministic map recipe

RGB PNG is 600×500. For row r=0..9 and column c=0..11, province ID i=12r+c+1 fills inclusive rectangle (50c,50r)..(50c+49,50r+49). RGB is (30+(43i mod190),30+(67i mod190),30+(97i mod190)); distinct IDs never share RGB within 1..120. All are land/plains/noncoastal. CSV header is `id,r,g,b,kind,terrain,coastal`, rows are ascending IDs and LF-terminated. No adjacency override is present; the engine derives the 4-neighbor graph. Projection is 2 km/pixel, inherited synthetic M1 configuration.

Nation n=1..6 owns columns 2(n−1),2(n−1)+1. State 2n−1 covers rows0..4 and state2n rows5..9; each has 10 provinces. Capitals are 2n−1 (top row), each has VP1. CSV/PNG generation uses Pillow RGB rectangles, `save(format="PNG", compress_level=9, optimize=False)` without metadata; implementation generation used Pillow12.3.0. PNG encoder-version changes may change bytes, so also compare decoded pixels and CSV. These are content-generation operations outside the simulation.

PNG SHA256: `e3eeb968006ecd7c557b55b55a6d9b55e3d05cbc7bbecc252cbac4802a8b527c`. CSV SHA256: `aa6079ea7816196b2ebee8633ee1407fcb379bb75cd5a27a5baf1f44d8207b3a`.

Every initial nation owns two states: population2000, steel daily flow4, IC20, sector IC5 each, conscription capacity250 and available250. PC starts0, gains3/day up to100. All numeric inputs are provisional test balance, not historical facts. Existing laws/formulas are reused; no new economy mechanics were defined. See [SOURCES.md](SOURCES.md) for values and licensing.

Pack data and original bitmap: CC BY-SA4.0, OpenHOI contributors. No third-party game material or AI image was used.
