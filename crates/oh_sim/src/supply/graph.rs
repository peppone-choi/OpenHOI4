use super::*;
use std::collections::BTreeSet;

pub(super) fn permitted(kind: LandKind) -> bool {
    matches!(
        kind,
        LandKind::Normal | LandKind::RiverSmall | LandKind::RiverLarge
    )
}
pub(super) fn controlled(input: &Network, id: ProvinceId) -> bool {
    input
        .nodes
        .get(&id)
        .is_some_and(|n| n.land && n.controller == Some(input.nation))
}
pub(super) fn validate(input: &Network) -> Result<(), Error> {
    if input.decay < Fx::ZERO {
        return Err(Error::InvalidValue);
    }
    if !input.nodes.get(&input.capital).is_some_and(|n| n.land) {
        return Err(Error::InvalidReference);
    }
    for (&(a, b), edge) in &input.land {
        if a >= b || !input.nodes.contains_key(&a) || !input.nodes.contains_key(&b) {
            return Err(Error::InvalidReference);
        }
        if edge.cost <= Fx::ZERO || edge.reverse_cost <= Fx::ZERO {
            return Err(Error::InvalidValue);
        }
    }
    for (&key, rail) in &input.rails {
        if !input.land.get(&key).is_some_and(|e| permitted(e.kind)) {
            return Err(Error::InvalidReference);
        }
        if !input.nodes[&key.0].land || !input.nodes[&key.1].land {
            return Err(Error::InvalidReference);
        }
        if rail.cost <= Fx::ZERO || rail.reverse_cost <= Fx::ZERO || rail.capacity < Qty::ZERO {
            return Err(Error::InvalidValue);
        }
    }
    let mut source_ids = BTreeSet::new();
    let mut placements = BTreeSet::new();
    for source in &input.sources {
        if !source_ids.insert(source.id) || !placements.insert(source.province) {
            return Err(Error::Duplicate);
        }
        if !input.nodes.get(&source.province).is_some_and(|n| n.land)
            || ((source.kind == SourceKind::Capital) != (source.province == input.capital))
        {
            return Err(Error::InvalidReference);
        }
        if source.capacity < Qty::ZERO {
            return Err(Error::InvalidValue);
        }
    }
    Ok(())
}

/// Positive edge costs make lexicographic path ties finite (no zero-cost cycles).
/// Costs are explicit, already-authoritative reference travel-hours inputs.
fn shortest(
    input: &Network,
    start: ProvinceId,
    rail_min: Option<Qty>,
) -> Result<BTreeMap<ProvinceId, (Fx, Vec<ProvinceId>)>, Error> {
    let mut labels = BTreeMap::new();
    if !controlled(input, start) {
        return Ok(labels);
    }
    labels.insert(start, (Fx::ZERO, vec![start]));
    let mut done = BTreeSet::new();
    loop {
        let next = labels
            .iter()
            .filter(|(id, _)| !done.contains(*id))
            .min_by(|a, b| a.1.cmp(b.1).then_with(|| a.0.cmp(b.0)))
            .map(|(&id, value)| (id, value.clone()));
        let Some((id, (cost, path))) = next else {
            break;
        };
        done.insert(id);
        for (&(a, b), land) in &input.land {
            let to = if a == id {
                b
            } else if b == id {
                a
            } else {
                continue;
            };
            if done.contains(&to) || !controlled(input, to) || !permitted(land.kind) {
                continue;
            }
            let edge_cost = if let Some(min) = rail_min {
                let Some(rail) = input.rails.get(&(a, b)) else {
                    continue;
                };
                if rail.capacity < min || rail.capacity == Qty::ZERO {
                    continue;
                }
                if id == a {
                    rail.cost
                } else {
                    rail.reverse_cost
                }
            } else {
                if id == a {
                    land.cost
                } else {
                    land.reverse_cost
                }
            };
            let total = cost.checked_add(edge_cost).ok_or(Error::Overflow)?;
            let mut route = path.clone();
            route.push(to);
            let candidate = (total, route);
            if labels.get(&to).is_none_or(|old| candidate < *old) {
                labels.insert(to, candidate);
            }
        }
    }
    Ok(labels)
}

/// Widest bottleneck, then shortest cost, then lexicographic full path.
/// Two stages avoid discarding a shorter prefix when a later edge lowers its bottleneck.
pub(super) fn feeder(
    input: &Network,
    destination: ProvinceId,
) -> Result<Option<Vec<ProvinceId>>, Error> {
    if !controlled(input, input.capital) || !controlled(input, destination) {
        return Ok(None);
    }
    if destination == input.capital {
        return Ok(Some(vec![destination]));
    }
    let mut width = BTreeMap::from([(input.capital, Qty::from_bits(i64::MAX))]);
    let mut done = BTreeSet::new();
    loop {
        let next = width
            .iter()
            .filter(|(id, _)| !done.contains(*id))
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            .map(|(&id, &w)| (id, w));
        let Some((id, w)) = next else { break };
        done.insert(id);
        for (&(a, b), rail) in &input.rails {
            let to = if a == id {
                b
            } else if b == id {
                a
            } else {
                continue;
            };
            if done.contains(&to) || !controlled(input, to) || rail.capacity == Qty::ZERO {
                continue;
            }
            let candidate = w.min(rail.capacity);
            if width.get(&to).is_none_or(|old| candidate > *old) {
                width.insert(to, candidate);
            }
        }
    }
    let Some(&minimum) = width.get(&destination) else {
        return Ok(None);
    };
    Ok(shortest(input, input.capital, Some(minimum))?
        .remove(&destination)
        .map(|(_, path)| path))
}
pub(super) fn distances(
    input: &Network,
    start: ProvinceId,
) -> Result<BTreeMap<ProvinceId, Fx>, Error> {
    Ok(shortest(input, start, None)?
        .into_iter()
        .map(|(id, (distance, _))| (id, distance))
        .collect())
}
pub(super) fn key(a: ProvinceId, b: ProvinceId) -> RailKey {
    (a.min(b), a.max(b))
}
