//! Initial-World diagnostic command. External config never creates daily authority.
use oh_core::NationId;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
pub const USAGE: &str = "supply-network inspect --pack <root> --scenario <id> --nation <canonical-u16> --config <external.toml> (initial World only; no --load)";
#[derive(Debug, Eq, PartialEq)]
pub struct Options {
    pub pack: PathBuf,
    pub scenario: String,
    pub nation: NationId,
    pub config: PathBuf,
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.first().map(String::as_str) != Some("supply-network")
            || args.get(1).map(String::as_str) != Some("inspect")
        {
            return Err(USAGE.into());
        }
        let mut values = BTreeMap::new();
        let mut i = 2;
        while i < args.len() {
            let key = args[i].as_str();
            if !["--pack", "--scenario", "--nation", "--config"].contains(&key) {
                return Err(format!(
                    "unsupported supply-network argument {key}; {USAGE}"
                ));
            }
            let v = args
                .get(i + 1)
                .filter(|v| !v.is_empty() && !v.starts_with("--"))
                .ok_or_else(|| format!("missing {key}; {USAGE}"))?;
            if values.insert(key, v.as_str()).is_some() {
                return Err(format!("duplicate {key}"));
            }
            i += 2;
        }
        let required = |k: &str| {
            values
                .get(k)
                .copied()
                .ok_or_else(|| format!("missing {k}; {USAGE}"))
        };
        let scenario = required("--scenario")?;
        if scenario.len() > 64
            || !scenario
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err("invalid scenario ID".into());
        }
        let n = required("--nation")?;
        let nation = n
            .parse::<u16>()
            .map_err(|_| "invalid nation ID: expected canonical u16")?;
        if nation.to_string() != n {
            return Err("invalid nation ID: expected canonical u16".into());
        }
        Ok(Self {
            pack: required("--pack")?.into(),
            scenario: scenario.into(),
            nation: NationId(nation),
            config: required("--config")?.into(),
        })
    }
}
/// Reuse the approved pack-validation/initial World loader; no Simulation is constructed.
pub fn inspect(options: &Options) -> Result<Value, String> {
    let config = oh_data::supply_network::read_file(&options.config)?;
    let (loaded, world) = crate::load_national(&options.pack, &options.scenario)?;
    let network = oh_sim::supply::config_adapter::prepare(
        &world,
        &loaded.pack.defines,
        options.nation,
        &config,
    )?;
    let feeders = network
        .feeders()
        .map_err(|e| format!("network connectivity: {e:?}"))?;
    let metadata = &config.nations()[&options.nation.0];
    let nodes=network.nodes.iter().map(|(&id,n)| {
        let p=world.province(id).expect("validated network province");
        let definition=world.defs().map().provinces.iter().find(|p|p.id==id.0).expect("validated map province");
        json!({"province":id.0,"land":n.land,"controller":n.controller.map(|n|n.0),"owner":p.owner().map(|n|n.0),"state":p.state().map(|s|s.0),"terrain":definition.terrain,"coastal":definition.coastal})
    }).collect::<Vec<_>>();
    let land=network.land.iter().map(|(&(a,b),e)|json!({"a":a.0,"b":b.0,"kind":format!("{:?}",e.kind),"a_to_b_hours_fx_bits":e.cost.to_bits(),"b_to_a_hours_fx_bits":e.reverse_cost.to_bits()})).collect::<Vec<_>>();
    let rails=metadata.rails.iter().map(|r|{
        let rail=&network.rails[&(r.a.into(),r.b.into())];
        json!({"a":r.a,"b":r.b,"level":r.level,"shared_capacity_qty_bits":rail.capacity.to_bits(),"a_to_b_hours_fx_bits":rail.cost.to_bits(),"b_to_a_hours_fx_bits":rail.reverse_cost.to_bits()})
    }).collect::<Vec<_>>();
    let sources=network.sources.iter().map(|s|json!({"id":s.id.0,"province":s.province.0,"kind":match s.kind {oh_sim::supply::SourceKind::Capital=>"capital",oh_sim::supply::SourceKind::Hub=>"hub",oh_sim::supply::SourceKind::Port=>"port"},"capacity_qty_bits":s.capacity.to_bits(),"feeder":feeders[&s.id].as_ref().map(|p|p.iter().map(|p|p.0).collect::<Vec<_>>())})).collect::<Vec<_>>();
    let states=world.inputs().states().iter().map(|s|json!({"state":s.id().0,"current_infrastructure_fx_bits":s.infrastructure().to_bits()})).collect::<Vec<_>>();
    let infrastructure = config
        .tuning()
        .infrastructure
        .iter()
        .map(|(l, f)| json!({"level_fx_bits":l.to_bits(),"time_factor_fx_bits":f.to_bits()}))
        .collect::<Vec<_>>();
    let capacity = config
        .tuning()
        .rail_capacity
        .iter()
        .map(|(l, c)| json!({"level":l,"capacity_qty_bits":c.to_bits()}))
        .collect::<Vec<_>>();
    Ok(json!({
        "scope":"initial_world_network",
        "provenance":{"sources":"explicit_external_static_metadata","rails":"explicit_external_static_metadata_not_construction_state","infrastructure":"exact_caller_coefficients_for_actual_current_levels","config_identity_scope":"normalized_diagnostic_only","config_identity":format!("{:016x}",config.identity()?),"policy":oh_data::supply_network::POLICY,"pack_id":loaded.pack.manifest.id,"scenario":options.scenario},
        "units":{"fx_bits":"signed_I32F32","qty_bits":"signed_I48F16","cost":"reference_travel_hours","capacity":"explicit_supply_units_per_day"},
        "nation":network.nation.0,"capital":network.capital.0,
        "tuning":{"reference_speed_kmh_fx_bits":config.tuning().reference_speed_kmh.to_bits(),"decay_per_reference_hour_fx_bits":network.decay.to_bits(),"infrastructure":infrastructure,"rail_capacity":capacity},
        "nodes":nodes,"land":land,"rails":rails,"sources":sources,"states":states
    }))
}
pub fn execute(args: &[String]) -> Result<(), String> {
    let output = inspect(&Options::parse(args)?)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
    );
    Ok(())
}
