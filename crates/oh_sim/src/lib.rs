//! Synchronous headless scheduler. No I/O, clocks, networking or rendering.
//! M0 gameplay slots stay empty; M1 adds private world state and ledger publication.
//! The host validates authority and supplies a server arrival sequence. Both
//! player and AI inputs use the same queue. Time commands assume single player;
//! multiplayer authorization belongs to oh_server (02 §10).
use oh_core::{NationId, SerializationError};
use serde::Serialize;
use std::collections::BTreeMap;
pub mod formula;
pub mod ledger;
mod time;
pub mod world;
pub use time::{Date, TimeConfig};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidWorld,
    InvalidDate,
    InvalidSpeed,
    InvalidTimeDefines,
    EmptyScenario,
    PastCommand,
    DuplicateCommand,
    ClockOverflow,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidWorld => "invalid world or ledger calculation",
            Self::InvalidDate => "invalid Gregorian date",
            Self::InvalidSpeed => "speed must be in 1..=5",
            Self::InvalidTimeDefines => "time defines require five nonnegative integer speed_ms_per_tick values and initial_speed in 1..=5",
            Self::EmptyScenario => "scenario ID must not be empty",
            Self::PastCommand => "command is scheduled before the current tick",
            Self::DuplicateCommand => "duplicate (tick, nation, arrival sequence)",
            Self::ClockOverflow => "simulation clock overflow",
        })
    }
}
impl std::error::Error for Error {}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum Command {
    Pause(bool),
    SetSpeed(u8),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Commands,
    Movement,
    Combat,
    RetreatAndControl,
    Supply,
    Economy,
    Production,
    Construction,
    ManpowerTrainingReinforcement,
    Research,
    Politics,
    AgendaAndDecisions,
    Events,
    Diplomacy,
    Ai,
    Ledger,
    MonthlyStatistics,
    Snapshot,
}
/// Owned, read-only snapshot. It exposes no setters or mutable references.
/// ```compile_fail
/// fn overwrite_time(state: &mut oh_sim::State) { state.speed = 5; }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct State {
    scenario: String,
    seed: u64,
    tick: u64,
    date: Date,
    hour: u8,
    paused: bool,
    speed: u8,
}
impl State {
    pub fn scenario(&self) -> &str {
        &self.scenario
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn date(&self) -> Date {
        self.date
    }
    pub fn hour(&self) -> u8 {
        self.hour
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn speed(&self) -> u8 {
        self.speed
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub nation: NationId,
    pub sequence: u64,
    pub result: Result<(), Error>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Step {
    pub advanced: bool,
    pub commands: Vec<CommandResult>,
    /// Ordered execution trace of actual scheduler slots, not gameplay results.
    pub phases: Vec<Phase>,
    pub snapshot: State,
}
/// The canonical hash covers state, configuration and ordered pending queue.
/// Insertion order and host wall-clock timing are absent from serialized state.
#[derive(Clone, Debug, Serialize)]
pub struct Simulation {
    state: State,
    config: TimeConfig,
    queue: BTreeMap<(u64, NationId, u64), Command>,
    #[serde(skip_serializing_if = "Option::is_none")]
    world: Option<world::World>,
}
impl Simulation {
    pub fn new(scenario: String, date: Date, seed: u64, config: TimeConfig) -> Result<Self, Error> {
        if scenario.is_empty() {
            return Err(Error::EmptyScenario);
        }
        Ok(Self {
            state: State {
                scenario,
                seed,
                tick: 0,
                date,
                hour: 0,
                paused: false,
                speed: config.initial_speed(),
            },
            config,
            queue: BTreeMap::new(),
            world: None,
        })
    }
    pub fn with_world(
        scenario: String,
        date: Date,
        seed: u64,
        config: TimeConfig,
        world: world::World,
    ) -> Result<Self, Error> {
        let mut sim = Self::new(scenario, date, seed, config)?;
        sim.world = Some(world);
        Ok(sim)
    }
    /// Read-only resume boundary: WP-11 must preserve config and the complete queue.
    pub fn config(&self) -> &TimeConfig {
        &self.config
    }
    pub fn pending_commands(&self) -> &BTreeMap<(u64, NationId, u64), Command> {
        &self.queue
    }
    pub fn world(&self) -> Option<&world::World> {
        self.world.as_ref()
    }
    pub fn snapshot(&self) -> State {
        self.state.clone()
    }
    pub fn ms_per_tick(&self) -> u64 {
        self.config.ms_per_tick(self.state.speed)
    }
    pub fn state_hash(&self) -> Result<u64, SerializationError> {
        oh_core::state_hash(self)
    }
    /// tick counts completed hours, i.e. the next step's start.
    /// While paused, enqueue resume at snapshot().tick() and keep calling step.
    pub fn enqueue(
        &mut self,
        tick: u64,
        nation: NationId,
        sequence: u64,
        command: Command,
    ) -> Result<(), Error> {
        if tick < self.state.tick {
            return Err(Error::PastCommand);
        }
        let key = (tick, nation, sequence);
        if self.queue.contains_key(&key) {
            return Err(Error::DuplicateCommand);
        }
        self.queue.insert(key, command);
        Ok(())
    }
    /// Pump commands even when paused. A running step advances exactly one hour.
    /// Clock failure is atomic: neither state nor pending commands are lost.
    pub fn step(&mut self) -> Result<Step, Error> {
        let mut next = self.state.clone();
        let mut next_world = self.world.clone();
        let keys: Vec<_> = self
            .queue
            .keys()
            .take_while(|(tick, _, _)| *tick == next.tick)
            .copied()
            .collect();
        let mut commands = Vec::new();
        for key @ (_, nation, sequence) in &keys {
            let result = match &self.queue[key] {
                Command::Pause(paused) => {
                    next.paused = *paused;
                    Ok(())
                }
                Command::SetSpeed(speed) => {
                    if self.config.valid_speed(*speed) {
                        next.speed = *speed;
                        Ok(())
                    } else {
                        Err(Error::InvalidSpeed)
                    }
                }
            };
            commands.push(CommandResult {
                nation: *nation,
                sequence: *sequence,
                result,
            });
        }
        let mut phases = vec![Phase::Commands];
        let advanced = !next.paused;
        if advanced {
            next.tick = next.tick.checked_add(1).ok_or(Error::ClockOverflow)?;
            let (date, hour) = formula::next_hour(next.date, next.hour)?;
            next.date = date;
            next.hour = hour;
            // Empty M0 slots in the prescribed order. Later WPs own their rules.
            // Calendar is advanced before midnight systems run.
            phases.extend([Phase::Movement, Phase::Combat, Phase::RetreatAndControl]);
            if hour == 0 {
                phases.extend([
                    Phase::Supply,
                    Phase::Economy,
                    Phase::Production,
                    Phase::Construction,
                    Phase::ManpowerTrainingReinforcement,
                    Phase::Research,
                    Phase::Politics,
                    Phase::AgendaAndDecisions,
                    Phase::Events,
                    Phase::Diplomacy,
                    Phase::Ai,
                    Phase::Ledger,
                ]);
                if date.day() == 1 {
                    phases.push(Phase::MonthlyStatistics);
                }
            }
        }
        if let Some(world) = next_world.as_mut() {
            world.evaluate(next.tick).map_err(|_| Error::InvalidWorld)?;
        }
        phases.push(Phase::Snapshot);
        for key in keys {
            self.queue.remove(&key);
        }
        self.state = next;
        self.world = next_world;
        Ok(Step {
            advanced,
            commands,
            phases,
            snapshot: self.snapshot(),
        })
    }
}
