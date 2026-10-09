//! Synchronous headless scheduler. No I/O, clocks, networking or rendering.
//! M0 gameplay slots stay empty; M1 adds private world state and ledger publication.
//! The host validates authority and supplies a server arrival sequence. Both
//! player and AI inputs use the same queue. Time commands assume single player;
//! multiplayer authorization belongs to oh_server (02 §10).
use oh_core::{DivisionId, NationId, ProvinceId, SerializationError};
use serde::Serialize;
use std::collections::BTreeMap;
pub mod economy;
pub mod economy_save;
pub mod formula;
pub mod ledger;
pub mod military;
pub mod military_save;
pub mod military_templates;
pub mod movement;
pub mod production;
pub mod production_save;
pub mod save_state;
pub mod supply;
mod time;
pub mod trigger;
pub mod trigger_save;
pub mod world;
pub use time::{Date, TimeConfig};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Production(production::ProductionError),
    Military(military::MilitaryError),
    Economy(economy::EconomyError),
    ScenarioEnded,
    Trigger(trigger::TriggerError),
    InvalidWorld,
    Movement(movement::MovementError),
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
        if let Self::Movement(error) = self {
            return error.fmt(f);
        }
        f.write_str(match self {
            Self::Military(_) => "military command or phase failed",
            Self::Production(_) => "production command or phase failed",
            Self::Economy(_) => "economy command or phase failed",
            Self::ScenarioEnded => "scenario-ended",
            Self::Trigger(_) => "trigger execution failed",
            Self::Movement(_) => "movement command or phase failed",
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
    Move {
        unit: DivisionId,
        destination: ProvinceId,
    },
    Stop {
        unit: DivisionId,
    },
    Effects {
        program: String,
    },
    Economy(economy::Action),
    Production(production::Action),
    Military(military::Action),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    movement: Option<movement::Movement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trigger: Option<trigger::TriggerState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    economy: Option<economy::Economy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    production: Option<production::Production>,
    #[serde(skip_serializing_if = "Option::is_none")]
    military: Option<military::Military>,
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
            movement: None,
            trigger: None,
            economy: None,
            production: None,
            military: None,
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
        if world.defs().economy().is_some() {
            sim.economy = Some(economy::Economy::initial(&world, 0).map_err(Error::Economy)?);
        }
        if world.defs().production().is_some() {
            sim.production =
                Some(production::Production::initial(&world).map_err(Error::Production)?);
        }
        if world.defs().military().is_some() {
            sim.military = Some(
                military::Military::initial(
                    &world,
                    sim.economy.as_ref().ok_or(Error::InvalidWorld)?,
                    sim.production.as_ref().ok_or(Error::InvalidWorld)?,
                )
                .map_err(Error::Military)?,
            );
        }
        sim.world = Some(world);
        if let Some(world) = sim.world.as_ref()
            && let Some(defs) = world.defs().trigger()
        {
            let mut trigger =
                trigger::TriggerState::initial(defs, world).map_err(Error::Trigger)?;
            trigger::checkpoint_economy(
                world,
                &sim.state,
                &mut trigger,
                Default::default(),
                true,
                sim.economy.as_ref(),
            )
            .map_err(Error::Trigger)?;
            sim.trigger = Some(trigger);
            if sim.is_ended()
                && let Some(economy) = sim.economy.as_mut()
            {
                economy
                    .checkpoint_scores(sim.world.as_ref().ok_or(Error::InvalidWorld)?, 0)
                    .map_err(Error::Economy)?;
            }
        }
        Ok(sim)
    }
    /// Trusted host initialization only. No client supplies speed or context.
    pub fn with_movement(
        scenario: String,
        date: Date,
        seed: u64,
        config: TimeConfig,
        world: world::World,
        movement: movement::Movement,
    ) -> Result<Self, Error> {
        movement.validate(&world).map_err(Error::Movement)?;
        let mut sim = Self::with_world(scenario, date, seed, config, world)?;
        sim.movement = Some(movement);
        Ok(sim)
    }
    pub fn military(&self) -> Option<&military::Military> {
        self.military.as_ref()
    }
    pub fn validate_military_pending(
        &self,
        n: NationId,
        a: &military::Action,
    ) -> Result<(), String> {
        self.military
            .as_ref()
            .ok_or("MilitaryModeMismatch")?
            .validate_pending(self.world.as_ref().ok_or("MilitaryRequiresWorld")?, n, a)
            .map_err(|e| e.to_string())
    }
    fn apply_military(
        c: &Command,
        n: NationId,
        w: Option<&world::World>,
        e: Option<&mut economy::Economy>,
        p: Option<&mut production::Production>,
        m: Option<&mut military::Military>,
        tick: u64,
    ) -> Result<(), Error> {
        let Command::Military(a) = c else {
            return Ok(());
        };
        m.ok_or(Error::Military(military::MilitaryError::MissingContext))?
            .command(
                w.ok_or(Error::InvalidWorld)?,
                e.ok_or(Error::InvalidWorld)?,
                p.ok_or(Error::InvalidWorld)?,
                n,
                a,
                tick,
            )
            .map_err(Error::Military)
    }
    pub fn production(&self) -> Option<&production::Production> {
        self.production.as_ref()
    }
    fn apply_production(
        command: &Command,
        nation: NationId,
        world: Option<&world::World>,
        economy: Option<&economy::Economy>,
        production: Option<&mut production::Production>,
        tick: u64,
    ) -> Result<(), Error> {
        let Command::Production(a) = command else {
            return Ok(());
        };
        production
            .ok_or(Error::Production(
                production::ProductionError::MissingContext,
            ))?
            .command(
                world.ok_or(Error::InvalidWorld)?,
                economy.ok_or(Error::Production(
                    production::ProductionError::MissingContext,
                ))?,
                nation,
                a,
                tick,
            )
            .map_err(Error::Production)
    }
    pub fn economy(&self) -> Option<&economy::Economy> {
        self.economy.as_ref()
    }
    fn apply_economy(
        command: &Command,
        nation: NationId,
        world: Option<&world::World>,
        state: &State,
        trigger: Option<&trigger::TriggerState>,
        economy: Option<&mut economy::Economy>,
    ) -> Result<(), Error> {
        let Command::Economy(action) = command else {
            return Ok(());
        };
        let world = world.ok_or(Error::InvalidWorld)?;
        let mut condition_met = true;
        if let economy::Action::ChangeLaw { law } = action {
            let definition = world
                .defs()
                .economy()
                .ok_or(Error::Economy(economy::EconomyError::MissingContext))?;
            let law = definition
                .laws
                .get(law)
                .ok_or(Error::Economy(economy::EconomyError::InvalidReference))?;
            let empty = trigger::TriggerState::empty(world);
            let host = trigger::SimHost::new(world, state, trigger.unwrap_or(&empty))
                .with_economy(economy.as_deref());
            condition_met =
                trigger::evaluate(&host, &law.condition, trigger::Scope::Nation(nation))
                    .map_err(Error::Trigger)?;
        }
        economy
            .ok_or(Error::Economy(economy::EconomyError::MissingContext))?
            .command(world, nation, action, state.tick(), condition_met)
            .map_err(Error::Economy)
    }
    pub fn movement(&self) -> Option<&movement::Movement> {
        self.movement.as_ref()
    }
    pub fn trigger_state(&self) -> Option<&trigger::TriggerState> {
        self.trigger.as_ref()
    }
    pub fn is_ended(&self) -> bool {
        self.trigger.as_ref().is_some_and(|t| t.ended.is_some())
    }
    fn apply_effects(
        command: &Command,
        nation: NationId,
        world: Option<&world::World>,
        state: &State,
        trigger: Option<&mut trigger::TriggerState>,
        economy: Option<&mut economy::Economy>,
    ) -> Result<std::collections::BTreeSet<String>, Error> {
        let Command::Effects { program } = command else {
            return Ok(Default::default());
        };
        let world = world.ok_or(Error::InvalidWorld)?;
        let defs = world
            .defs()
            .trigger()
            .ok_or(Error::Trigger(trigger::TriggerError::MissingContext))?;
        let p = defs
            .effect_programs
            .as_ref()
            .and_then(|ps| ps.get(program))
            .ok_or(Error::Trigger(trigger::TriggerError::InvalidReference))?;
        if world.nation(nation).is_none() {
            return Err(Error::Trigger(trigger::TriggerError::InvalidReference));
        }
        let scope = trigger::root(world, p.root.as_deref()).map_err(Error::Trigger)?;
        if let trigger::Scope::Nation(id) = scope
            && id != nation
        {
            return Err(Error::Trigger(trigger::TriggerError::InvalidReference));
        }
        let trigger = trigger.ok_or(Error::Trigger(trigger::TriggerError::MissingContext))?;
        let mut host =
            trigger::SimHost::new(world, state, trigger).with_economy(economy.as_deref());
        trigger::execute(&mut host, &p.effects, scope).map_err(Error::Trigger)?;
        let sources = host.sources.clone();
        if let (Some(target), Some(value)) = (economy, host.economy()) {
            *target = value.clone();
        }
        host.commit(trigger);
        Ok(sources)
    }
    fn apply_movement(
        command: &Command,
        nation: NationId,
        world: Option<&world::World>,
        movement: Option<&mut movement::Movement>,
    ) -> Result<(), Error> {
        let (unit, destination) = match command {
            Command::Move { unit, destination } => (*unit, Some(*destination)),
            Command::Stop { unit } => (*unit, None),
            _ => return Ok(()),
        };
        movement
            .ok_or(Error::Movement(movement::MovementError::MissingContext))?
            .command(world.ok_or(Error::InvalidWorld)?, nation, unit, destination)
            .map_err(Error::Movement)
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
        if self.is_ended() {
            return Err(Error::ScenarioEnded);
        }
        if tick < self.state.tick {
            return Err(Error::PastCommand);
        }
        let key = (tick, nation, sequence);
        if self.queue.contains_key(&key) {
            return Err(Error::DuplicateCommand);
        }
        if matches!(command, Command::Move { .. } | Command::Stop { .. }) {
            let mut preview = self.movement.clone();
            Self::apply_movement(&command, nation, self.world.as_ref(), preview.as_mut())?;
        }
        if matches!(command, Command::Effects { .. }) {
            let mut preview = self.trigger.clone();
            let mut preview_economy = self.economy.clone();
            Self::apply_effects(
                &command,
                nation,
                self.world.as_ref(),
                &self.state,
                preview.as_mut(),
                preview_economy.as_mut(),
            )?;
        }
        if matches!(command, Command::Economy(_)) {
            let mut preview = self.economy.clone();
            Self::apply_economy(
                &command,
                nation,
                self.world.as_ref(),
                &self.state,
                self.trigger.as_ref(),
                preview.as_mut(),
            )?;
        }
        if matches!(command, Command::Production(_)) {
            let mut preview = self.production.clone();
            Self::apply_production(
                &command,
                nation,
                self.world.as_ref(),
                self.economy.as_ref(),
                preview.as_mut(),
                self.state.tick(),
            )?;
        }
        if let Command::Military(a) = &command {
            self.military
                .as_ref()
                .ok_or(Error::Military(military::MilitaryError::MissingContext))?
                .validate_pending(self.world.as_ref().ok_or(Error::InvalidWorld)?, nation, a)
                .map_err(Error::Military)?;
        }
        self.queue.insert(key, command);
        Ok(())
    }
    /// Pump commands even when paused. A running step advances exactly one hour.
    /// Clock failure is atomic: neither state nor pending commands are lost.
    pub fn step(&mut self) -> Result<Step, Error> {
        if self.is_ended() {
            return Err(Error::ScenarioEnded);
        }
        let mut next = self.state.clone();
        let mut next_world = self.world.clone();
        let mut next_movement = self.movement.clone();
        let mut next_trigger = self.trigger.clone();
        let mut next_economy = self.economy.clone();
        let mut next_production = self.production.clone();
        let mut next_military = self.military.clone();
        let mut end_sources = std::collections::BTreeSet::new();
        let keys: Vec<_> = self
            .queue
            .keys()
            .take_while(|(tick, _, _)| *tick == next.tick)
            .copied()
            .collect();
        let mut commands = Vec::new();
        for key @ (_, nation, sequence) in &keys {
            let result = match &self.queue[key] {
                c @ Command::Military(_) => Self::apply_military(
                    c,
                    *nation,
                    next_world.as_ref(),
                    next_economy.as_mut(),
                    next_production.as_mut(),
                    next_military.as_mut(),
                    next.tick(),
                ),
                c @ Command::Production(_) => Self::apply_production(
                    c,
                    *nation,
                    next_world.as_ref(),
                    next_economy.as_ref(),
                    next_production.as_mut(),
                    next.tick(),
                ),
                c @ Command::Economy(_) => Self::apply_economy(
                    c,
                    *nation,
                    next_world.as_ref(),
                    &next,
                    next_trigger.as_ref(),
                    next_economy.as_mut(),
                ),
                c @ Command::Effects { .. } => Self::apply_effects(
                    c,
                    *nation,
                    next_world.as_ref(),
                    &next,
                    next_trigger.as_mut(),
                    next_economy.as_mut(),
                )
                .map(|sources| end_sources.extend(sources)),
                Command::Pause(paused) => {
                    next.paused = *paused;
                    Ok(())
                }
                c @ (Command::Move { .. } | Command::Stop { .. }) => {
                    Self::apply_movement(c, *nation, next_world.as_ref(), next_movement.as_mut())
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
            if let Some(movement) = next_movement.as_mut() {
                movement.advance().map_err(Error::Movement)?;
            }
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
                if let Some(economy) = next_economy.as_mut() {
                    let world = next_world.as_mut().ok_or(Error::InvalidWorld)?;
                    // Each producer executes at its prescribed daily slot.
                    economy
                        .daily_economy(world, next.tick)
                        .map_err(Error::Economy)?;
                    if let Some(production) = next_production.as_mut() {
                        if let Some(m) = next_military.as_ref() {
                            production
                                .daily_with_holdings(
                                    world,
                                    economy,
                                    next.tick,
                                    &m.returnable_holdings().map_err(Error::Military)?,
                                )
                                .map_err(Error::Production)?;
                        } else {
                            production
                                .daily(world, economy, next.tick)
                                .map_err(Error::Production)?;
                        }
                    }
                    economy
                        .daily_construction(world, next.tick)
                        .map_err(Error::Economy)?;
                    if let Some(military) = next_military.as_mut() {
                        military
                            .daily(
                                world,
                                economy,
                                next_production.as_mut().ok_or(Error::InvalidWorld)?,
                                next.tick,
                            )
                            .map_err(Error::Military)?;
                    }
                    economy.daily_politics(world).map_err(Error::Economy)?;
                }
                if date.day() == 1 {
                    phases.push(Phase::MonthlyStatistics);
                }
            }
        }
        if let Some(world) = next_world.as_mut() {
            world.evaluate(next.tick).map_err(|_| Error::InvalidWorld)?;
        }
        if let Some(trigger) = next_trigger.as_mut() {
            trigger::checkpoint_economy(
                next_world.as_ref().ok_or(Error::InvalidWorld)?,
                &next,
                trigger,
                end_sources,
                advanced,
                next_economy.as_ref(),
            )
            .map_err(Error::Trigger)?;
        }
        if next_trigger.as_ref().is_some_and(|t| t.ended.is_some())
            && let Some(economy) = next_economy.as_mut()
        {
            economy
                .checkpoint_scores(next_world.as_ref().ok_or(Error::InvalidWorld)?, next.tick())
                .map_err(Error::Economy)?;
        }
        phases.push(Phase::Snapshot);
        for key in keys {
            self.queue.remove(&key);
        }
        self.state = next;
        self.world = next_world;
        self.movement = next_movement;
        self.trigger = next_trigger;
        self.economy = next_economy;
        self.production = next_production;
        self.military = next_military;
        Ok(Step {
            advanced,
            commands,
            phases,
            snapshot: self.snapshot(),
        })
    }
}
