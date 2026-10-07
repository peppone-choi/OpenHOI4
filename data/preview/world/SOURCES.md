# World preview sources

Accessed 2026-10-07. Modern generalized geography only; no historical borders, nation ownership or game state.

| Input | Version | Official source / terms | Applied |
|---|---|---|---|
| Natural Earth land | 5.1.1 | https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-land/ | Cell-center land polygon mask |
| Natural Earth lakes | 5.0.0 | https://www.naturalearthdata.com/downloads/10m-physical-vectors/10m-lakes/ | Lake polygon mask intersected with land |

Fixed ZIP bytes/URLs/version/sha256/terms in source-manifest.json. Embedded VERSION.txt was checked at acquisition. Natural Earth [terms](https://www.naturalearthdata.com/about/terms-of-use/) designate site vector/raster versions public domain. AWS official hosted ZIP used because the website's old download link contains malformed `http//`. No original CCM or HOI files used. 10m means scale 1:10,000,000.

Explicit acquisition: `python tools/maps/acquire_preview.py`. Offline generation: `python tools/maps/world_preview.py --out client/public/preview/world`. Existing source mismatch fails; neither command substitutes different data. Generator has no network code.

land/sea nearest seed regions use spherical coordinates and reproducible PCG64. Coastal geometry is the polygon mask; not a rectilinear province partition. Disconnected raster pieces receive separate stable-for-this-preview IDs. Lake fragments and subpixel-coast fragmentation account for count above the seed target. No nation/state content exists. Single lake seed simply assigns connected source lake components; it does not group disjoint lakes into a province. Coast precision limited by raster 0.087890625° and Natural Earth generalization. Subpixel islands/lakes can vanish; dateline raster pieces are separate provinces with seam adjacency.

At first checkpoint: 14,510 total = 7,881 land + 3,960 sea + 2,669 lake. No DEM/elevation/river/city-density input yet. Terrain button disabled; no claimed relief. NOAA ETOPO2022 is a follow-up official subset candidate documented in ADR-3201; not yet downloaded/applied.

Offline tools already installed: Python 3.11.0, NumPy 2.2.6, SciPy 1.17.1. Official [NumPy LICENSE](https://raw.githubusercontent.com/numpy/numpy/v2.2.6/LICENSE.txt) and [SciPy LICENSE](https://raw.githubusercontent.com/scipy/scipy/v1.17.1/LICENSE.txt): BSD-3-Clause. No Python dependency is redistributed in the map/runtime. Pillow is not used because its exact MIT-CMU expression is outside the existing explicit allowlist. SHP reader is local stdlib implementation. No Cargo/npm dependency or lock changes.
