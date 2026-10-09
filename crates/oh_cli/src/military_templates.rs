//! Inspect component-declared normal stats with actual production-model binding checks.
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf};
pub const USAGE: &str = "military-template inspect --pack <root> --scenario <id> --nation <canonical-u16> --definitions <external.toml> --template <id> (normal component arithmetic only)";
#[derive(Debug, Eq, PartialEq)]
pub struct Options {
    pub pack: PathBuf,
    pub scenario: String,
    pub nation: u16,
    pub definitions: PathBuf,
    pub template: String,
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        if args.first().map(String::as_str) != Some("military-template")
            || args.get(1).map(String::as_str) != Some("inspect")
        {
            return Err(USAGE.into());
        }
        let mut values = BTreeMap::new();
        let mut i = 2;
        while i < args.len() {
            let key = args[i].as_str();
            if ![
                "--pack",
                "--scenario",
                "--nation",
                "--definitions",
                "--template",
            ]
            .contains(&key)
            {
                return Err(format!(
                    "unsupported military-template argument {key}; {USAGE}"
                ));
            }
            let value = args
                .get(i + 1)
                .filter(|v| !v.is_empty() && !v.starts_with("--"))
                .ok_or_else(|| format!("missing {key}; {USAGE}"))?;
            if values.insert(key, value.as_str()).is_some() {
                return Err(format!("duplicate {key}"));
            }
            i += 2;
        }
        let required = |key: &str| {
            values
                .get(key)
                .copied()
                .ok_or_else(|| format!("missing {key}; {USAGE}"))
        };
        let scenario = required("--scenario")?;
        let template = required("--template")?;
        if [scenario, template]
            .iter()
            .any(|id| id.len() > 64 || !oh_data::valid_id(id))
        {
            return Err("invalid bounded scenario/template ID".into());
        }
        let n = required("--nation")?;
        let nation = n
            .parse::<u16>()
            .map_err(|_| "invalid nation ID: canonical u16 required")?;
        if nation.to_string() != n {
            return Err("invalid nation ID: canonical u16 required".into());
        }
        Ok(Self {
            pack: required("--pack")?.into(),
            scenario: scenario.into(),
            nation,
            definitions: required("--definitions")?.into(),
            template: template.into(),
        })
    }
}
pub fn inspect(options: &Options) -> Result<Value, String> {
    let definitions = oh_data::military_templates::read_file(&options.definitions)?;
    let validated_hash = crate::validate_for_run(&options.pack)?;
    let loaded = oh_data::national::load_scenario(&options.pack, &options.scenario)
        .map_err(|e| e.to_string())?;
    crate::verify_validation_identity(&options.pack, validated_hash)?;
    definitions.validate_bindings(&loaded, options.nation, &options.template)?;
    let normal = oh_sim::military_templates::aggregate(&definitions, &options.template)?;
    let s = &normal.stats;
    let template = &definitions.templates()[&options.template];
    let production = loaded
        .production
        .as_ref()
        .expect("validated actual production");
    let bindings = template
        .bindings
        .iter()
        .map(|(family, model)| {
            let m = &production.models[model];
            json!({"family":family,"model":model,"name_key":m.name_key,"generation":m.generation})
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "scope":"normal_template_stats",
        "provenance":{"stats":"component_declared_resolved_normal_inputs","equipment":"requirements_with_validated_actual_production_bindings","identity_scope":"normalized_external_diagnostic_only","identity":format!("{:016x}",definitions.identity()?),"policy":oh_data::military_templates::POLICY,"pack_id":loaded.pack.manifest.id,"scenario":options.scenario,"game_editor_legality":"not_evaluated"},
        "nation":options.nation,"template":options.template,"composition":{"combat":template.combat,"support":template.support},"bindings":bindings,
        "units":{"qty_bits":"signed_I48F16","fx_bits":"signed_I32F32","strength_fire_defense_breakthrough_organization_armor_piercing":"declared_normal_points","frontage":"frontage_units","supply_use":"descriptive_supply_units_per_day","speed_kmh":"km_per_hour","manpower":"whole_personnel","equipment":"whole_items","weighted_remainders":"Qty_raw_numerator_remainder_over_total_manpower"},
        "stats":{"strength_qty_bits":s.strength.to_bits(),"soft_fire_qty_bits":s.soft_fire.to_bits(),"hard_fire_qty_bits":s.hard_fire.to_bits(),"defense_qty_bits":s.defense.to_bits(),"breakthrough_qty_bits":s.breakthrough.to_bits(),"frontage_qty_bits":s.frontage.to_bits(),"supply_use_qty_bits":s.supply_use.to_bits(),"organization_qty_bits":s.organization.to_bits(),"armor_qty_bits":s.armor.to_bits(),"piercing_qty_bits":s.piercing.to_bits(),"speed_kmh_fx_bits":s.speed_kmh.to_bits()},
        "manpower":normal.manpower,"equipment_requirements":normal.equipment,"weighted_remainders":normal.weighted_remainders
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
