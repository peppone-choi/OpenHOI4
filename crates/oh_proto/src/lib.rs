//! M0 MessagePack contract. Rust is the sole protocol type source.
use serde::{Deserialize, Serialize};
use ts_rs::TS;
pub const PROTOCOL_VERSION: &str = "m0-v1";
pub const MAP_DISPLAY_VERSION: u16 = 1;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum TimeCommand {
    Pause { paused: bool },
    SetSpeed { speed: u8 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Hello {
        protocol_version: String,
    },
    Create {
        scenario: String,
        seed: String,
        mode: String,
    },
    Join {
        session: String,
        nation: Option<String>,
    },
    Command {
        sequence: String,
        command: TimeCommand,
    },
    Query {
        request: String,
        kind: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PackInfo {
    pub id: String,
    pub version: String,
    pub hash: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct TimeState {
    /// Fully formatted by the server; browsers never calculate dates.
    pub date: String,
    pub hour: u8,
    /// Decimal text preserves full u64 precision in JavaScript.
    pub tick: String,
    pub paused: bool,
    pub speed: u8,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct NationFlagsView {
    pub nation: u16,
    pub keys: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum EndCauseView {
    Date,
    Condition,
    Explicit { source: String },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct EndView {
    pub tick: String,
    pub date: String,
    pub hour: u8,
    pub causes: Vec<EndCauseView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct TriggerView {
    pub definitions_hash: String,
    pub flags: Vec<NationFlagsView>,
    pub ended: Option<EndView>,
}
impl TriggerView {
    pub fn from_sim(sim: &oh_sim::Simulation) -> Option<Self> {
        let t = sim.trigger_state()?;
        Some(Self {
            definitions_hash: format!("{:016x}", t.definitions_hash),
            flags: t
                .flags
                .iter()
                .map(|(id, keys)| NationFlagsView {
                    nation: *id,
                    keys: keys.clone(),
                })
                .collect(),
            ended: t.ended.as_ref().map(|e| EndView {
                tick: e.tick.to_string(),
                date: format!("{:04}-{:02}-{:02}", e.date.year, e.date.month, e.date.day),
                hour: e.hour,
                causes: e
                    .causes
                    .iter()
                    .map(|c| match c {
                        oh_sim::trigger::EndCause::Date => EndCauseView::Date,
                        oh_sim::trigger::EndCause::Condition => EndCauseView::Condition,
                        oh_sim::trigger::EndCause::Explicit(source) => EndCauseView::Explicit {
                            source: source.clone(),
                        },
                    })
                    .collect(),
            }),
        })
    }
}
impl From<&oh_sim::State> for TimeState {
    fn from(state: &oh_sim::State) -> Self {
        Self {
            date: state.date().to_string(),
            hour: state.hour(),
            tick: state.tick().to_string(),
            paused: state.paused(),
            speed: state.speed(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
pub enum ServerMessage {
    TriggerResult {
        request: String,
        supported: bool,
        reason_key: Option<String>,
        trigger: Option<TriggerView>,
    },
    Welcome {
        engine_version: String,
        protocol_version: String,
        accepted: bool,
        reason_key: Option<String>,
        packs: Vec<PackInfo>,
        sessions: Vec<String>,
    },
    CommandResult {
        sequence: String,
        accepted: bool,
        reason_key: Option<String>,
    },
    Snapshot {
        state: TimeState,
    },
    Delta {
        sequence: String,
        state: TimeState,
    },
    QueryResult {
        request: String,
        supported: bool,
        reason_key: Option<String>,
        state: Option<TimeState>,
    },
    WorldResult {
        request: String,
        supported: bool,
        reason_key: Option<String>,
        world: Option<WorldView>,
    },
    Notice {
        key: String,
    },
}
pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    rmp_serde::to_vec_named(message)
}
pub fn decode_client(bytes: &[u8]) -> Result<ClientMessage, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}
pub fn decode_server(bytes: &[u8]) -> Result<ServerMessage, rmp_serde::decode::Error> {
    rmp_serde::from_slice(bytes)
}
pub fn typescript() -> String {
    let mut out = format!(
        "// Generated by oh_proto; do not edit.\nexport const PROTOCOL_VERSION = {PROTOCOL_VERSION:?} as const;\nexport const MAP_DISPLAY_VERSION = {MAP_DISPLAY_VERSION} as const;\n"
    );
    for declaration in [
        TimeCommand::decl(&ts_rs::Config::default()),
        PackInfo::decl(&ts_rs::Config::default()),
        TimeState::decl(&ts_rs::Config::default()),
        NationFlagsView::decl(&ts_rs::Config::default()),
        EndCauseView::decl(&ts_rs::Config::default()),
        EndView::decl(&ts_rs::Config::default()),
        TriggerView::decl(&ts_rs::Config::default()),
        LedgerRow::decl(&ts_rs::Config::default()),
        LedgerView::decl(&ts_rs::Config::default()),
        SupportView::decl(&ts_rs::Config::default()),
        NationView::decl(&ts_rs::Config::default()),
        ScalarView::decl(&ts_rs::Config::default()),
        StateView::decl(&ts_rs::Config::default()),
        ProvinceView::decl(&ts_rs::Config::default()),
        WorldView::decl(&ts_rs::Config::default()),
        MapStyle::decl(&ts_rs::Config::default()),
        MapMetadata::decl(&ts_rs::Config::default()),
        ClientMessage::decl(&ts_rs::Config::default()),
        ServerMessage::decl(&ts_rs::Config::default()),
    ] {
        out.push_str("export ");
        for line in declaration.lines() {
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LedgerRow {
    pub id: String,
    pub label_key: String,
    pub operation_key: String,
    pub value: String,
    pub accumulated: String,
    pub source_key: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct LedgerView {
    pub base: String,
    pub final_value: String,
    pub tick: String,
    pub entries: Vec<LedgerRow>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SupportView {
    pub name_key: String,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct NationView {
    pub id: u16,
    pub tag: String,
    pub name_key: String,
    pub color: [u8; 3],
    pub capital: u16,
    pub government_key: String,
    pub support: Vec<SupportView>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ScalarView {
    pub name_key: String,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct StateView {
    pub id: u16,
    pub name_key: String,
    pub provinces: Vec<u16>,
    pub owner: u16,
    pub population: String,
    pub resources: Vec<ScalarView>,
    pub buildings: Vec<ScalarView>,
    pub infrastructure: LedgerView,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ProvinceView {
    pub id: u16,
    pub state: Option<u16>,
    pub owner: Option<u16>,
    pub controller: Option<u16>,
    pub owner_color: Option<[u8; 3]>,
    pub controller_color: Option<[u8; 3]>,
    pub terrain_key: String,
    pub terrain_color: [u8; 3],
    pub state_color: Option<[u8; 3]>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct WorldView {
    pub tick: String,
    pub map_id: String,
    pub width: u32,
    pub height: u32,
    pub province_ids: Vec<u16>,
    pub neutral_color: [u8; 3],
    pub mode_keys: Vec<String>,
    pub nations: Vec<NationView>,
    pub states: Vec<StateView>,
    pub provinces: Vec<ProvinceView>,
}
/// Presentation settings only, loaded by the host from defines.toml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct MapStyle {
    pub background: [u8; 3],
    pub nation_border: [u8; 3],
    pub state_border: [u8; 3],
    pub province_border: [u8; 3],
    pub selected: [u8; 3],
    pub hovered: [u8; 3],
    pub nation_width_milli: u32,
    pub state_width_milli: u32,
    pub province_width_milli: u32,
    pub highlight_milli: u32,
    pub fit_milli: u32,
    pub zoom_min_milli: u32,
    pub zoom_max_milli: u32,
    pub wheel_milli: u32,
    pub drag_threshold: u32,
}
/// HTTP map contract. RG8 bytes are dense u16 LE, row-major from top-left.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct MapMetadata {
    pub schema_version: u16,
    pub map_id: String,
    pub width: u32,
    pub height: u32,
    pub province_ids: Vec<u16>,
    pub pack_hash: String,
    pub index_hash: String,
    pub byte_length: String,
    pub style: MapStyle,
}
impl WorldView {
    pub fn from_sim(sim: &oh_sim::Simulation) -> Option<Self> {
        let w = sim.world()?;
        let nations = w
            .inputs()
            .nations()
            .iter()
            .map(|n| {
                let d = w
                    .defs()
                    .nations()
                    .iter()
                    .find(|d| d.id == n.id().get())
                    .expect("validated nation");
                NationView {
                    id: n.id().get(),
                    tag: n.tag().into(),
                    name_key: d.name_key.clone(),
                    color: d.color,
                    capital: d.capital,
                    government_key: n.government().into(),
                    support: n
                        .support()
                        .iter()
                        .map(|(k, v)| SupportView {
                            name_key: k.clone(),
                            value: v.to_string(),
                        })
                        .collect(),
                }
            })
            .collect::<Vec<_>>();
        let color = |id: Option<oh_core::NationId>| {
            id.and_then(|id| nations.iter().find(|n| n.id == id.get()).map(|n| n.color))
        };
        let provinces = w
            .inputs()
            .provinces()
            .iter()
            .map(|p| ProvinceView {
                id: p.id().get(),
                state: p.state().map(|i| i.get()),
                owner: p.owner().map(|i| i.get()),
                controller: p.controller().map(|i| i.get()),
                owner_color: color(p.owner()),
                controller_color: color(p.controller()),
                terrain_key: w
                    .defs()
                    .map()
                    .provinces
                    .iter()
                    .find(|d| d.id == p.id().get())
                    .expect("validated province")
                    .terrain
                    .clone(),
                terrain_color: w.defs().visuals().terrain[&w
                    .defs()
                    .map()
                    .provinces
                    .iter()
                    .find(|d| d.id == p.id().get())
                    .expect("validated province")
                    .terrain],
                state_color: p.state().map(|id| w.defs().visuals().state[&id.get()]),
            })
            .collect();
        let states = w
            .inputs()
            .states()
            .iter()
            .map(|s| {
                let d = w
                    .defs()
                    .map()
                    .states
                    .iter()
                    .find(|d| d.id == s.id().get())
                    .expect("validated state");
                s.ledger()
                    .verify_applied_value(s.infrastructure())
                    .expect("applied ledger invariant");
                let entries = s
                    .ledger()
                    .entries()
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let operation_key = match e.op {
                            oh_sim::ledger::LedgerOp::Base => "ledger-base",
                            oh_sim::ledger::LedgerOp::Add => "ledger-add",
                            oh_sim::ledger::LedgerOp::Mul => "ledger-multiply",
                        };
                        LedgerRow {
                            id: i.to_string(),
                            label_key: e.source.clone().unwrap_or("infrastructure".into()),
                            operation_key: operation_key.into(),
                            value: e.value.to_string(),
                            accumulated: e.accumulated.to_string(),
                            source_key: e.source.clone(),
                        }
                    })
                    .collect();
                StateView {
                    id: s.id().get(),
                    name_key: d.name_key.clone(),
                    provinces: d.provinces.clone(),
                    owner: s.owner().get(),
                    population: s.population().to_string(),
                    resources: s
                        .resources()
                        .iter()
                        .map(|(k, v)| ScalarView {
                            name_key: format!("resource-{k}"),
                            value: v.to_string(),
                        })
                        .collect(),
                    buildings: s
                        .buildings()
                        .iter()
                        .map(|(k, v)| ScalarView {
                            name_key: format!("building-{k}"),
                            value: v.to_string(),
                        })
                        .collect(),
                    infrastructure: LedgerView {
                        base: s.base().to_string(),
                        final_value: s.infrastructure().to_string(),
                        tick: s.ledger().tick().to_string(),
                        entries,
                    },
                }
            })
            .collect();
        Some(Self {
            tick: sim.snapshot().tick().to_string(),
            map_id: w.defs().map_id().into(),
            width: w.defs().map().width,
            height: w.defs().map().height,
            neutral_color: w.defs().visuals().neutral,
            mode_keys: [
                "map-mode-owner",
                "map-mode-control",
                "map-mode-terrain",
                "map-mode-state",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            province_ids: w.defs().map().provinces.iter().map(|p| p.id).collect(),
            nations,
            states,
            provinces,
        })
    }
}
