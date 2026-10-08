//! Dedicated synchronous simulation thread, bounded host channel, read-only output.
use oh_core::NationId;
use oh_proto::{TimeCommand, TimeState};
use oh_sim::Simulation;
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::{oneshot, watch};
#[derive(Debug)]
pub enum Request {
    Production {
        reply: oneshot::Sender<Option<oh_proto::ProductionView>>,
    },
    ProductionCommand {
        nation: NationId,
        command: oh_proto::ProductionCommand,
        reply: oneshot::Sender<Result<TimeState, &'static str>>,
    },
    Economy {
        reply: oneshot::Sender<Option<oh_proto::EconomyView>>,
    },
    EconomyCommand {
        nation: NationId,
        command: oh_proto::EconomyCommand,
        reply: oneshot::Sender<Result<TimeState, &'static str>>,
    },
    Trigger {
        reply: oneshot::Sender<Option<oh_proto::TriggerView>>,
    },
    World {
        reply: oneshot::Sender<Option<oh_proto::WorldView>>,
    },
    Command {
        command: TimeCommand,
        reply: oneshot::Sender<Result<TimeState, &'static str>>,
    },
}
pub struct Session {
    pub commands: mpsc::SyncSender<Request>,
    pub states: watch::Receiver<TimeState>,
    thread: Option<thread::JoinHandle<()>>,
}
pub(crate) fn validate_resume(sim: &Simulation) -> Result<(), String> {
    if sim
        .pending_commands()
        .keys()
        .any(|(_, _, sequence)| *sequence == u64::MAX)
    {
        return Err("ArrivalOverflow: pending sequence".into());
    }
    let now = Instant::now();
    if sim
        .config()
        .speed_ms_per_tick()
        .iter()
        .any(|ms| now.checked_add(Duration::from_millis(*ms)).is_none())
    {
        return Err("PacingOverflow: host Instant".into());
    }
    Ok(())
}
impl Session {
    pub fn start(mut sim: Simulation, capacity: usize, delta_ms: u64) -> Self {
        let (commands, incoming) = mpsc::sync_channel(capacity);
        let (outgoing, states) = watch::channel(TimeState::from(&sim.snapshot()));
        let thread = thread::Builder::new()
            .name("openhoi-simulation".into())
            .spawn(move || {
                let mut arrival = sim
                    .pending_commands()
                    .keys()
                    .map(|(_, _, sequence)| *sequence)
                    .max()
                    .unwrap_or(0);
                let mut deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                let mut published = Instant::now();
                loop {
                    let wait = if sim.snapshot().paused() || sim.is_ended() {
                        Duration::from_millis(delta_ms)
                    } else {
                        deadline.saturating_duration_since(Instant::now())
                    };
                    match incoming.recv_timeout(wait) {
                        Ok(Request::Production { reply }) => {
                            let _ = reply.send(oh_proto::ProductionView::from_sim(&sim));
                            continue;
                        }
                        Ok(Request::Economy { reply }) => {
                            let _ = reply.send(oh_proto::EconomyView::from_sim(&sim));
                            continue;
                        }
                        Ok(Request::ProductionCommand {
                            nation,
                            command,
                            reply,
                        }) => {
                            if sim.is_ended() {
                                let _ = reply.send(Err("scenario-ended"));
                                continue;
                            }
                            let action = match command.into_action() {
                                Ok(action) => action,
                                Err(_) => {
                                    let _ = reply.send(Err("invalid-message"));
                                    continue;
                                }
                            };
                            let Some(next_arrival) = arrival.checked_add(1) else {
                                let _ = reply.send(Err("simulation-error"));
                                break;
                            };
                            if sim
                                .enqueue(
                                    sim.snapshot().tick(),
                                    nation,
                                    next_arrival,
                                    oh_sim::Command::Production(action),
                                )
                                .is_err()
                            {
                                let _ = reply.send(Err("invalid-message"));
                                continue;
                            }
                            arrival = next_arrival;
                            match sim.step() {
                                Ok(step) => {
                                    let accepted = step
                                        .commands
                                        .iter()
                                        .find(|c| c.sequence == arrival && c.nation == nation)
                                        .is_some_and(|c| c.result.is_ok());
                                    let _ = reply.send(if accepted {
                                        Ok(TimeState::from(&step.snapshot))
                                    } else {
                                        Err("invalid-message")
                                    });
                                }
                                Err(_) => {
                                    let _ = reply.send(Err("simulation-error"));
                                    continue;
                                }
                            }
                            deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                        }
                        Ok(Request::EconomyCommand {
                            nation,
                            command,
                            reply,
                        }) => {
                            if sim.is_ended() {
                                let _ = reply.send(Err("scenario-ended"));
                                continue;
                            }
                            let action = match command.into_action() {
                                Ok(action) => action,
                                Err(_) => {
                                    let _ = reply.send(Err("invalid-message"));
                                    continue;
                                }
                            };
                            let Some(next_arrival) = arrival.checked_add(1) else {
                                let _ = reply.send(Err("simulation-error"));
                                break;
                            };
                            if sim
                                .enqueue(
                                    sim.snapshot().tick(),
                                    nation,
                                    next_arrival,
                                    oh_sim::Command::Economy(action),
                                )
                                .is_err()
                            {
                                let _ = reply.send(Err("invalid-message"));
                                continue;
                            }
                            arrival = next_arrival;
                            match sim.step() {
                                Ok(step) => {
                                    let accepted = step
                                        .commands
                                        .iter()
                                        .find(|c| c.sequence == arrival && c.nation == nation)
                                        .is_some_and(|c| c.result.is_ok());
                                    let _ = reply.send(if accepted {
                                        Ok(TimeState::from(&step.snapshot))
                                    } else {
                                        Err("invalid-message")
                                    });
                                }
                                Err(_) => {
                                    let _ = reply.send(Err("simulation-error"));
                                    continue;
                                }
                            }
                            deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                        }
                        Ok(Request::Trigger { reply }) => {
                            let _ = reply.send(oh_proto::TriggerView::from_sim(&sim));
                            continue;
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Ok(Request::World { reply }) => {
                            let _ = reply.send(oh_proto::WorldView::from_sim(&sim));
                            continue;
                        }
                        Ok(Request::Command { command, reply }) => {
                            if sim.is_ended() {
                                let _ = reply.send(Err("scenario-ended"));
                                continue;
                            }
                            // Each input is applied at the next simulation step's command phase.
                            let command = match command {
                                TimeCommand::Pause { paused } => oh_sim::Command::Pause(paused),
                                TimeCommand::SetSpeed { speed } => oh_sim::Command::SetSpeed(speed),
                            };
                            let Some(next_arrival) = arrival.checked_add(1) else {
                                let _ = reply.send(Err("simulation-error"));
                                break;
                            };
                            arrival = next_arrival;
                            let result = sim
                                .enqueue(sim.snapshot().tick(), NationId(0), arrival, command)
                                .and_then(|()| sim.step());
                            match result {
                                Ok(step) => {
                                    let accepted =
                                        step.commands.first().is_some_and(|c| c.result.is_ok());
                                    let _ = reply.send(if accepted {
                                        Ok(TimeState::from(&step.snapshot))
                                    } else {
                                        Err("invalid-speed")
                                    });
                                }
                                Err(_) => {
                                    let _ = reply.send(Err("simulation-error"));
                                    break;
                                }
                            }
                            deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            if sim.is_ended() {
                                continue;
                            }
                            if sim.step().is_err() {
                                break;
                            }
                            deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                        }
                    }
                    if sim.is_ended() || published.elapsed() >= Duration::from_millis(delta_ms) {
                        outgoing.send_replace(TimeState::from(&sim.snapshot()));
                        published = Instant::now();
                    }
                }
            })
            .expect("simulation thread creation");
        Self {
            commands,
            states,
            thread: Some(thread),
        }
    }
    pub async fn stop(mut self) {
        // Dropping the sole sender cannot fail even if the bounded queue is full.
        drop(self.commands);
        if let Some(thread) = self.thread.take() {
            let _ = tokio::task::spawn_blocking(move || thread.join()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn req_gen_05_thread_uses_simulation_and_stops_with_queued_input() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0");
        let loaded = oh_data::m0::load_m0_scenario(&root, "testland").unwrap();
        let config = oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap();
        let sim = Simulation::new(
            "testland".into(),
            oh_sim::Date::new(2000, 1, 1).unwrap(),
            1,
            config,
        )
        .unwrap();
        let session = Session::start(sim, 1, 100);
        async fn command(session: &Session, command: TimeCommand) -> TimeState {
            let (reply, result) = oneshot::channel();
            session
                .commands
                .try_send(Request::Command { command, reply })
                .unwrap();
            result.await.unwrap().unwrap()
        }
        let paused = command(&session, TimeCommand::Pause { paused: true }).await;
        assert!(paused.paused);
        let fast = command(&session, TimeCommand::SetSpeed { speed: 5 }).await;
        assert_eq!(fast.tick, paused.tick);
        assert_eq!(fast.speed, 5);
        let (reply, _) = oneshot::channel();
        session
            .commands
            .try_send(Request::Command {
                command: TimeCommand::Pause { paused: false },
                reply,
            })
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), session.stop())
            .await
            .unwrap();
    }
    #[tokio::test]
    async fn req_sav_02_restored_current_tick_queue_does_not_collide_with_new_arrival() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/examples/m0");
        let context = oh_save::SaveContext::m0(&root, "testland").unwrap();
        let mut dto = context.simulation(7).unwrap().export_save().unwrap();
        dto.state.paused = true;
        dto.queue = vec![
            oh_sim::save_state::PendingV1 {
                tick: 0,
                nation: 0,
                sequence: 1,
                command: oh_sim::save_state::CommandV1::Pause(false),
            },
            oh_sim::save_state::PendingV1 {
                tick: 12,
                nation: 0,
                sequence: 100,
                command: oh_sim::save_state::CommandV1::SetSpeed(5),
            },
        ];
        let sim = Simulation::from_save(dto, context.restore_context()).unwrap();
        let session = Session::start(sim, 1, 100);
        let (reply, result) = oneshot::channel();
        session
            .commands
            .try_send(Request::Command {
                command: TimeCommand::Pause { paused: true },
                reply,
            })
            .unwrap();
        let state = tokio::time::timeout(Duration::from_secs(2), result)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(state.tick, "0");
        assert!(state.paused);
        session.stop().await;
    }
}
