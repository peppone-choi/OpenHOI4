//! Geometry is validated at host load; HTTP never contacts external map services.
use crate::{Host, positive};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use oh_proto::{MapMetadata, MapStyle};
use std::collections::BTreeMap;
fn setting(host: &Host, key: &str) -> Result<u32, String> {
    u32::try_from(positive(
        &host.loaded.pack.defines,
        &format!("map_display.{key}"),
    )?)
    .map_err(|e| e.to_string())
}
fn rgb(host: &Host, key: &str) -> Result<[u8; 3], String> {
    let n = match host.loaded.pack.defines.get(&format!("map_display.{key}")) {
        Some(oh_data::DefineValue::Number(oh_data::Number::Integer(n))) if *n >= 0 => {
            u32::try_from(*n).map_err(|e| e.to_string())?
        }
        _ => {
            return Err(format!(
                "map_display.{key} must be a nonnegative RGB24 integer"
            ));
        }
    };
    if n > 0xffffff {
        return Err(format!("map_display.{key} exceeds RGB24"));
    }
    Ok([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}
pub fn validate(host: &Host) -> Result<(), String> {
    if let Some(world) = host.world.as_ref() {
        document(host, world.defs().map_id())?;
    }
    Ok(())
}
fn document(host: &Host, id: &str) -> Result<MapMetadata, String> {
    let w = host.world.as_ref().ok_or("map-unavailable")?;
    if id != w.defs().map_id() {
        return Err("map-unavailable".into());
    }
    let map = w.defs().map();
    let style = MapStyle {
        background: rgb(host, "background")?,
        nation_border: rgb(host, "nation_border")?,
        state_border: rgb(host, "state_border")?,
        province_border: rgb(host, "province_border")?,
        selected: rgb(host, "selected")?,
        hovered: rgb(host, "hovered")?,
        nation_width_milli: setting(host, "nation_width_milli")?,
        state_width_milli: setting(host, "state_width_milli")?,
        province_width_milli: setting(host, "province_width_milli")?,
        highlight_milli: setting(host, "highlight_milli")?,
        fit_milli: setting(host, "fit_milli")?,
        zoom_min_milli: setting(host, "zoom_min_milli")?,
        zoom_max_milli: setting(host, "zoom_max_milli")?,
        wheel_milli: setting(host, "wheel_milli")?,
        drag_threshold: setting(host, "drag_threshold")?,
    };
    if style.zoom_min_milli > style.zoom_max_milli
        || style.fit_milli > 1000
        || style.highlight_milli > 1000
    {
        return Err("invalid map display limits".into());
    }
    let bytes = map.index_le_bytes();
    // Hash raw bytes directly; FNV-1a matches the engine hash algorithm.
    let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    Ok(MapMetadata {
        schema_version: oh_proto::MAP_DISPLAY_VERSION,
        map_id: id.into(),
        width: map.width,
        height: map.height,
        province_ids: map.provinces.iter().map(|p| p.id).collect(),
        pack_hash: host.pack.hash.clone(),
        index_hash: format!("{hash:016x}"),
        byte_length: bytes.len().to_string(),
        style,
    })
}
pub async fn metadata(State(host): State<Host>, Path(id): Path<String>) -> Response {
    match document(&host, &id) {
        Ok(value) => ([("cache-control", "no-cache")], Json(value)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
pub async fn index(
    State(host): State<Host>,
    Path(id): Path<String>,
    Query(query): Query<BTreeMap<String, String>>,
) -> Response {
    let Ok(meta) = document(&host, &id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if query.get("pack").is_none_or(|hash| hash != &meta.pack_hash) {
        return StatusCode::CONFLICT.into_response();
    }
    let mut headers = HeaderMap::new();
    for (key, value) in [
        ("content-type", "application/octet-stream".to_owned()),
        (
            "cache-control",
            "private, max-age=31536000, immutable".to_owned(),
        ),
        ("x-map-width", meta.width.to_string()),
        ("x-map-height", meta.height.to_string()),
        ("x-pack-hash", meta.pack_hash),
        ("x-index-hash", meta.index_hash),
    ] {
        headers.insert(
            axum::http::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            value.parse().unwrap(),
        );
    }
    (
        headers,
        host.world.as_ref().unwrap().defs().map().index_le_bytes(),
    )
        .into_response()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map_metadata_matches_validated_dense_bytes_and_pack_identity() {
        let (_, shutdown) = tokio::sync::watch::channel(false);
        let host = Host::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs"),
            shutdown,
        )
        .unwrap();
        let meta = document(&host, "testland").unwrap();
        assert_eq!(
            (meta.width, meta.height, meta.byte_length.as_str()),
            (8, 6, "96")
        );
        assert_eq!(meta.province_ids, [10, 20, 30, 40, 50, 60]);
        assert_eq!(meta.pack_hash, host.pack.hash);
        assert!(document(&host, "../testland").is_err());
    }
    #[test]
    fn display_settings_accept_black_and_reject_missing_or_invalid_before_http() {
        let (_, shutdown) = tokio::sync::watch::channel(false);
        let mut host = Host::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs"),
            shutdown,
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut host.loaded)
            .pack
            .defines
            .0
            .get_mut("map_display")
            .unwrap()
            .insert(
                "nation_border".into(),
                oh_data::DefineValue::Number(oh_data::Number::Integer(0)),
            );
        assert_eq!(rgb(&host, "nation_border").unwrap(), [0, 0, 0]);
        assert!(validate(&host).is_ok());
        std::sync::Arc::make_mut(&mut host.loaded)
            .pack
            .defines
            .0
            .get_mut("map_display")
            .unwrap()
            .remove("fit_milli");
        assert!(validate(&host).is_err());
    }
}
