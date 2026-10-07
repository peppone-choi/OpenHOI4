//! Replay Host contracts. Simulation remains unchanged and owns every game rule.
use crate::{
    Result, SaveContext,
    repro_zip::{self, Policy},
};
use oh_core::{DivisionId, NationId, ProvinceId};
use oh_sim::{Command, Simulation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum InputCommand {
    Pause(bool),
    SetSpeed(u8),
    Move { unit: u32, destination: u16 },
    Stop { unit: u32 },
    Effects { program: String },
}
impl From<InputCommand> for Command {
    fn from(c: InputCommand) -> Self {
        match c {
            InputCommand::Pause(v) => Self::Pause(v),
            InputCommand::SetSpeed(v) => Self::SetSpeed(v),
            InputCommand::Move { unit, destination } => Self::Move {
                unit: DivisionId(unit),
                destination: ProvinceId(destination),
            },
            InputCommand::Stop { unit } => Self::Stop {
                unit: DivisionId(unit),
            },
            InputCommand::Effects { program } => Self::Effects { program },
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
pub enum Input {
    Enqueue {
        tick: u64,
        nation: u16,
        sequence: u64,
        command: InputCommand,
    },
    Pump {},
    Step {},
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applied {
    pub nation: u16,
    pub sequence: u64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub error: Option<String>,
    pub advanced: bool,
    #[serde(deserialize_with = "bounded_commands")]
    pub commands: Vec<Applied>,
    #[serde(deserialize_with = "bounded_phases")]
    pub phases: Vec<String>,
}
fn bounded<'de, T: Deserialize<'de>, D: serde::Deserializer<'de>>(
    deserializer: D,
    max: usize,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Visitor<T> {
        max: usize,
        marker: std::marker::PhantomData<T>,
    }
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded replay array")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut items = Vec::new();
            loop {
                if items.len() == self.max {
                    if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                        return Err(serde::de::Error::custom("Repro: array limit"));
                    }
                    return Ok(items);
                }
                match seq.next_element()? {
                    Some(item) => items.push(item),
                    None => return Ok(items),
                }
            }
        }
    }
    deserializer.deserialize_seq(Visitor {
        max,
        marker: std::marker::PhantomData,
    })
}
fn bounded_commands<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Applied>, D::Error> {
    bounded(d, crate::Limits::default().queue_max_entries as usize)
}
fn bounded_phases<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    bounded(d, Policy::default().phases_per_event_max)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub index: usize,
    pub before_tick: u64,
    pub input: Input,
    pub outcome: Outcome,
    pub after_hash: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub format: u16,
    pub dto: serde_json::Value,
    pub canonical_hex: String,
    pub hash: String,
    pub ended: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub schema: u16,
    pub engine_version: String,
    pub scenario: String,
    pub national: bool,
    pub pack_id: String,
    pub pack_version: String,
    pub pack_hash: String,
    pub effective_defines_hash: String,
    pub seed: u64,
    pub start_save: bool,
    pub start_tick: u64,
    pub start_hash: String,
    pub end_tick: u64,
    pub expected_hash: String,
    pub events: usize,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hash(sim: &Simulation) -> Result<String> {
    Ok(format!(
        "{:016x}",
        sim.state_hash().map_err(|e| e.to_string())?
    ))
}
pub fn report(sim: &Simulation) -> Result<Report> {
    let (format, dto) = if sim.trigger_state().is_some() {
        (4, serde_json::to_value(sim.export_save_v4()?))
    } else if sim.movement().is_some_and(|m| m.has_strait_context()) {
        (3, serde_json::to_value(sim.export_save_v3()?))
    } else if sim.movement().is_some() {
        (2, serde_json::to_value(sim.export_save_v2()?))
    } else {
        (1, serde_json::to_value(sim.export_save()?))
    };
    Ok(Report {
        format,
        dto: dto.map_err(|e| e.to_string())?,
        canonical_hex: hex(&oh_core::canonical_bytes(sim).map_err(|e| e.to_string())?),
        hash: hash(sim)?,
        ended: sim.is_ended(),
    })
}
/// No preview mutation: logs the actual enqueue/step result, including rollback.
pub fn apply(sim: &mut Simulation, input: Input, index: usize) -> Result<Event> {
    let before_tick = sim.snapshot().tick();
    let mut outcome = Outcome {
        error: None,
        advanced: false,
        commands: vec![],
        phases: vec![],
    };
    match &input {
        Input::Enqueue {
            tick,
            nation,
            sequence,
            command,
        } => {
            outcome.error = sim
                .enqueue(*tick, NationId(*nation), *sequence, command.clone().into())
                .err()
                .map(|e| format!("{e:?}"));
        }
        Input::Pump {} | Input::Step {} => match sim.step() {
            Ok(step) => {
                outcome.advanced = step.advanced;
                outcome.commands = step
                    .commands
                    .into_iter()
                    .map(|c| Applied {
                        nation: c.nation.0,
                        sequence: c.sequence,
                        error: c.result.err().map(|e| format!("{e:?}")),
                    })
                    .collect();
                outcome.phases = step.phases.iter().map(|p| format!("{p:?}")).collect();
            }
            Err(e) => outcome.error = Some(format!("{e:?}")),
        },
    }
    Ok(Event {
        index,
        before_tick,
        input,
        outcome,
        after_hash: hash(sim)?,
    })
}
pub fn human(events: &[Event]) -> Result<Vec<u8>> {
    let mut b = String::new();
    for e in events {
        b.push_str(&format!(
            "{} tick={} {} => {} hash={}\n",
            e.index,
            e.before_tick,
            serde_json::to_string(&e.input).map_err(|e| e.to_string())?,
            serde_json::to_string(&e.outcome).map_err(|e| e.to_string())?,
            e.after_hash
        ));
    }
    Ok(b.into_bytes())
}
pub struct Recorder {
    start: Vec<u8>,
    metadata: Bundle,
    events: Vec<Event>,
    log_bytes: u64,
}
impl Recorder {
    pub fn new(sim: &Simulation, c: &SaveContext) -> Result<Self> {
        let s = sim.snapshot();
        Ok(Self {
            start: crate::encode(sim, c, 0, vec![])?,
            metadata: Bundle {
                schema: 1,
                engine_version: env!("CARGO_PKG_VERSION").into(),
                scenario: s.scenario().into(),
                national: sim.world().is_some(),
                pack_id: c.pack().id.clone(),
                pack_version: c.pack().version.clone(),
                pack_hash: format!("{:016x}", c.pack().content_hash),
                effective_defines_hash: format!("{:016x}", c.defines_hash()),
                seed: s.seed(),
                start_save: true,
                start_tick: s.tick(),
                start_hash: hash(sim)?,
                end_tick: s.tick(),
                expected_hash: hash(sim)?,
                events: 0,
            },
            events: vec![],
            log_bytes: 0,
        })
    }
    pub fn input(&mut self, sim: &mut Simulation, input: Input) -> Result<Event> {
        let p = Policy::default();
        if self.events.len() >= p.events_max {
            return Err("Repro: event limit".into());
        }
        // Apply in a candidate so host allocation/budget failures preserve authority.
        let mut candidate = sim.clone();
        let e = apply(&mut candidate, input, self.events.len())?;
        let charge = serde_json::to_vec(&e).map_err(|e| e.to_string())?.len() as u64;
        let total = self
            .log_bytes
            .checked_add(charge)
            .ok_or("Repro: log overflow")?;
        if total > p.member_max_bytes {
            return Err("Repro: log limit".into());
        }
        self.events.push(e.clone());
        self.log_bytes = total;
        *sim = candidate;
        Ok(e)
    }
    pub fn finish(mut self, sim: &Simulation, c: &SaveContext) -> Result<Vec<u8>> {
        c.verify_unchanged()?;
        self.metadata.end_tick = sim.snapshot().tick();
        self.metadata.expected_hash = hash(sim)?;
        self.metadata.events = self.events.len();
        let members = BTreeMap::from([
            (
                "bundle.toml".into(),
                toml::to_string(&self.metadata)
                    .map_err(|e| e.to_string())?
                    .into_bytes(),
            ),
            (
                "commands.log".into(),
                serde_json::to_vec(&self.events).map_err(|e| e.to_string())?,
            ),
            ("commands.txt".into(), human(&self.events)?),
            ("start.ohsave".into(), self.start),
            (
                "final.json".into(),
                serde_json::to_vec(&report(sim)?).map_err(|e| e.to_string())?,
            ),
        ]);
        let b = repro_zip::encode(&members, &Policy::default())?;
        replay(&b, c)?;
        Ok(b)
    }
}
pub fn inspect(bytes: &[u8]) -> Result<Bundle> {
    let m = repro_zip::decode(bytes, &Policy::default())?;
    let b: Bundle = toml::from_str(
        std::str::from_utf8(m.get("bundle.toml").ok_or("Repro: missing bundle.toml")?)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if b.schema != 1
        || b.engine_version != env!("CARGO_PKG_VERSION")
        || b.events > Policy::default().events_max
        || !oh_data::valid_id(&b.scenario)
        || !oh_data::valid_id(&b.pack_id)
    {
        return Err("Repro: unsupported schema/engine/events".into());
    }
    for h in [
        &b.pack_hash,
        &b.effective_defines_hash,
        &b.start_hash,
        &b.expected_hash,
    ] {
        if h.len() != 16
            || !h
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err("Repro: malformed hash".into());
        }
    }
    Ok(b)
}
pub fn replay(bytes: &[u8], c: &SaveContext) -> Result<Simulation> {
    let b = inspect(bytes)?;
    if b.scenario != c.restore_context().scenario
        || b.national != c.restore_context().world.is_some()
        || b.pack_id != c.pack().id
        || b.pack_version != c.pack().version
        || b.pack_hash != format!("{:016x}", c.pack().content_hash)
        || b.effective_defines_hash != format!("{:016x}", c.defines_hash())
    {
        return Err("Repro: pack/context mismatch".into());
    }
    c.verify_unchanged()?;
    let m = repro_zip::decode(bytes, &Policy::default())?;
    for n in ["commands.log", "commands.txt", "final.json"] {
        if !m.contains_key(n) {
            return Err(format!("Repro: missing {n}"));
        }
    }
    if b.start_save != m.contains_key("start.ohsave") {
        return Err("Repro: start_save mismatch".into());
    }
    let mut deserializer = serde_json::Deserializer::from_slice(&m["commands.log"]);
    let events: Vec<Event> = bounded(&mut deserializer, b.events).map_err(|e| e.to_string())?;
    deserializer.end().map_err(|e| e.to_string())?;
    if events.len() != b.events
        || events.iter().enumerate().any(|(i, e)| e.index != i)
        || human(&events)? != m["commands.txt"]
    {
        return Err("Repro: log order/count/human mismatch".into());
    }
    let mut sim = if b.start_save {
        crate::decode(&m["start.ohsave"], c, false)?.simulation
    } else {
        c.simulation(b.seed)?
    };
    if sim.snapshot().seed() != b.seed
        || sim.snapshot().tick() != b.start_tick
        || hash(&sim)? != b.start_hash
    {
        return Err("Repro: start state mismatch".into());
    }
    for expected in events {
        let actual = apply(&mut sim, expected.input.clone(), expected.index)?;
        if actual != expected {
            return Err(format!("Repro: event {} mismatch", expected.index));
        }
    }
    if sim.snapshot().tick() != b.end_tick || hash(&sim)? != b.expected_hash {
        return Err("Repro: expected hash/end tick mismatch".into());
    }
    // Compare canonical host JSON bytes; do not allocate untrusted arbitrary
    // serde_json::Value trees from final.json before validating authority.
    if serde_json::to_vec(&report(&sim)?).map_err(|e| e.to_string())? != m["final.json"] {
        return Err("Repro: full DTO/canonical mismatch".into());
    }
    c.verify_unchanged()?;
    Ok(sim)
}
