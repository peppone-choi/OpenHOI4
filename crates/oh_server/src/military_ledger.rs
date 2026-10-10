//! Only this opt-in response is bounded; legacy responses and inbound limits stay intact.
use axum::extract::ws::{Message, WebSocket};
use oh_proto::ServerMessage;
#[cfg(test)]
#[path = "../../oh_save/tests/support/military.rs"]
mod fixture;
#[cfg(test)]
#[path = "../../oh_proto/tests/support/military_ledger.rs"]
mod ledger_fixture;
pub const KIND: &str = "military-normal-ledger.v1";
pub const PREFIX: &str = "military-normal-ledger.v1:";
pub fn valid_request(request: &str) -> bool {
    request
        .strip_prefix("military-ledger:")
        .is_some_and(|serial| serial.parse::<u64>().is_ok_and(|v| v.to_string() == serial))
}
fn failure(request: &str) -> ServerMessage {
    ServerMessage::MilitaryNormalLedgerResult {
        request: request.into(),
        supported: false,
        reason_key: Some("ledger-too-large".into()),
        ledger: None,
    }
}
pub fn minimum_budget() -> usize {
    let request = format!("military-ledger:{}", u64::MAX);
    let capability = ServerMessage::MilitaryNormalLedgerCapabilityResult {
        request: request.clone(),
        supported: false,
        reason_key: Some("unsupported-query".into()),
    };
    oh_proto::encode(&failure(&request))
        .expect("fixed failure encodes")
        .len()
        .max(
            oh_proto::encode(&capability)
                .expect("fixed capability encodes")
                .len(),
        )
}
pub fn validate_budget(budget: usize) -> Result<(), String> {
    if budget < minimum_budget() {
        Err(format!(
            "military ledger budget must be at least {} bytes",
            minimum_budget()
        ))
    } else {
        Ok(())
    }
}
fn bounded(message: &ServerMessage, request: &str, budget: usize) -> Result<Vec<u8>, String> {
    let bytes = oh_proto::encode(message);
    if let Ok(bytes) = bytes
        && bytes.len() <= budget
    {
        return Ok(bytes);
    }
    let fallback = oh_proto::encode(&failure(request)).map_err(|e| e.to_string())?;
    if fallback.len() > budget {
        return Err("military ledger failure exceeds budget".into());
    }
    Ok(fallback)
}
pub async fn send(
    socket: &mut WebSocket,
    message: ServerMessage,
    request: &str,
    budget: usize,
) -> bool {
    match bounded(&message, request, budget) {
        Ok(bytes) => socket.send(Message::Binary(bytes.into())).await.is_ok(),
        Err(_) => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_envelope_not_just_ledger_is_bounded_and_failure_fits() {
        let request = format!("military-ledger:{}", u64::MAX);
        let (root, template) = ledger_fixture::maximum_pack(fixture::pack());
        let loaded = oh_data::national::load_scenario(&root, "m1").unwrap();
        let sim = oh_sim::Simulation::with_world(
            "m1".into(),
            oh_sim::Date::new(2000, 1, 1).unwrap(),
            1,
            oh_sim::TimeConfig::from_defines(&loaded.pack.defines).unwrap(),
            oh_sim::world::World::from_loaded(&loaded).unwrap(),
        )
        .unwrap();
        let message = ServerMessage::MilitaryNormalLedgerResult {
            request: request.clone(),
            supported: true,
            reason_key: None,
            ledger: Some(oh_proto::MilitaryNormalLedgerView::from_sim(&sim, &template).unwrap()),
        };
        let budget = minimum_budget();
        let bytes = bounded(&message, &request, budget).unwrap();
        assert!(bytes.len() <= budget);
        assert_eq!(oh_proto::decode_server(&bytes).unwrap(), failure(&request));
        assert!(validate_budget(budget - 1).is_err());
        assert!(validate_budget(budget).is_ok());
        let exact = oh_proto::encode(&message).unwrap().len();
        assert_eq!(
            oh_proto::decode_server(&bounded(&message, &request, exact).unwrap()).unwrap(),
            message
        );
        assert_eq!(
            oh_proto::decode_server(&bounded(&message, &request, exact - 1).unwrap()).unwrap(),
            failure(&request)
        );
    }
    #[test]
    fn request_grammar_is_bounded_and_canonical() {
        for valid in ["military-ledger:0", "military-ledger:18446744073709551615"] {
            assert!(valid_request(valid));
        }
        for bad in [
            "military-ledger:01",
            "military-ledger:-1",
            "military-ledger:18446744073709551616",
            "military:1",
        ] {
            assert!(!valid_request(bad));
        }
    }
    #[test]
    fn cli_budget_is_opt_in_canonical_and_cannot_disable_failure_packets() {
        let parse = |v: &[&str]| {
            crate::Options::parse(&v.iter().map(|v| v.to_string()).collect::<Vec<_>>())
        };
        assert_eq!(parse(&[]).unwrap().military_ledger_budget_bytes, None);
        assert_eq!(
            parse(&["--military-ledger-budget-bytes", "256"])
                .unwrap()
                .military_ledger_budget_bytes,
            Some(256)
        );
        for invalid in ["0", "1", "0256", "-256", "256.0"] {
            assert!(parse(&["--military-ledger-budget-bytes", invalid]).is_err());
        }
        assert!(parse(&["--military-ledger-budget-bytes"]).is_err());
        assert!(
            parse(&[
                "--military-ledger-budget-bytes",
                "256",
                "--military-ledger-budget-bytes",
                "256"
            ])
            .is_err()
        );
    }
}
