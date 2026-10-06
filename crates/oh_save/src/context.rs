use crate::{PackV1, Result};
use oh_data::{DefineValue, Defines, Number};
use oh_sim::{Date, Simulation, TimeConfig, save_state::RestoreContext, world::World};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone)]
pub struct SaveContext {
    pub(crate) restore: RestoreContext,
    pub(crate) pack: PackV1,
    pub(crate) defines_hash: u64,
    pub(crate) root: PathBuf,
    time: TimeConfig,
}
pub fn parse_date(text: &str) -> Result<Date> {
    let parts: Vec<_> = text.split('-').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
    {
        return Err("InvalidDate: expected YYYY-MM-DD".into());
    }
    Date::new(
        parts[0].parse().map_err(|_| "InvalidDate: year")?,
        parts[1].parse().map_err(|_| "InvalidDate: month")?,
        parts[2].parse().map_err(|_| "InvalidDate: day")?,
    )
    .map_err(|e| e.to_string())
}
impl SaveContext {
    pub fn restore_context(&self) -> &RestoreContext {
        &self.restore
    }
    pub fn pack(&self) -> &PackV1 {
        &self.pack
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn defines_hash(&self) -> u64 {
        self.defines_hash
    }
    pub fn national(root: &Path, id: &str) -> Result<Self> {
        let before = pack_hash(root)?;
        let loaded = oh_data::national::load_scenario(root, id).map_err(|e| e.to_string())?;
        let world = World::from_loaded(&loaded)?;
        Self::build(
            root,
            loaded.pack,
            id,
            parse_date(&loaded.scenario.start_date)?,
            Some(world),
            before,
        )
    }
    pub fn m0(root: &Path, id: &str) -> Result<Self> {
        let before = pack_hash(&root.join(id))?;
        let loaded = oh_data::m0::load_m0_scenario(root, id)?;
        Self::build(
            &root.join(id),
            loaded.pack,
            id,
            parse_date(&loaded.start_date)?,
            None,
            before,
        )
    }
    fn build(
        root: &Path,
        pack: oh_data::DataPack,
        id: &str,
        start_date: Date,
        world: Option<World>,
        before: u64,
    ) -> Result<Self> {
        let content_hash = pack_hash(root)?;
        if before != content_hash {
            return Err("PackChangedDuringLoad".into());
        }
        let time = TimeConfig::from_defines(&pack.defines).map_err(|e| e.to_string())?;
        Ok(Self {
            restore: RestoreContext {
                scenario: id.into(),
                start_date,
                world,
            },
            pack: PackV1 {
                id: pack.manifest.id,
                version: pack.manifest.version,
                content_hash,
            },
            defines_hash: effective_defines_hash(&pack.defines)?,
            root: root.canonicalize().map_err(|e| e.to_string())?,
            time,
        })
    }
    pub fn simulation(&self, seed: u64) -> Result<Simulation> {
        match self.restore.world.clone() {
            Some(world) => Simulation::with_world(
                self.restore.scenario.clone(),
                self.restore.start_date,
                seed,
                self.time.clone(),
                world,
            ),
            None => Simulation::new(
                self.restore.scenario.clone(),
                self.restore.start_date,
                seed,
                self.time.clone(),
            ),
        }
        .map_err(|e| e.to_string())
    }
    pub fn verify_unchanged(&self) -> Result<()> {
        if pack_hash(&self.root)? != self.pack.content_hash {
            return Err("PackChangedDuringLoad".into());
        }
        Ok(())
    }
}
pub fn pack_hash(root: &Path) -> Result<u64> {
    fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let m = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if m.file_type().is_symlink() || reparse(&m) {
                return Err("PackPath: symlink/reparse unsupported".into());
            }
            if m.is_dir() {
                walk(root, &path, files)?;
            } else if m.is_file() {
                let relative = path
                    .strip_prefix(root)
                    .map_err(|e| e.to_string())?
                    .components()
                    .map(|component| component.as_os_str().to_str().ok_or("PackPath: non-UTF8"))
                    .collect::<std::result::Result<Vec<_>, _>>()?
                    .join("/");
                files.push((
                    relative,
                    fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
                ));
            } else {
                return Err("PackPath: nonregular file".into());
            }
        }
        Ok(())
    }
    let m = fs::symlink_metadata(root).map_err(|e| e.to_string())?;
    if m.file_type().is_symlink() || reparse(&m) || !m.is_dir() {
        return Err("PackPath: root".into());
    }
    let mut files = Vec::new();
    walk(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    oh_core::state_hash(&files).map_err(|e| e.to_string())
}
pub(crate) fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}
pub fn effective_defines_hash(defines: &Defines) -> Result<u64> {
    #[derive(Serialize)]
    enum TaggedNumber {
        Integer(i64),
        Fixed(i64),
    }
    #[derive(Serialize)]
    enum TaggedValue {
        Scalar(TaggedNumber),
        Array(Vec<TaggedNumber>),
    }
    let number = |n: &Number| match n {
        Number::Integer(n) => TaggedNumber::Integer(*n),
        Number::Fixed(n) => TaggedNumber::Fixed(n.to_bits()),
    };
    let values: Vec<_> = defines
        .0
        .iter()
        .map(|(system, items)| {
            (
                system,
                items
                    .iter()
                    .map(|(item, v)| {
                        (
                            item,
                            match v {
                                DefineValue::Number(n) => TaggedValue::Scalar(number(n)),
                                DefineValue::Array(a) => {
                                    TaggedValue::Array(a.iter().map(number).collect())
                                }
                            },
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    oh_core::state_hash(&values).map_err(|e| e.to_string())
}
