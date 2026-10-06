use crate::{Limits, Result, SaveContext};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
pub struct SaveOutcome {
    pub durability_warning: Option<String>,
}
pub fn read_file(path: &Path, limits: &Limits) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .take(
            limits
                .file_max_bytes
                .checked_add(1)
                .ok_or("Limit: file cap")?,
        )
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limits.file_max_bytes {
        return Err("Limit: file".into());
    }
    Ok(bytes)
}
pub fn write_atomic(path: &Path, bytes: &[u8], context: &SaveContext) -> Result<SaveOutcome> {
    write_transaction(path, context, |file| {
        file.write_all(bytes).map_err(|e| e.to_string())
    })
}
fn write_transaction(
    path: &Path,
    context: &SaveContext,
    write: impl FnOnce(&mut fs::File) -> Result<()>,
) -> Result<SaveOutcome> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = parent
        .canonicalize()
        .map_err(|e| format!("SavePath: {e}"))?;
    let target = parent.join(path.file_name().ok_or("SavePath: filename")?);
    if target.starts_with(&context.root) {
        return Err("SavePath: destination inside pack".into());
    }
    if let Ok(m) = fs::symlink_metadata(&target)
        && (m.file_type().is_symlink() || crate::context::reparse(&m) || !m.is_file())
    {
        return Err("SavePath: nonregular target".into());
    }
    let mut temp = tempfile::NamedTempFile::new_in(&parent).map_err(|e| format!("SaveIO: {e}"))?;
    let prepared = (|| {
        write(temp.as_file_mut())?;
        temp.as_file_mut()
            .flush()
            .map_err(|e| format!("SaveIO: flush {e}"))?;
        temp.as_file()
            .sync_all()
            .map_err(|e| format!("SaveIO: sync {e}"))?;
        Ok(())
    })();
    if let Err(error) = prepared {
        return Err(cleanup(temp, error));
    }
    if let Err(error) = temp.persist(&target) {
        return Err(cleanup(
            error.file,
            format!("SaveIO: commit {}", error.error),
        ));
    }
    #[cfg(unix)]
    let warning = fs::File::open(&parent)
        .and_then(|f| f.sync_all())
        .err()
        .map(|e| format!("committed; directory sync: {e}"));
    #[cfg(not(unix))]
    let warning = Some("committed; directory durability not confirmed on this platform".into());
    Ok(SaveOutcome {
        durability_warning: warning,
    })
}
fn cleanup(temp: tempfile::NamedTempFile, error: String) -> String {
    match temp.close() {
        Ok(()) => error,
        Err(cleanup) => format!("{error}; temporary cleanup failed: {cleanup}"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn req_sav_01_write_fault_preserves_previous_file_and_state() {
        let context = SaveContext::national(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            "m1",
        )
        .unwrap();
        let sim = context.simulation(7).unwrap();
        let hash = sim.state_hash().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("old.ohsave");
        fs::write(&target, b"old").unwrap();
        assert!(
            write_transaction(&target, &context, |file| {
                file.write_all(b"partial").unwrap();
                Err("injected write failure".into())
            })
            .is_err()
        );
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert_eq!(sim.state_hash().unwrap(), hash);
    }
    #[test]
    fn req_sav_01_successful_replace_and_failed_commit_preserve_complete_files() {
        let context = SaveContext::national(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            "m1",
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("saved.ohsave");
        let sim = context.simulation(7).unwrap();
        let bytes = crate::encode(&sim, &context, 0, vec![]).unwrap();
        write_atomic(&target, &bytes, &context).unwrap();
        write_atomic(&target, &bytes, &context).unwrap();
        assert_eq!(fs::read(&target).unwrap(), bytes);
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let locked = fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(&target)
                .unwrap();
            assert!(write_atomic(&target, b"different", &context).is_err());
            drop(locked);
            assert_eq!(fs::read(&target).unwrap(), bytes);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let old = fs::metadata(dir.path()).unwrap().permissions();
            fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o500)).unwrap();
            let result = write_atomic(&target, b"different", &context);
            fs::set_permissions(dir.path(), old).unwrap();
            assert!(result.is_err());
            assert_eq!(fs::read(&target).unwrap(), bytes);
        }
        assert!(crate::decode(&fs::read(&target).unwrap(), &context, false).is_ok());
    }
}
