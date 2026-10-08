use super::*;
use std::collections::BTreeSet;
const Q: i128 = 1i128 << 32; // Fx representation scale, not a game coefficient.
fn add(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_add(b).ok_or(Error::Overflow)
}
fn mul(a: i128, b: i128) -> Result<i128, Error> {
    a.checked_mul(b).ok_or(Error::Overflow)
}
fn qty(raw: i128) -> Result<Qty, Error> {
    Ok(Qty::from_bits(
        i64::try_from(raw).map_err(|_| Error::Overflow)?,
    ))
}
fn ratio(raw: i128) -> Result<Fx, Error> {
    Ok(Fx::from_bits(
        i64::try_from(raw).map_err(|_| Error::Overflow)?,
    ))
}
#[derive(Clone)]
struct Route {
    source: SourceId,
    distance: Fx,
    attenuation: Fx,
    weight: i128,
}
struct Province {
    demand: Qty,
    reason: Reason,
    routes: Vec<Route>,
}
struct SourceInfo {
    capacity: Qty,
    feeder: Option<Vec<ProvinceId>>,
}
#[derive(Default)]
struct Trial {
    sources: BTreeMap<SourceId, i128>,
    rails: BTreeMap<RailKey, i128>,
    shares: BTreeMap<(ProvinceId, SourceId), (i128, i128)>,
    delivered: BTreeMap<ProvinceId, i128>,
}
fn evaluate(
    provinces: &BTreeMap<ProvinceId, Province>,
    sources: &BTreeMap<SourceId, SourceInfo>,
    targets: &BTreeMap<ProvinceId, i128>,
) -> Result<Trial, Error> {
    let mut t = Trial::default();
    for (&pid, p) in provinces {
        let desired = mul(i128::from(p.demand.to_bits()), targets[&pid])? / Q;
        let mut delivered = 0;
        for r in &p.routes {
            let y = mul(desired, r.weight)? / Q;
            let a = i128::from(r.attenuation.to_bits());
            let x = if y == 0 {
                0
            } else {
                add(mul(y, Q)?, a - 1)? / a
            };
            delivered = add(delivered, y)?;
            let used = t.sources.entry(r.source).or_default();
            *used = add(*used, x)?;
            let feeder = sources[&r.source].feeder.as_ref().ok_or(Error::Invariant)?;
            for pair in feeder.windows(2) {
                let used = t.rails.entry(graph::key(pair[0], pair[1])).or_default();
                *used = add(*used, x)?;
            }
            t.shares.insert((pid, r.source), (x, y));
        }
        t.delivered.insert(pid, delivered);
    }
    Ok(t)
}
fn violations(
    t: &Trial,
    sources: &BTreeMap<SourceId, SourceInfo>,
    input: &Input,
) -> (BTreeSet<SourceId>, BTreeSet<RailKey>) {
    let ss = t
        .sources
        .iter()
        .filter(|(id, x)| **x > i128::from(sources[id].capacity.to_bits()))
        .map(|(&id, _)| id)
        .collect();
    let es = t
        .rails
        .iter()
        .filter(|(key, x)| **x > i128::from(input.rails[key].capacity.to_bits()))
        .map(|(&key, _)| key)
        .collect();
    (ss, es)
}
fn feasible(t: &Trial, sources: &BTreeMap<SourceId, SourceInfo>, input: &Input) -> bool {
    let (s, e) = violations(t, sources, input);
    s.is_empty() && e.is_empty()
}
fn trial_targets(
    targets: &BTreeMap<ProvinceId, i128>,
    active: &BTreeSet<ProvinceId>,
    delta: i128,
) -> BTreeMap<ProvinceId, i128> {
    targets
        .iter()
        .map(|(&id, &r)| (id, r + if active.contains(&id) { delta } else { 0 }))
        .collect()
}

/// Pure, transactional calculation from explicit per-nation graph and real-demand inputs.
/// Does not create division state, apply movement/combat effects, save, or expose a server query.
pub fn calculate(input: &Input) -> Result<Day, Error> {
    graph::validate(input)?;
    let mut sources = BTreeMap::new();
    let mut source_distances = BTreeMap::new();
    for s in &input.sources {
        let feeder = graph::feeder(input, s.province)?;
        if feeder.is_some() {
            source_distances.insert(s.id, graph::distances(input, s.province)?);
        }
        sources.insert(
            s.id,
            SourceInfo {
                capacity: s.capacity,
                feeder,
            },
        );
    }
    let mut demands = BTreeMap::<ProvinceId, Qty>::new();
    for d in &input.demands {
        let v = demands.entry(d.province).or_default();
        *v = v.checked_add(d.amount).ok_or(Error::Overflow)?;
    }
    let mut provinces = BTreeMap::new();
    for (pid, demand) in demands {
        let mut routes = Vec::new();
        let mut reachable = false;
        let mut positive_attenuation = false;
        let mut weighted = Vec::new();
        let mut total = 0;
        for (&sid, s) in &sources {
            let Some(&distance) = source_distances.get(&sid).and_then(|d| d.get(&pid)) else {
                continue;
            };
            reachable = true;
            let loss = input.decay.checked_mul(distance).ok_or(Error::Overflow)?;
            let attenuation = (Fx::ONE - loss.min(Fx::ONE)).max(Fx::ZERO);
            if attenuation == Fx::ZERO {
                continue;
            }
            positive_attenuation = true;
            if s.capacity == Qty::ZERO {
                continue;
            }
            let value = mul(
                i128::from(s.capacity.to_bits()),
                i128::from(attenuation.to_bits()),
            )?;
            total = add(total, value)?;
            weighted.push((sid, distance, attenuation, value));
        }
        let reason = if demand == Qty::ZERO {
            Reason::ZeroDemand
        } else if !reachable {
            Reason::Isolated
        } else if !positive_attenuation {
            Reason::OutOfRange
        } else if total == 0 {
            Reason::NoCapacity
        } else {
            Reason::Allocated
        };
        if total > 0 {
            let mut weights = Vec::new();
            let mut sum = 0;
            for &(sid, distance, attenuation, value) in &weighted {
                let scaled = mul(value, Q)?;
                let w = scaled / total;
                sum = add(sum, w)?;
                weights.push((sid, distance, attenuation, w, scaled % total));
            }
            let mut order: Vec<_> = (0..weights.len()).collect();
            order.sort_by(|&a, &b| {
                weights[b]
                    .4
                    .cmp(&weights[a].4)
                    .then_with(|| weights[a].0.cmp(&weights[b].0))
            });
            let residual = usize::try_from(Q - sum).map_err(|_| Error::Invariant)?;
            if residual > order.len() {
                return Err(Error::Invariant);
            }
            for &i in order.iter().take(residual) {
                weights[i].3 += 1;
            }
            for (source, distance, attenuation, weight, _) in weights {
                if weight > 0 {
                    routes.push(Route {
                        source,
                        distance,
                        attenuation,
                        weight,
                    });
                }
            }
        }
        provinces.insert(
            pid,
            Province {
                demand,
                reason,
                routes,
            },
        );
    }
    let mut targets: BTreeMap<_, _> = provinces.keys().map(|&id| (id, 0)).collect();
    let mut active: BTreeSet<_> = provinces
        .iter()
        .filter(|(_, p)| p.reason == Reason::Allocated)
        .map(|(&id, _)| id)
        .collect();
    while !active.is_empty() {
        let room = active
            .iter()
            .map(|id| Q - targets[id])
            .min()
            .ok_or(Error::Invariant)?;
        let (mut low, mut high) = (0, room);
        while low < high {
            let mid = low + (high - low + 1) / 2;
            let trial = evaluate(&provinces, &sources, &trial_targets(&targets, &active, mid))?;
            if feasible(&trial, &sources, input) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }
        for id in &active {
            *targets.get_mut(id).ok_or(Error::Invariant)? += low;
        }
        let mut frozen: BTreeSet<_> = active
            .iter()
            .filter(|id| targets[id] == Q)
            .copied()
            .collect();
        let remaining: BTreeSet<_> = active.difference(&frozen).copied().collect();
        if !remaining.is_empty() {
            let trial = evaluate(
                &provinces,
                &sources,
                &trial_targets(&targets, &remaining, 1),
            )?;
            let (bad_sources, bad_rails) = violations(&trial, &sources, input);
            for &id in &remaining {
                if provinces[&id].routes.iter().any(|r| {
                    bad_sources.contains(&r.source)
                        || sources[&r.source].feeder.as_ref().is_some_and(|path| {
                            path.windows(2)
                                .any(|pair| bad_rails.contains(&graph::key(pair[0], pair[1])))
                        })
                }) {
                    frozen.insert(id);
                }
            }
        }
        if frozen.is_empty() {
            return Err(Error::Invariant);
        }
        for id in frozen {
            active.remove(&id);
        }
    }
    let trial = evaluate(&provinces, &sources, &targets)?;
    if !feasible(&trial, &sources, input) {
        return Err(Error::Invariant);
    }
    let mut day = Day {
        sources: BTreeMap::new(),
        rails: BTreeMap::new(),
        provinces: BTreeMap::new(),
    };
    for (&sid, s) in &sources {
        let x = trial.sources.get(&sid).copied().unwrap_or(0);
        let y = trial
            .shares
            .iter()
            .filter(|((_, id), _)| *id == sid)
            .try_fold(0, |acc, (_, (_, y))| add(acc, *y))?;
        day.sources.insert(
            sid,
            SourceLedger {
                feeder: s.feeder.clone(),
                dispatch: qty(x)?,
                delivered: qty(y)?,
                loss: qty(x - y)?,
                unused: qty(i128::from(s.capacity.to_bits()) - x)?,
            },
        );
    }
    for (&key, r) in &input.rails {
        let x = trial.rails.get(&key).copied().unwrap_or(0);
        day.rails.insert(
            key,
            RailLedger {
                dispatch: qty(x)?,
                unused: qty(i128::from(r.capacity.to_bits()) - x)?,
            },
        );
    }
    for (&pid, p) in &provinces {
        let y = trial.delivered[&pid];
        let desired = mul(i128::from(p.demand.to_bits()), targets[&pid])? / Q;
        let coverage = if p.demand == Qty::ZERO {
            Q
        } else {
            mul(y, Q)? / i128::from(p.demand.to_bits())
        };
        let mut shares = Vec::new();
        for r in &p.routes {
            let (x, y) = trial.shares[&(pid, r.source)];
            shares.push(Share {
                source: r.source,
                distance: r.distance,
                attenuation: r.attenuation,
                weight: ratio(r.weight)?,
                dispatch: qty(x)?,
                delivered: qty(y)?,
            });
        }
        day.provinces.insert(
            pid,
            ProvinceLedger {
                demand: p.demand,
                delivered: qty(y)?,
                ratio: ratio(coverage)?,
                target: ratio(targets[&pid])?,
                rounding_residual: qty(desired - y)?,
                reason: p.reason,
                shares,
            },
        );
    }
    Ok(day)
}

#[cfg(test)]
mod monotonic_tests {
    use super::*;
    #[test]
    fn monotonic_floor_split_and_ceil_dispatch_never_hamilton_reallocate() {
        // Exhaust raw target increments with unequal fixed weights and attenuation.
        let pid = ProvinceId(0);
        let sid = SourceId(0);
        for demand in 1..40 {
            for weight in [1, Q / 3, Q / 2, Q - 1, Q] {
                for a in [1, Q / 3, Q / 2, Q] {
                    let provinces = BTreeMap::from([(
                        pid,
                        Province {
                            demand: Qty::from_bits(demand),
                            reason: Reason::Allocated,
                            routes: vec![Route {
                                source: sid,
                                distance: Fx::ZERO,
                                attenuation: Fx::from_bits(a as i64),
                                weight,
                            }],
                        },
                    )]);
                    let sources = BTreeMap::from([(
                        sid,
                        SourceInfo {
                            capacity: Qty::from_bits(i64::MAX),
                            feeder: Some(vec![pid]),
                        },
                    )]);
                    let (mut last_x, mut last_y) = (0, 0);
                    for i in 0..=128 {
                        let t =
                            evaluate(&provinces, &sources, &BTreeMap::from([(pid, i * Q / 128)]))
                                .unwrap();
                        let (x, y) = t.shares[&(pid, sid)];
                        assert!(x >= last_x && y >= last_y);
                        last_x = x;
                        last_y = y;
                    }
                }
            }
        }
    }
}
