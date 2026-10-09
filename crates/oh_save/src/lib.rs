//! Host save I/O. Untrusted bytes never deserialize directly into live authority.
mod bounds;
mod context;
mod file;
pub mod repro;
pub mod repro_zip;
pub use context::{SaveContext, effective_defines_hash, pack_hash};
pub use file::{SaveOutcome, read_file, write_atomic};
use oh_sim::{
    Simulation,
    save_state::{DateV1, SimulationSaveV1, SimulationSaveV2, SimulationSaveV3},
};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
pub type Result<T> = std::result::Result<T, String>;
pub const FORMAT_VERSION: u16 = 1;
/// Additive format; legacy writers still produce version 1.
pub const MOVEMENT_FORMAT_VERSION: u16 = 2;
pub const STRAIT_FORMAT_VERSION: u16 = 3;
pub const TRIGGER_FORMAT_VERSION: u16 = 4;
pub const ECONOMY_FORMAT_VERSION: u16 = 5;
pub const PRODUCTION_FORMAT_VERSION: u16 = 6;
pub const MILITARY_FORMAT_VERSION: u16 = 7;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackV1 {
    pub id: String,
    pub version: String,
    pub content_hash: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HeaderV1 {
    pub engine_version: String,
    pub format_version: u16,
    pub scenario_id: String,
    pub packs: Vec<PackV1>,
    pub game_date: DateV1,
    pub tick: u64,
    pub seed: u64,
    pub state_hash: u64,
    pub player_nations: Vec<u16>,
    pub saved_at_utc: i64,
    pub definitions_hash: Option<u64>,
    pub effective_defines_hash: u64,
}
pub struct LoadedSave {
    pub header: HeaderV1,
    pub simulation: Simulation,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub header_max_bytes: u64,
    pub file_max_bytes: u64,
    pub body_max_bytes: u64,
    pub string_max_bytes: u64,
    pub queue_max_entries: u64,
    pub modifiers_per_state: u64,
    pub map_entries_max: u64,
    pub packs_max: u64,
    pub zstd_window_log_max: u32,
    pub compression_level: i32,
    pub saved_at_max: i64,
    pub allocation_budget_bytes: u64,
    pub entry_allocation_charge_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        #[derive(Deserialize)]
        struct Policy {
            save: Limits,
        }
        toml::from_str::<Policy>(include_str!("../defines.toml"))
            .expect("bundled save policy")
            .save
    }
}
fn serialize<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    oh_core::canonical_bytes(v).map_err(|e| format!("Postcard: {e}"))
}
fn deserialize<T: serde::de::DeserializeOwned + Serialize>(b: &[u8]) -> Result<T> {
    let v: T = oh_core::from_canonical_bytes(b).map_err(|e| format!("Postcard: {e}"))?;
    if serialize(&v)? != b {
        return Err("NonCanonicalPostcard".into());
    }
    Ok(v)
}
pub fn encode(sim: &Simulation, c: &SaveContext, time: i64, players: Vec<u16>) -> Result<Vec<u8>> {
    encode_with_limits(sim, c, time, players, &Limits::default())
}
pub fn encode_with_limits(
    sim: &Simulation,
    context: &SaveContext,
    saved_at_utc: i64,
    player_nations: Vec<u16>,
    limits: &Limits,
) -> Result<Vec<u8>> {
    context.verify_unchanged()?;
    let version = if sim.military().is_some() {
        MILITARY_FORMAT_VERSION
    } else if sim.production().is_some() {
        PRODUCTION_FORMAT_VERSION
    } else if sim.economy().is_some() {
        ECONOMY_FORMAT_VERSION
    } else if sim.trigger_state().is_some() {
        TRIGGER_FORMAT_VERSION
    } else if sim.movement().is_some_and(|m| m.has_strait_context()) {
        STRAIT_FORMAT_VERSION
    } else if sim.movement().is_some() {
        MOVEMENT_FORMAT_VERSION
    } else {
        FORMAT_VERSION
    };
    let (body, candidate) = if version == MILITARY_FORMAT_VERSION {
        let dto = sim.export_save_v7()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v7(dto, &context.restore)?,
        )
    } else if version == PRODUCTION_FORMAT_VERSION {
        let dto = sim.export_save_v6()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v6(dto, &context.restore)?,
        )
    } else if version == ECONOMY_FORMAT_VERSION {
        let dto = sim.export_save_v5()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v5(dto, &context.restore)?,
        )
    } else if version == TRIGGER_FORMAT_VERSION {
        let dto = sim.export_save_v4()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v4(dto, &context.restore)?,
        )
    } else if version == STRAIT_FORMAT_VERSION {
        let dto = sim.export_save_v3()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v3(dto, &context.restore)?,
        )
    } else if version == MOVEMENT_FORMAT_VERSION {
        let dto = sim.export_save_v2()?;
        (
            serialize(&dto)?,
            Simulation::from_save_v2(dto, &context.restore)?,
        )
    } else {
        let dto = sim.export_save()?;
        (
            serialize(&dto)?,
            Simulation::from_save(dto, &context.restore)?,
        )
    };
    if candidate.state_hash().map_err(|e| e.to_string())?
        != sim.state_hash().map_err(|e| e.to_string())?
    {
        return Err("StateHashMismatch: export".into());
    }
    let s = sim.snapshot();
    let header = HeaderV1 {
        engine_version: env!("CARGO_PKG_VERSION").into(),
        format_version: version,
        scenario_id: s.scenario().into(),
        packs: vec![context.pack.clone()],
        game_date: s.date().into(),
        tick: s.tick(),
        seed: s.seed(),
        state_hash: sim.state_hash().map_err(|e| e.to_string())?,
        player_nations,
        saved_at_utc,
        definitions_hash: sim.world().map(|w| w.definitions_hash()),
        effective_defines_hash: context.defines_hash,
    };
    check_header(&header, context, limits, false)?;
    check_players(&header, &candidate)?;
    let header_bytes = serialize(&header)?;
    bounds::header(&header_bytes, limits)?;
    if version == FORMAT_VERSION {
        bounds::body(&body, limits)?;
    } else {
        bounds::body_version(&body, limits, version)?;
    }
    let mut compressor = zstd::stream::write::Encoder::new(Vec::new(), limits.compression_level)
        .map_err(|e| format!("Compression: {e}"))?;
    compressor
        .include_checksum(true)
        .map_err(|e| e.to_string())?;
    compressor
        .window_log(limits.zstd_window_log_max)
        .map_err(|e| e.to_string())?;
    compressor
        .write_all(&body)
        .map_err(|e| format!("Compression: {e}"))?;
    let compressed = compressor
        .finish()
        .map_err(|e| format!("Compression: {e}"))?;
    let length = u32::try_from(header_bytes.len()).map_err(|_| "Limit: header u32")?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"OHSV");
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.extend(header_bytes);
    bytes.extend(compressed);
    if bytes.len() as u64 > limits.file_max_bytes {
        return Err("Limit: encoded file".into());
    }
    Ok(bytes)
}
pub fn decode(bytes: &[u8], c: &SaveContext, force: bool) -> Result<LoadedSave> {
    decode_with_limits(bytes, c, force, &Limits::default())
}
/// Bounded metadata inspection for choosing a local scenario, not activation.
pub fn inspect_header(bytes: &[u8], limits: &Limits) -> Result<HeaderV1> {
    if bytes.len() as u64 > limits.file_max_bytes {
        return Err("Limit: file".into());
    }
    if bytes.len() < 10 || &bytes[..4] != b"OHSV" {
        return Err("BadMagic/Truncated: prefix".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if ![
        FORMAT_VERSION,
        MOVEMENT_FORMAT_VERSION,
        STRAIT_FORMAT_VERSION,
        TRIGGER_FORMAT_VERSION,
        ECONOMY_FORMAT_VERSION,
        PRODUCTION_FORMAT_VERSION,
        MILITARY_FORMAT_VERSION,
    ]
    .contains(&version)
    {
        return Err(format!(
            "UnsupportedFormat: file v{version}, supported v{FORMAT_VERSION}/v{MOVEMENT_FORMAT_VERSION}/v{STRAIT_FORMAT_VERSION}"
        ));
    }
    let length =
        u32::from_le_bytes(bytes[6..10].try_into().map_err(|_| "Truncated: length")?) as u64;
    if length == 0 || length > limits.header_max_bytes {
        return Err("Limit: header length".into());
    }
    let end = 10usize
        .checked_add(usize::try_from(length).map_err(|_| "Limit: header usize")?)
        .ok_or("Limit: offset")?;
    let h = bytes.get(10..end).ok_or("Truncated: header")?;
    bounds::header(h, limits)?;
    let header: HeaderV1 = deserialize(h)?;
    if header.format_version != version {
        return Err("UnsupportedFormat: prefix/header version mismatch".into());
    }
    Ok(header)
}
pub fn decode_with_limits(
    bytes: &[u8],
    context: &SaveContext,
    force: bool,
    limits: &Limits,
) -> Result<LoadedSave> {
    if bytes.len() as u64 > limits.file_max_bytes {
        return Err("Limit: file".into());
    }
    if bytes.len() < 10 {
        return Err("Truncated: prefix".into());
    }
    if &bytes[..4] != b"OHSV" {
        return Err("BadMagic".into());
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if ![
        FORMAT_VERSION,
        MOVEMENT_FORMAT_VERSION,
        STRAIT_FORMAT_VERSION,
        TRIGGER_FORMAT_VERSION,
        ECONOMY_FORMAT_VERSION,
        PRODUCTION_FORMAT_VERSION,
        MILITARY_FORMAT_VERSION,
    ]
    .contains(&version)
    {
        return Err(format!(
            "UnsupportedFormat: file v{version}, supported v{FORMAT_VERSION}/v{MOVEMENT_FORMAT_VERSION}/v{STRAIT_FORMAT_VERSION}"
        ));
    }
    let length =
        u32::from_le_bytes(bytes[6..10].try_into().map_err(|_| "Truncated: length")?) as u64;
    if length == 0 || length > limits.header_max_bytes {
        return Err("Limit: header length".into());
    }
    let end = 10usize
        .checked_add(usize::try_from(length).map_err(|_| "Limit: header usize")?)
        .ok_or("Limit: offset")?;
    let h = bytes.get(10..end).ok_or("Truncated: header")?;
    bounds::header(h, limits)?;
    let header: HeaderV1 = deserialize(h)?;
    if header.format_version != version {
        return Err("UnsupportedFormat: prefix/header version mismatch".into());
    }
    let warnings = check_header(&header, context, limits, force)?;
    let compressed = bytes.get(end..).ok_or("Truncated: body")?;
    if !compressed.starts_with(&[0x28, 0xb5, 0x2f, 0xfd]) {
        return Err("Compression: standard frame required".into());
    }
    let mut decoder = zstd::stream::read::Decoder::with_buffer(compressed)
        .map_err(|e| format!("Compression: {e}"))?
        .single_frame();
    decoder
        .window_log_max(limits.zstd_window_log_max)
        .map_err(|e| format!("Compression: {e}"))?;
    let mut body = Vec::new();
    let mut block = [0u8; 8192]; // I/O chunk size, not policy/game coefficient.
    loop {
        let count = decoder
            .read(&mut block)
            .map_err(|e| format!("Compression: {e}"))?;
        if count == 0 {
            break;
        }
        if (body.len() as u64)
            .checked_add(count as u64)
            .is_none_or(|n| n > limits.body_max_bytes)
        {
            return Err("Limit: decompressed body".into());
        }
        body.extend_from_slice(&block[..count]);
    }
    if !decoder.finish().is_empty() {
        return Err("Trailing: compressed frame".into());
    }
    if version == FORMAT_VERSION {
        bounds::body(&body, limits)?;
    } else {
        bounds::body_version(&body, limits, version)?;
    }
    let (base, dto2, dto3, dto4, dto5, dto6, dto7) = if version == MILITARY_FORMAT_VERSION {
        let dto: oh_sim::military_save::SimulationSaveV7 = deserialize(&body)?;
        (
            dto.base.base.base.base.base.clone(),
            None,
            None,
            None,
            None,
            None,
            Some(dto),
        )
    } else if version == PRODUCTION_FORMAT_VERSION {
        let dto: oh_sim::production_save::SimulationSaveV6 = deserialize(&body)?;
        (
            dto.base.base.base.base.clone(),
            None,
            None,
            None,
            None,
            Some(dto),
            None,
        )
    } else if version == ECONOMY_FORMAT_VERSION {
        let dto: oh_sim::economy_save::SimulationSaveV5 = deserialize(&body)?;
        (
            dto.base.base.base.clone(),
            None,
            None,
            None,
            Some(dto),
            None,
            None,
        )
    } else if version == TRIGGER_FORMAT_VERSION {
        let dto: oh_sim::trigger_save::SimulationSaveV4 = deserialize(&body)?;
        (
            dto.base.base.base.clone(),
            None,
            None,
            Some(dto),
            None,
            None,
            None,
        )
    } else if version == STRAIT_FORMAT_VERSION {
        let dto: SimulationSaveV3 = deserialize(&body)?;
        (
            dto.base.base.clone(),
            None,
            Some(dto),
            None,
            None,
            None,
            None,
        )
    } else if version == MOVEMENT_FORMAT_VERSION {
        let dto: SimulationSaveV2 = deserialize(&body)?;
        (dto.base.clone(), Some(dto), None, None, None, None, None)
    } else {
        (
            deserialize::<SimulationSaveV1>(&body)?,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    };
    if header.scenario_id != base.state.scenario
        || header.game_date != base.state.date
        || header.tick != base.state.tick
        || header.seed != base.state.seed
        || header.definitions_hash != base.world.as_ref().map(|w| w.definitions_hash)
    {
        return Err("MetadataMismatch: header/body".into());
    }
    context.verify_unchanged()?;
    let simulation = if let Some(dto) = dto7 {
        Simulation::from_save_v7(dto, &context.restore)?
    } else if let Some(dto) = dto6 {
        Simulation::from_save_v6(dto, &context.restore)?
    } else if let Some(dto) = dto5 {
        Simulation::from_save_v5(dto, &context.restore)?
    } else if let Some(dto) = dto4 {
        Simulation::from_save_v4(dto, &context.restore)?
    } else if let Some(dto) = dto3 {
        Simulation::from_save_v3(dto, &context.restore)?
    } else if let Some(dto) = dto2 {
        Simulation::from_save_v2(dto, &context.restore)?
    } else {
        Simulation::from_save(base, &context.restore)?
    };
    check_players(&header, &simulation)?;
    if simulation.state_hash().map_err(|e| e.to_string())? != header.state_hash {
        return Err("StateHashMismatch".into());
    }
    Ok(LoadedSave {
        header,
        simulation,
        warnings,
    })
}
fn check_header(h: &HeaderV1, c: &SaveContext, l: &Limits, force: bool) -> Result<Vec<String>> {
    let has_military = c
        .restore
        .world
        .as_ref()
        .is_some_and(|w| w.defs().military().is_some());
    if (h.format_version == MILITARY_FORMAT_VERSION) != has_military {
        return Err("MilitaryModeMismatch: save/local definitions".into());
    }
    let has_production = c
        .restore
        .world
        .as_ref()
        .is_some_and(|w| w.defs().production().is_some());
    if (matches!(
        h.format_version,
        PRODUCTION_FORMAT_VERSION | MILITARY_FORMAT_VERSION
    )) != has_production
    {
        return Err("ProductionModeMismatch: save/local definitions".into());
    }
    let has_trigger = c
        .restore
        .world
        .as_ref()
        .is_some_and(|w| w.defs().trigger().is_some());
    let has_economy = c
        .restore
        .world
        .as_ref()
        .is_some_and(|w| w.defs().economy().is_some());
    if (matches!(
        h.format_version,
        ECONOMY_FORMAT_VERSION | PRODUCTION_FORMAT_VERSION | MILITARY_FORMAT_VERSION
    )) != has_economy
    {
        return Err("EconomyModeMismatch: save/local definitions".into());
    }
    if !matches!(
        h.format_version,
        ECONOMY_FORMAT_VERSION | PRODUCTION_FORMAT_VERSION | MILITARY_FORMAT_VERSION
    ) && (h.format_version == TRIGGER_FORMAT_VERSION) != has_trigger
    {
        return Err("TriggerModeMismatch: save/local definitions".into());
    }
    if ![
        FORMAT_VERSION,
        MOVEMENT_FORMAT_VERSION,
        STRAIT_FORMAT_VERSION,
        TRIGGER_FORMAT_VERSION,
        ECONOMY_FORMAT_VERSION,
        PRODUCTION_FORMAT_VERSION,
        MILITARY_FORMAT_VERSION,
    ]
    .contains(&h.format_version)
    {
        return Err("UnsupportedFormat: prefix/header version mismatch".into());
    }
    semver::Version::parse(&h.engine_version).map_err(|_| "InvalidHeader: engine version")?;
    if h.engine_version != env!("CARGO_PKG_VERSION") {
        return Err(format!("UnsupportedEngine: {}", h.engine_version));
    }
    h.game_date.validate()?;
    if h.saved_at_utc < 0 || h.saved_at_utc > l.saved_at_max {
        return Err("InvalidHeader: saved_at_utc".into());
    }
    if h.scenario_id != c.restore.scenario || h.packs.len() != 1 || h.packs[0].id != c.pack.id {
        return Err("PackMismatch: id/list/scenario".into());
    }
    semver::Version::parse(&h.packs[0].version).map_err(|_| "InvalidHeader: pack version")?;
    if h.definitions_hash != c.restore.world.as_ref().map(|w| w.definitions_hash())
        || h.effective_defines_hash != c.defines_hash
    {
        return Err("DefinitionsMismatch: definitions/defines".into());
    }
    let mut warnings = Vec::new();
    if h.packs[0] != c.pack {
        let message = format!(
            "PackMismatch: stored {:?}; current {:?}",
            h.packs[0], c.pack
        );
        if !force {
            return Err(message);
        }
        warnings.push(message);
    }
    Ok(warnings)
}
fn check_players(h: &HeaderV1, s: &Simulation) -> Result<()> {
    if !h.player_nations.windows(2).all(|w| w[0] < w[1])
        || h.player_nations.iter().any(|id| {
            s.world()
                .is_none_or(|w| w.nation(oh_core::NationId(*id)).is_none())
        })
    {
        return Err("InvalidReference: player_nations".into());
    }
    Ok(())
}
