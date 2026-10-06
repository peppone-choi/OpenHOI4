//! M1 map loading boundary. Never called implicitly by the M0 pack loader.
//! All IDs/edges/caches use sorted storage; geometry uses checked fixed-point math.
use crate::{DataError, ErrorKind, Fixed, raw::Source, read, valid_id};
use schemars::{JsonSchema, Schema};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use toml::de::{DeTable, DeValue as Raw};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProvinceKind {
    Land,
    Sea,
    Lake,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProvinceDefinition {
    pub id: u16,
    pub rgb: [u8; 3],
    pub kind: ProvinceKind,
    pub terrain: String,
    pub coastal: bool,
    /// Suppresses only the disconnected-component warning, never pixel errors.
    pub island: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeKind {
    Normal,
    RiverSmall,
    RiverLarge,
    Strait,
    Impassable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Canonical endpoint IDs: a < b. Impassable edges remain in the cache.
    pub a: u16,
    pub b: u16,
    pub kind: EdgeKind,
    pub distance_km: Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StateDefinition {
    pub id: u16,
    pub name_key: String,
    pub provinces: Vec<u16>,
    pub population: i64,
    pub resources: BTreeMap<String, i64>,
    pub buildings: BTreeMap<String, i64>,
    pub infrastructure: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VictoryPoint {
    pub province: u16,
    pub points: i64,
    pub name_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatesDocument {
    pub state: Vec<StateDefinition>,
    #[serde(default)]
    pub victory_point: Vec<VictoryPoint>,
}

/// Structural numeric schema. Runtime parses the numeric source lexeme into Fx.
pub fn regions_schema() -> Schema {
    schemars::json_schema!({"type":"object", "additionalProperties":false,
        "required":["km_per_pixel"], "properties":{"km_per_pixel":{"type":"number", "exclusiveMinimum":0,
        "description":"Positive I32F32 scale parsed from its exact TOML lexeme; edge distances must fit I32F32."}}})
}
pub fn states_schema() -> Schema {
    schemars::schema_for!(StatesDocument)
}
pub fn province_schema() -> Schema {
    schemars::schema_for!(ProvinceDefinition)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapWarning {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub province: u16,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapData {
    pub width: u32,
    pub height: u32,
    /// Row-major top-left origin. Index zero is a real province, never a sentinel.
    /// Sorted province IDs map to dense indices independently of CSV row order.
    pub index: Vec<u16>,
    pub provinces: Vec<ProvinceDefinition>,
    /// Pixel-center centroid (x+0.5,y+0.5); division truncates to Fx precision.
    pub centers: Vec<[Fixed; 2]>,
    pub edges: Vec<Edge>,
    pub states: Vec<StateDefinition>,
    pub victory_points: Vec<VictoryPoint>,
    pub km_per_pixel: Fixed,
    pub warnings: Vec<MapWarning>,
    province_states: BTreeMap<u16, u16>,
}
impl MapData {
    pub fn province_index(&self, id: u16) -> Option<u16> {
        self.provinces
            .binary_search_by_key(&id, |p| p.id)
            .ok()
            .map(|i| i as u16)
    }
    pub fn state_for_province(&self, id: u16) -> Option<u16> {
        self.province_states.get(&id).copied()
    }
    pub fn index_le_bytes(&self) -> Vec<u8> {
        self.index.iter().flat_map(|i| i.to_le_bytes()).collect()
    }
    pub fn edge(&self, a: u16, b: u16) -> Option<&Edge> {
        let key = ordered(a, b);
        self.edges
            .binary_search_by_key(&key, |e| (e.a, e.b))
            .ok()
            .map(|i| &self.edges[i])
    }
}

fn error(path: &Path, message: impl Into<String>) -> DataError {
    DataError {
        path: path.to_owned(),
        line: 1,
        column: 1,
        kind: ErrorKind::Schema,
        message: message.into(),
    }
}
fn ordered(a: u16, b: u16) -> (u16, u16) {
    if a < b { (a, b) } else { (b, a) }
}

/// Load a standalone map from <pack>/maps/<id> and common registries. Validation
/// fails atomically; disconnected provinces return ordered warnings on success.
/// WP-24 decides whether warnings are fatal for its --deny-warnings CLI.
pub fn load_map(pack: impl AsRef<Path>, map_id: &str) -> Result<MapData, DataError> {
    let pack = pack.as_ref();
    if !valid_id(map_id) {
        return Err(error(&pack.join("maps"), "map id must match [a-z0-9_]+"));
    }
    let root = pack.join("maps").join(map_id);
    let terrains = registry(&pack.join("common/terrain.toml"), "terrain")?;
    let resources = registry(&pack.join("common/resources.toml"), "resource")?;
    let buildings = registry(&pack.join("common/buildings.toml"), "building")?;
    let csv = read(&root.join("provinces.csv"))?;
    let (provinces, offsets) = provinces(&csv, &terrains)?;
    let png_path = root.join("provinces.png");
    let (width, height, rgb) = bitmap(&png_path)?;
    let mut colors = BTreeMap::new();
    for (i, p) in provinces.iter().enumerate() {
        colors.insert(p.rgb, i as u16);
    }
    let mut index = Vec::with_capacity(rgb.len() / 3);
    for (pixel, &color) in rgb.as_chunks::<3>().0.iter().enumerate() {
        let Some(&i) = colors.get(&color) else {
            return Err(error(
                &png_path,
                format!(
                    "unknown RGB {color:?} at pixel ({},{})",
                    pixel % width as usize,
                    pixel / width as usize
                ),
            ));
        };
        index.push(i);
    }
    let (centers, warnings) =
        geometry(width, height, &index, &provinces, &csv, &offsets, &png_path)?;
    let mut kinds = adjacency(width, height, &index, &provinces);
    overrides(
        &read(&root.join("adjacency_overrides.csv"))?,
        &provinces,
        &mut kinds,
    )?;
    let scale_source = read(&root.join("regions.toml"))?;
    let km_per_pixel = scale(&scale_source)?;
    let mut edges = Vec::with_capacity(kinds.len());
    for ((a, b), kind) in kinds {
        let ai = provinces
            .binary_search_by_key(&a, |p| p.id)
            .expect("validated endpoint");
        let bi = provinces
            .binary_search_by_key(&b, |p| p.id)
            .expect("validated endpoint");
        let distance_km = distance(centers[ai], centers[bi], km_per_pixel)
            .ok_or_else(|| error(&scale_source.path, format!("distance overflow for {a}/{b}")))?;
        edges.push(Edge {
            a,
            b,
            kind,
            distance_km,
        });
    }
    let (states, victory_points, province_states) = states(
        &read(&root.join("states.toml"))?,
        &provinces,
        &resources,
        &buildings,
    )?;
    Ok(MapData {
        width,
        height,
        index,
        provinces,
        centers,
        edges,
        states,
        victory_points,
        km_per_pixel,
        warnings,
        province_states,
    })
}

/// Common definition files may carry later-system fields. This boundary reads
/// only validated, unique IDs and never invents economics or terrain modifiers.
fn registry(path: &Path, kind: &str) -> Result<BTreeSet<String>, DataError> {
    let source = read(path)?;
    let doc = source.document()?;
    let mut result = BTreeSet::new();
    for (key, value) in &doc {
        if key.get_ref() != kind {
            return Err(source.error(
                key.span().start,
                ErrorKind::Schema,
                "unknown registry root field",
            ));
        }
        let Raw::Array(entries) = value.get_ref() else {
            return Err(source.error(
                value.span().start,
                ErrorKind::Schema,
                "registry must use array tables",
            ));
        };
        for entry in entries {
            let Raw::Table(fields) = entry.get_ref() else {
                return Err(source.error(
                    entry.span().start,
                    ErrorKind::Schema,
                    "registry entry must be a table",
                ));
            };
            let Some(id) = fields.get("id") else {
                return Err(source.error(
                    entry.span().start,
                    ErrorKind::Schema,
                    "missing registry id",
                ));
            };
            let Raw::String(id_value) = id.get_ref() else {
                return Err(source.error(
                    id.span().start,
                    ErrorKind::Schema,
                    "registry id must be a string",
                ));
            };
            if !valid_id(id_value) || !result.insert(id_value.to_string()) {
                return Err(source.error(
                    id.span().start,
                    ErrorKind::Schema,
                    "invalid or duplicate registry id",
                ));
            }
        }
    }
    Ok(result)
}

struct Row<'a> {
    cells: Vec<&'a str>,
    offsets: Vec<usize>,
}
fn csv<'a>(source: &'a Source, headers: &[&str]) -> Result<Vec<Row<'a>>, DataError> {
    let mut rows = Vec::new();
    let mut offset = 0;
    for (i, line) in source.text.split_inclusive('\n').enumerate() {
        let line = line.trim_end_matches(['\r', '\n']);
        let mut cells = Vec::new();
        let mut offsets = Vec::new();
        let mut cell_offset = offset;
        for cell in line.split(',') {
            offsets.push(cell_offset);
            cells.push(cell);
            cell_offset += cell.len() + 1;
        }
        if i == 0 {
            if !headers.contains(&line) {
                return Err(source.error(offset, ErrorKind::Schema, "invalid CSV header"));
            }
        } else if !line.is_empty() {
            let expected = source
                .text
                .lines()
                .next()
                .unwrap_or_default()
                .trim_end_matches('\r')
                .split(',')
                .count();
            if cells.len() != expected || cells.iter().any(|s| s.contains('"') || s.trim() != *s) {
                return Err(source.error(
                    offset,
                    ErrorKind::Schema,
                    "CSV requires exact unquoted columns",
                ));
            }
            rows.push(Row { cells, offsets });
        }
        // Source offsets retain CRLF, so later rows keep exact Unicode locations.
        offset += source.text[offset..]
            .split_inclusive('\n')
            .next()
            .unwrap_or_default()
            .len();
    }
    if source.text.is_empty() {
        return Err(source.error(0, ErrorKind::Schema, "missing CSV header"));
    }
    Ok(rows)
}
fn cell<T: std::str::FromStr>(
    source: &Source,
    row: &Row<'_>,
    i: usize,
    label: &str,
) -> Result<T, DataError> {
    row.cells[i].parse().map_err(|_| {
        source.error(
            row.offsets[i],
            ErrorKind::Schema,
            format!("invalid {label}"),
        )
    })
}
fn provinces(
    source: &Source,
    terrains: &BTreeSet<String>,
) -> Result<(Vec<ProvinceDefinition>, Vec<usize>), DataError> {
    let rows = csv(
        source,
        &[
            "id,r,g,b,kind,terrain,coastal",
            "id,r,g,b,kind,terrain,coastal,island",
        ],
    )?;
    let mut by_id = BTreeMap::new();
    let mut colors = BTreeSet::new();
    for row in rows {
        let id = cell(source, &row, 0, "province id (u16)")?;
        let rgb = [
            cell(source, &row, 1, "red (u8)")?,
            cell(source, &row, 2, "green (u8)")?,
            cell(source, &row, 3, "blue (u8)")?,
        ];
        let kind = match row.cells[4] {
            "land" => ProvinceKind::Land,
            "sea" => ProvinceKind::Sea,
            "lake" => ProvinceKind::Lake,
            _ => {
                return Err(source.error(
                    row.offsets[4],
                    ErrorKind::Schema,
                    "invalid province kind",
                ));
            }
        };
        let terrain = row.cells[5].to_owned();
        if !terrains.contains(&terrain) {
            return Err(source.error(row.offsets[5], ErrorKind::Schema, "unknown terrain"));
        }
        let coastal = cell(source, &row, 6, "coastal boolean")?;
        let island = if row.cells.len() == 8 {
            cell(source, &row, 7, "island boolean")?
        } else {
            false
        };
        if by_id.contains_key(&id) {
            return Err(source.error(row.offsets[0], ErrorKind::Schema, "duplicate province id"));
        }
        if !colors.insert(rgb) {
            return Err(source.error(row.offsets[1], ErrorKind::Schema, "duplicate province RGB"));
        }
        by_id.insert(
            id,
            (
                ProvinceDefinition {
                    id,
                    rgb,
                    kind,
                    terrain,
                    coastal,
                    island,
                },
                row.offsets[0],
            ),
        );
    }
    if by_id.is_empty() {
        return Err(source.error(0, ErrorKind::Schema, "map must define provinces"));
    }
    Ok(by_id.into_values().unzip())
}
fn bitmap(path: &Path) -> Result<(u32, u32, Vec<u8>), DataError> {
    let bytes = fs::read(path).map_err(|e| DataError {
        kind: ErrorKind::Io,
        ..error(path, e.to_string())
    })?;
    let mut reader = png::Decoder::new(Cursor::new(bytes))
        .read_info()
        .map_err(|e| error(path, format!("invalid PNG: {e}")))?;
    let info = reader.info();
    if info.color_type != png::ColorType::Rgb
        || info.bit_depth != png::BitDepth::Eight
        || info.animation_control.is_some()
    {
        return Err(error(path, "provinces.png must be static 24-bit RGB PNG"));
    }
    let (width, height) = (info.width, info.height);
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| error(path, "PNG dimensions overflow"))?;
    let mut data = vec![0; size];
    let output = reader
        .next_frame(&mut data)
        .map_err(|e| error(path, format!("invalid PNG: {e}")))?;
    data.truncate(output.buffer_size());
    if data.len()
        != (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(3))
            .ok_or_else(|| error(path, "PNG dimensions overflow"))?
    {
        return Err(error(path, "unexpected PNG pixel buffer size"));
    }
    // Consume through IEND as well: truncated/corrupt trailing chunks are errors.
    reader
        .finish()
        .map_err(|e| error(path, format!("invalid PNG: {e}")))?;
    Ok((width, height, data))
}

type Geometry = (Vec<[Fixed; 2]>, Vec<MapWarning>);
fn geometry(
    width: u32,
    height: u32,
    index: &[u16],
    provinces: &[ProvinceDefinition],
    source: &Source,
    offsets: &[usize],
    path: &Path,
) -> Result<Geometry, DataError> {
    let w = width as usize;
    let h = height as usize;
    let mut counts = vec![0_u64; provinces.len()];
    let mut sums = vec![[0_u128; 2]; provinces.len()];
    for (pixel, &i) in index.iter().enumerate() {
        counts[i as usize] += 1;
        sums[i as usize][0] += (pixel % w) as u128;
        sums[i as usize][1] += (pixel / w) as u128;
    }
    let mut visited = vec![false; index.len()];
    let mut components = vec![0_u64; provinces.len()];
    let mut pending = Vec::new();
    for start in 0..index.len() {
        if visited[start] {
            continue;
        }
        let i = index[start];
        components[i as usize] += 1;
        visited[start] = true;
        pending.push(start);
        while let Some(p) = pending.pop() {
            let (x, y) = (p % w, p / w);
            let neighbors = [
                if x > 0 { Some(p - 1) } else { None },
                if x + 1 < w { Some(p + 1) } else { None },
                if y > 0 { Some(p - w) } else { None },
                if y + 1 < h { Some(p + w) } else { None },
            ];
            for n in neighbors.into_iter().flatten() {
                if !visited[n] && index[n] == i {
                    visited[n] = true;
                    pending.push(n);
                }
            }
        }
    }
    let mut centers = Vec::with_capacity(provinces.len());
    let mut warnings = Vec::new();
    for (i, p) in provinces.iter().enumerate() {
        if counts[i] == 0 {
            return Err(source.error(
                offsets[i],
                ErrorKind::Schema,
                format!("province {} has no bitmap pixels", p.id),
            ));
        }
        if counts[i] == 1 {
            return Err(error(path, format!("1-pixel province {}", p.id)));
        }
        let mut center = [Fixed::ZERO; 2];
        for axis in 0..2 {
            let bits = ((sums[i][axis] << 32) / u128::from(counts[i])) + (1_u128 << 31);
            center[axis] = Fixed::from_bits(
                i64::try_from(bits).map_err(|_| error(path, "centroid overflow"))?,
            );
        }
        centers.push(center);
        if components[i] > 1 && !p.island {
            let location = source.error(offsets[i], ErrorKind::Schema, "");
            warnings.push(MapWarning {
                path: source.path.clone(),
                line: location.line,
                column: location.column,
                province: p.id,
                message: format!(
                    "province {} has {} disconnected components",
                    p.id, components[i]
                ),
            });
        }
    }
    Ok((centers, warnings))
}
fn adjacency(
    width: u32,
    height: u32,
    index: &[u16],
    provinces: &[ProvinceDefinition],
) -> BTreeMap<(u16, u16), EdgeKind> {
    let w = width as usize;
    let mut edges = BTreeMap::new();
    for (p, &a) in index.iter().enumerate() {
        for neighbor in [
            if p % w + 1 < w { Some(p + 1) } else { None },
            if p / w + 1 < (height as usize) {
                Some(p + w)
            } else {
                None
            },
        ]
        .into_iter()
        .flatten()
        {
            let b = index[neighbor];
            if a != b {
                edges.insert(
                    ordered(provinces[a as usize].id, provinces[b as usize].id),
                    EdgeKind::Normal,
                );
            }
        }
    }
    edges
}
fn overrides(
    source: &Source,
    provinces: &[ProvinceDefinition],
    edges: &mut BTreeMap<(u16, u16), EdgeKind>,
) -> Result<(), DataError> {
    let mut seen = BTreeSet::new();
    for row in csv(source, &["a,b,kind"])? {
        let a = cell(source, &row, 0, "endpoint a")?;
        let b = cell(source, &row, 1, "endpoint b")?;
        for (i, id) in [a, b].into_iter().enumerate() {
            if provinces.binary_search_by_key(&id, |p| p.id).is_err() {
                return Err(source.error(
                    row.offsets[i],
                    ErrorKind::Schema,
                    "unknown province in override",
                ));
            }
        }
        if a == b {
            return Err(source.error(row.offsets[0], ErrorKind::Schema, "self edge"));
        }
        let key = ordered(a, b);
        if !seen.insert(key) {
            return Err(source.error(row.offsets[0], ErrorKind::Schema, "duplicate override"));
        }
        let kind = match row.cells[2] {
            "river_small" => EdgeKind::RiverSmall,
            "river_large" => EdgeKind::RiverLarge,
            "strait" => EdgeKind::Strait,
            "impassable" => EdgeKind::Impassable,
            _ => {
                return Err(source.error(
                    row.offsets[2],
                    ErrorKind::Schema,
                    "invalid override kind",
                ));
            }
        };
        if kind != EdgeKind::Strait && !edges.contains_key(&key) {
            return Err(source.error(
                row.offsets[0],
                ErrorKind::Schema,
                "override requires bitmap adjacency (only strait adds an edge)",
            ));
        }
        edges.insert(key, kind);
    }
    Ok(())
}
fn scale(source: &Source) -> Result<Fixed, DataError> {
    let doc = source.document()?;
    for (key, _) in &doc {
        if key.get_ref() != "km_per_pixel" {
            return Err(source.error(key.span().start, ErrorKind::Schema, "unknown regions field"));
        }
    }
    let value = doc
        .get("km_per_pixel")
        .ok_or_else(|| source.error(0, ErrorKind::Schema, "missing km_per_pixel"))?;
    let fail = || {
        source.error(
            value.span().start,
            ErrorKind::Schema,
            "km_per_pixel must be a positive finite I32F32 number",
        )
    };
    if !matches!(value.get_ref(), Raw::Integer(_) | Raw::Float(_)) {
        return Err(fail());
    }
    let number = crate::parse_number(source, value).map_err(|_| fail())?;
    let fx = match number {
        crate::Number::Fixed(fx) => fx,
        crate::Number::Integer(i) => Fixed::checked_from_num(i).ok_or_else(fail)?,
    };
    if fx <= Fixed::ZERO {
        return Err(fail());
    }
    Ok(fx)
}
/// sqrt(dx^2+dy^2) on Q64.64 integer bits, floor to Q32.32, then
/// fixed's checked multiplication by scale. No float or platform libm.
fn distance(a: [Fixed; 2], b: [Fixed; 2], scale: Fixed) -> Option<Fixed> {
    let dx = (i128::from(a[0].to_bits()) - i128::from(b[0].to_bits())).unsigned_abs();
    let dy = (i128::from(a[1].to_bits()) - i128::from(b[1].to_bits())).unsigned_abs();
    let bits = dx
        .checked_mul(dx)?
        .checked_add(dy.checked_mul(dy)?)?
        .isqrt();
    Fixed::from_bits(i64::try_from(bits).ok()?).checked_mul(scale)
}
fn field_offset(doc: &DeTable<'_>, key: &str, index: usize, field: &str) -> usize {
    if let Some(list) = doc.get(key)
        && let Raw::Array(entries) = list.get_ref()
        && let Some(entry) = entries.get(index)
    {
        if let Raw::Table(fields) = entry.get_ref()
            && let Some(value) = fields.get(field)
        {
            return value.span().start;
        }
        return entry.span().start;
    }
    0
}
type StateResult = (Vec<StateDefinition>, Vec<VictoryPoint>, BTreeMap<u16, u16>);
fn states(
    source: &Source,
    provinces: &[ProvinceDefinition],
    resources: &BTreeSet<String>,
    buildings: &BTreeSet<String>,
) -> Result<StateResult, DataError> {
    let doc = source.document()?;
    let mut data: StatesDocument = toml::from_str(&source.text).map_err(|e| {
        source.error(
            e.span().map_or(0, |s| s.start),
            ErrorKind::Schema,
            e.message(),
        )
    })?;
    let mut ids = BTreeSet::new();
    let mut owners = BTreeMap::new();
    for (i, state) in data.state.iter_mut().enumerate() {
        let fail = |field: &str, message: &str| {
            source.error(
                field_offset(&doc, "state", i, field),
                ErrorKind::Schema,
                message,
            )
        };
        if !ids.insert(state.id) {
            return Err(fail("id", "duplicate state id"));
        }
        if state.name_key.is_empty() {
            return Err(fail("name_key", "state name_key must not be empty"));
        }
        if state.population < 0 {
            return Err(fail("population", "population must be nonnegative"));
        }
        if state.infrastructure < 0 {
            return Err(fail("infrastructure", "infrastructure must be nonnegative"));
        }
        if state.provinces.is_empty() {
            return Err(fail("provinces", "state must contain land provinces"));
        }
        state.provinces.sort_unstable();
        for &id in &state.provinces {
            let p = provinces
                .binary_search_by_key(&id, |p| p.id)
                .map_err(|_| fail("provinces", "unknown province in state"))?;
            if provinces[p].kind != ProvinceKind::Land {
                return Err(fail("provinces", "only land provinces belong to a state"));
            }
            if owners.insert(id, state.id).is_some() {
                return Err(fail(
                    "provinces",
                    "province appears in multiple states or repeatedly in one state",
                ));
            }
        }
        for (key, amount) in &state.resources {
            if !resources.contains(key) {
                return Err(fail("resources", "unknown resource"));
            }
            if *amount < 0 {
                return Err(fail("resources", "resource amount must be nonnegative"));
            }
        }
        for (key, level) in &state.buildings {
            if !buildings.contains(key) {
                return Err(fail("buildings", "unknown building"));
            }
            if *level < 0 {
                return Err(fail("buildings", "building level must be nonnegative"));
            }
        }
    }
    for p in provinces {
        if p.kind == ProvinceKind::Land && !owners.contains_key(&p.id) {
            return Err(source.error(
                0,
                ErrorKind::Schema,
                format!("land province {} must belong to exactly one state", p.id),
            ));
        }
    }
    let mut vp_ids = BTreeSet::new();
    for (i, vp) in data.victory_point.iter().enumerate() {
        let fail = |field: &str, message: &str| {
            source.error(
                field_offset(&doc, "victory_point", i, field),
                ErrorKind::Schema,
                message,
            )
        };
        provinces
            .binary_search_by_key(&vp.province, |p| p.id)
            .map_err(|_| fail("province", "unknown province in victory point"))?;
        if !vp_ids.insert(vp.province) {
            return Err(fail("province", "duplicate victory point"));
        }
        if vp.points <= 0 {
            return Err(fail("points", "victory point must be positive"));
        }
        if vp.name_key.is_empty() {
            return Err(fail("name_key", "victory point name_key must not be empty"));
        }
    }
    data.state.sort_by_key(|s| s.id);
    data.victory_point.sort_by_key(|v| v.province);
    Ok((data.state, data.victory_point, owners))
}
