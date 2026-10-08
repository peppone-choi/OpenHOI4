//! External diagnostic configuration into real World network inputs only.
use super::{
    Network, Source, SourceId, SourceKind,
    world_adapter::{self, StaticMetadata, StaticRail, TransportTuning},
};
use oh_core::{NationId, ProvinceId};
/// Revalidate every configured nation against actual World definitions, then prepare
/// the selected network from actual current inputs. No demand or daily result is created.
pub fn prepare(
    world: &crate::world::World,
    defines: &oh_data::Defines,
    nation: NationId,
    config: &oh_data::supply_network::Config,
) -> Result<Network, String> {
    config.validate_against(world.defs().nations(), world.defs().map())?;
    let n = config
        .nations()
        .get(&nation.0)
        .ok_or("MissingContext: explicit config nation")?;
    let t = config.tuning();
    // Even an empty/isolated static graph must carry complete actual initial context.
    for state in world.inputs().states() {
        if !t.infrastructure.contains_key(&state.infrastructure()) {
            return Err(format!(
                "MissingContext: exact current infrastructure factor for state {}",
                state.id()
            ));
        }
    }
    let tuning = TransportTuning {
        reference_speed_kmh: t.reference_speed_kmh,
        decay_per_reference_hour: t.decay_per_reference_hour,
        infrastructure_factors: t.infrastructure.clone(),
        rail_capacity_by_level: t.rail_capacity.clone(),
    };
    let metadata = StaticMetadata {
        sources: n
            .sources
            .iter()
            .map(|s| Source {
                id: SourceId(s.id),
                province: ProvinceId(s.province),
                capacity: s.capacity,
                kind: match s.kind {
                    oh_data::supply_network::SourceKind::Capital => SourceKind::Capital,
                    oh_data::supply_network::SourceKind::Hub => SourceKind::Hub,
                    oh_data::supply_network::SourceKind::Port => SourceKind::Port,
                },
            })
            .collect(),
        rails: n
            .rails
            .iter()
            .map(|r| StaticRail {
                edge: (ProvinceId(r.a), ProvinceId(r.b)),
                level: r.level,
            })
            .collect(),
    };
    world_adapter::prepare(world, defines, nation, &tuning, &metadata)
        .map_err(|e| format!("world network: {e:?}"))
}
