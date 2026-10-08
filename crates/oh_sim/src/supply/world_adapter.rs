//! Pure network preparation. No division-demand adapter, state mutation or daily supply producer.
use super::*;
use crate::world::World;
use oh_data::Defines;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportTuning {
    pub reference_speed_kmh: Fx,
    pub decay_per_reference_hour: Fx,
    /// Exact current infrastructure LEVEL -> positive travel-time FACTOR. No fallback/interpolation.
    pub infrastructure_factors: BTreeMap<Fx, Fx>,
    pub rail_capacity_by_level: BTreeMap<u32, Qty>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticRail {
    pub edge: RailKey,
    pub level: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticMetadata {
    pub sources: Vec<Source>,
    pub rails: Vec<StaticRail>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidValue,
    InvalidReference,
    Duplicate,
    MissingContext,
    UnsupportedBuildingInstance,
    Overflow,
    Graph(Error),
}
fn movement_error(e: crate::movement::MovementError) -> AdapterError {
    use crate::movement::MovementError::*;
    match e {
        MissingContext => AdapterError::MissingContext,
        Overflow => AdapterError::Overflow,
        InvalidValue => AdapterError::InvalidValue,
        _ => AdapterError::InvalidReference,
    }
}
/// Builds ONLY current network inputs from actual authority plus explicit static metadata.
/// Infrastructure table and rail placements are trusted caller data, not implicit game policies.
pub fn prepare(
    world: &World,
    defines: &Defines,
    nation: NationId,
    tuning: &TransportTuning,
    metadata: &StaticMetadata,
) -> Result<Network, AdapterError> {
    if tuning.reference_speed_kmh <= Fx::ZERO
        || tuning.decay_per_reference_hour < Fx::ZERO
        || tuning
            .infrastructure_factors
            .iter()
            .any(|(level, factor)| *level < Fx::ZERO || *factor <= Fx::ZERO)
        || tuning
            .rail_capacity_by_level
            .iter()
            .any(|(level, capacity)| *level == 0 || *capacity <= Qty::ZERO)
    {
        return Err(AdapterError::InvalidValue);
    }
    if tuning.infrastructure_factors.is_empty() {
        return Err(AdapterError::MissingContext);
    }
    if world.nation(nation).is_none() {
        return Err(AdapterError::InvalidReference);
    }
    let definition = world
        .defs()
        .nations()
        .iter()
        .find(|n| n.id == nation.0)
        .ok_or(AdapterError::InvalidReference)?;
    let map = world.defs().map();
    let mut network = Network {
        nation,
        capital: ProvinceId(definition.capital),
        nodes: BTreeMap::new(),
        land: BTreeMap::new(),
        rails: BTreeMap::new(),
        sources: metadata.sources.clone(),
        decay: tuning.decay_per_reference_hour,
    };
    for province in &map.provinces {
        let actual = world
            .province(ProvinceId(province.id))
            .ok_or(AdapterError::InvalidReference)?;
        let land = province.kind == oh_data::map::ProvinceKind::Land;
        if land && actual.state().and_then(|id| world.state(id)).is_none() {
            return Err(AdapterError::InvalidReference);
        }
        if network
            .nodes
            .insert(
                actual.id(),
                Node {
                    controller: actual.controller(),
                    land,
                },
            )
            .is_some()
        {
            return Err(AdapterError::Duplicate);
        }
    }
    let cost = |from: ProvinceId, to: ProvinceId, distance: Fx| -> Result<Fx, AdapterError> {
        let state = world
            .province(to)
            .and_then(|p| p.state())
            .and_then(|id| world.state(id))
            .ok_or(AdapterError::InvalidReference)?;
        let infrastructure = *tuning
            .infrastructure_factors
            .get(&state.infrastructure())
            .ok_or(AdapterError::MissingContext)?;
        let factors =
            crate::movement::Factors::from_defines(defines, map, from, to, infrastructure, Fx::ONE)
                .map_err(movement_error)?;
        crate::formula::movement_hours(distance, tuning.reference_speed_kmh, factors)
            .map_err(movement_error)
    };
    for edge in &map.edges {
        let key = (ProvinceId(edge.a), ProvinceId(edge.b));
        if edge.a >= edge.b
            || !network.nodes.contains_key(&key.0)
            || !network.nodes.contains_key(&key.1)
        {
            return Err(AdapterError::InvalidReference);
        }
        if edge.distance_km <= Fx::ZERO {
            return Err(AdapterError::InvalidValue);
        }
        let kind = match edge.kind {
            oh_data::map::EdgeKind::Normal => LandKind::Normal,
            oh_data::map::EdgeKind::RiverSmall => LandKind::RiverSmall,
            oh_data::map::EdgeKind::RiverLarge => LandKind::RiverLarge,
            _ => continue,
        };
        if !network.nodes[&key.0].land || !network.nodes[&key.1].land {
            continue;
        }
        let forward = cost(key.0, key.1, edge.distance_km)?;
        let reverse = cost(key.1, key.0, edge.distance_km)?;
        if network
            .land
            .insert(
                key,
                LandEdge {
                    cost: forward,
                    reverse_cost: reverse,
                    kind,
                },
            )
            .is_some()
        {
            return Err(AdapterError::Duplicate);
        }
    }
    for source in &network.sources {
        let province = map
            .provinces
            .iter()
            .find(|p| p.id == source.province.0)
            .ok_or(AdapterError::InvalidReference)?;
        if province.kind != oh_data::map::ProvinceKind::Land
            || world
                .province(source.province)
                .and_then(|p| p.state())
                .and_then(|s| world.state(s))
                .is_none()
        {
            return Err(AdapterError::InvalidReference);
        }
        if source.kind == SourceKind::Port && !province.coastal {
            return Err(AdapterError::InvalidReference);
        }
        if source.kind != SourceKind::Capital {
            return Err(AdapterError::UnsupportedBuildingInstance);
        }
    }
    for rail in &metadata.rails {
        if rail.level == 0 {
            return Err(AdapterError::InvalidValue);
        }
        let edge = network
            .land
            .get(&rail.edge)
            .ok_or(AdapterError::InvalidReference)?;
        let capacity = *tuning
            .rail_capacity_by_level
            .get(&rail.level)
            .ok_or(AdapterError::MissingContext)?;
        if network
            .rails
            .insert(
                rail.edge,
                Rail {
                    cost: edge.cost,
                    reverse_cost: edge.reverse_cost,
                    capacity,
                },
            )
            .is_some()
        {
            return Err(AdapterError::Duplicate);
        }
    }
    graph::validate(&network).map_err(AdapterError::Graph)?;
    Ok(network)
}
