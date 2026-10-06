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
impl Session {
    pub fn start(mut sim: Simulation, capacity: usize, delta_ms: u64) -> Self {
        let (commands, incoming) = mpsc::sync_channel(capacity);
        let (outgoing, states) = watch::channel(TimeState::from(&sim.snapshot()));
        let thread = thread::Builder::new()
            .name("openhoi-simulation".into())
            .spawn(move || {
                let mut arrival = 0u64;
                let mut deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                let mut published = Instant::now();
                loop {
                    let wait = if sim.snapshot().paused() {
                        Duration::from_millis(delta_ms)
                    } else {
                        deadline.saturating_duration_since(Instant::now())
                    };
                    match incoming.recv_timeout(wait) {
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Ok(Request::Command { command, reply }) => {
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
                            if sim.step().is_err() {
                                break;
                            }
                            deadline = Instant::now() + Duration::from_millis(sim.ms_per_tick());
                        }
                    }
                    if published.elapsed() >= Duration::from_millis(delta_ms) {
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
}
