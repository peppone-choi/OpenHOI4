//! A deliberately narrow ZIP32 stored profile. Never extracts to the filesystem.
use crate::{Result, context::reparse};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::Path,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub archive_max_bytes: u64,
    pub member_max_bytes: u64,
    pub total_max_bytes: u64,
    pub members_max: usize,
    pub events_max: usize,
    pub phases_per_event_max: usize,
    pub input_line_max_bytes: u64,
}
impl Default for Policy {
    fn default() -> Self {
        toml::from_str(include_str!("../repro-defines.toml")).expect("repro host policy")
    }
}
const NAMES: [&str; 5] = [
    "bundle.toml",
    "commands.log",
    "commands.txt",
    "final.json",
    "start.ohsave",
];
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0_u32;
    for b in bytes {
        crc ^= u32::from(*b);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}
fn u16_at(b: &[u8], i: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(i..i + 2)
            .ok_or("ZIP: truncated u16")?
            .try_into()
            .map_err(|_| "ZIP: u16")?,
    ))
}
fn u32_at(b: &[u8], i: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(i..i + 4)
            .ok_or("ZIP: truncated u32")?
            .try_into()
            .map_err(|_| "ZIP: u32")?,
    ))
}
fn put16(b: &mut Vec<u8>, v: u16) {
    b.extend(v.to_le_bytes());
}
fn put32(b: &mut Vec<u8>, v: u32) {
    b.extend(v.to_le_bytes());
}
fn name(b: &[u8]) -> Result<&str> {
    let n = std::str::from_utf8(b).map_err(|_| "ZIP: non UTF8 name")?;
    if !NAMES.contains(&n) {
        return Err("ZIP: unknown/unsafe member name".into());
    }
    Ok(n)
}
pub fn decode(bytes: &[u8], p: &Policy) -> Result<BTreeMap<String, Vec<u8>>> {
    if bytes.len() as u64 > p.archive_max_bytes || bytes.len() < 22 {
        return Err("ZIP: archive limit/truncated".into());
    }
    let e = bytes.len() - 22;
    if u32_at(bytes, e)? != 0x06054b50
        || u16_at(bytes, e + 4)? != 0
        || u16_at(bytes, e + 6)? != 0
        || u16_at(bytes, e + 20)? != 0
    {
        return Err("ZIP: EOCD/multidisk/comment".into());
    }
    let count = usize::from(u16_at(bytes, e + 10)?);
    if count == 0 || count > p.members_max || usize::from(u16_at(bytes, e + 8)?) != count {
        return Err("ZIP: member count".into());
    }
    let cstart = usize::try_from(u32_at(bytes, e + 16)?).map_err(|_| "ZIP: offset")?;
    let clen = usize::try_from(u32_at(bytes, e + 12)?).map_err(|_| "ZIP: size")?;
    if cstart.checked_add(clen) != Some(e) {
        return Err("ZIP: central size".into());
    }
    let mut central = cstart;
    let mut local = 0_usize;
    let mut total = 0_u64;
    let mut result = BTreeMap::new();
    for _ in 0..count {
        if central.checked_add(46).is_none_or(|v| v > e) || u32_at(bytes, central)? != 0x02014b50 {
            return Err("ZIP: central header".into());
        }
        let size = u32_at(bytes, central + 24)?;
        let compressed = u32_at(bytes, central + 20)?;
        total = total
            .checked_add(u64::from(size))
            .ok_or("ZIP: total overflow")?;
        if u64::from(size) > p.member_max_bytes
            || u64::from(compressed) > p.member_max_bytes
            || size != compressed
            || total > p.total_max_bytes
        {
            return Err("ZIP: size limits/declared lengths".into());
        }
        let attrs = u32_at(bytes, central + 38)?;
        let mode = (attrs >> 16) & 0xf000;
        if (mode != 0 && mode != 0x8000) || attrs & 0xffff != 0 {
            return Err("ZIP: nonregular/symlink/reparse member".into());
        }
        if u16_at(bytes, central + 6)? != 20
            || u16_at(bytes, central + 8)? != 0
            || u16_at(bytes, central + 10)? != 0
            || u16_at(bytes, central + 30)? != 0
            || u16_at(bytes, central + 32)? != 0
            || u16_at(bytes, central + 34)? != 0
            || u16_at(bytes, central + 36)? != 0
            || usize::try_from(u32_at(bytes, central + 42)?).ok() != Some(local)
        {
            return Err("ZIP: unsupported method/flags/offset/extra".into());
        }
        let len = usize::from(u16_at(bytes, central + 28)?);
        let cend = central
            .checked_add(46 + len)
            .ok_or("ZIP: central overflow")?;
        if cend > e {
            return Err("ZIP: name length".into());
        }
        let cname = &bytes[central + 46..cend];
        let n = name(cname)?;
        if result.contains_key(n) {
            return Err("ZIP: duplicate member".into());
        }
        if local.checked_add(30).is_none_or(|v| v > cstart)
            || u32_at(bytes, local)? != 0x04034b50
            || u16_at(bytes, local + 4)? != 20
            || u16_at(bytes, local + 6)? != 0
            || u16_at(bytes, local + 8)? != 0
            || u16_at(bytes, local + 28)? != 0
            || usize::from(u16_at(bytes, local + 26)?) != len
            || u32_at(bytes, local + 14)? != u32_at(bytes, central + 16)?
            || u32_at(bytes, local + 18)? != compressed
            || u32_at(bytes, local + 22)? != size
            || u32_at(bytes, local + 10)? != u32_at(bytes, central + 12)?
        {
            return Err("ZIP: local/central disagreement".into());
        }
        let data = local.checked_add(30 + len).ok_or("ZIP: local overflow")?;
        let end = data
            .checked_add(size as usize)
            .ok_or("ZIP: data overflow")?;
        if end > cstart || bytes.get(local + 30..data) != Some(cname) {
            return Err("ZIP: truncated/local name".into());
        }
        if crc32(&bytes[data..end]) != u32_at(bytes, central + 16)? {
            return Err("ZIP: CRC mismatch".into());
        }
        result.insert(n.to_owned(), bytes[data..end].to_vec());
        local = end;
        central = cend;
    }
    if local != cstart || central != e {
        return Err("ZIP: gaps/trailing content".into());
    }
    Ok(result)
}
pub fn encode(members: &BTreeMap<String, Vec<u8>>, p: &Policy) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut central = Vec::new();
    let count = u16::try_from(members.len()).map_err(|_| "ZIP: count")?;
    let mut total = 0_u64;
    if members.len() > p.members_max || members.is_empty() {
        return Err("ZIP: count limit".into());
    }
    for (n, b) in members {
        name(n.as_bytes())?;
        total = total.checked_add(b.len() as u64).ok_or("ZIP: total")?;
        if b.len() as u64 > p.member_max_bytes || total > p.total_max_bytes {
            return Err("ZIP: member/total limit".into());
        }
        let size = u32::try_from(b.len()).map_err(|_| "ZIP: ZIP32 size")?;
        let offset = u32::try_from(bytes.len()).map_err(|_| "ZIP: ZIP32 offset")?;
        let len = u16::try_from(n.len()).map_err(|_| "ZIP: name")?;
        let crc = crc32(b);
        put32(&mut bytes, 0x04034b50);
        put16(&mut bytes, 20);
        put16(&mut bytes, 0);
        put16(&mut bytes, 0);
        put32(&mut bytes, 0);
        put32(&mut bytes, crc);
        put32(&mut bytes, size);
        put32(&mut bytes, size);
        put16(&mut bytes, len);
        put16(&mut bytes, 0);
        bytes.extend(n.as_bytes());
        bytes.extend(b);
        put32(&mut central, 0x02014b50);
        put16(&mut central, 20);
        put16(&mut central, 20);
        put16(&mut central, 0);
        put16(&mut central, 0);
        put32(&mut central, 0);
        put32(&mut central, crc);
        put32(&mut central, size);
        put32(&mut central, size);
        put16(&mut central, len);
        put16(&mut central, 0);
        put16(&mut central, 0);
        put16(&mut central, 0);
        put16(&mut central, 0);
        put32(&mut central, 0);
        put32(&mut central, offset);
        central.extend(n.as_bytes());
    }
    let start = u32::try_from(bytes.len()).map_err(|_| "ZIP: ZIP32 offset")?;
    let size = u32::try_from(central.len()).map_err(|_| "ZIP: ZIP32 size")?;
    bytes.extend(central);
    put32(&mut bytes, 0x06054b50);
    put16(&mut bytes, 0);
    put16(&mut bytes, 0);
    put16(&mut bytes, count);
    put16(&mut bytes, count);
    put32(&mut bytes, size);
    put32(&mut bytes, start);
    put16(&mut bytes, 0);
    decode(&bytes, p)?;
    Ok(bytes)
}
fn safe_path(path: &Path) -> Result<()> {
    for parent in path.ancestors() {
        if parent.as_os_str().is_empty() {
            continue;
        }
        match fs::symlink_metadata(parent) {
            Ok(m) if m.file_type().is_symlink() || reparse(&m) => {
                return Err("ReproPath: symlink/reparse".into());
            }
            Ok(m) if parent == path && !m.is_file() => {
                return Err("ReproPath: nonregular file".into());
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound && parent == path => (),
            Err(e) => return Err(format!("ReproPath: {e}")),
        }
    }
    Ok(())
}
pub fn read(path: &Path, cap: u64) -> Result<Vec<u8>> {
    safe_path(path)?;
    let mut b = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(cap.checked_add(1).ok_or("Repro: cap overflow")?)
        .read_to_end(&mut b)
        .map_err(|e| e.to_string())?;
    if b.len() as u64 > cap {
        return Err("Repro: file limit".into());
    }
    Ok(b)
}
pub fn write(path: &Path, bytes: &[u8], context: &crate::SaveContext) -> Result<()> {
    transaction(path, bytes, context, |file, bytes| file.write_all(bytes))
}
fn transaction(
    path: &Path,
    bytes: &[u8],
    context: &crate::SaveContext,
    write: impl FnOnce(&mut fs::File, &[u8]) -> std::io::Result<()>,
) -> Result<()> {
    safe_path(path)?;
    decode(bytes, &Policy::default())?;
    context.verify_unchanged()?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    write(temp.as_file_mut(), bytes).map_err(|e| e.to_string())?;
    temp.flush().map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    safe_path(path)?;
    context.verify_unchanged()?;
    temp.persist(path).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| format!("Repro committed; directory sync {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn retained_fixture() -> std::path::PathBuf {
        let base = std::env::var_os("OH_WP25_EVIDENCE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/evidence/WP-25-io")
            });
        fs::create_dir_all(&base).unwrap();
        let directory = tempfile::Builder::new()
            .prefix("partial-io-")
            .tempdir_in(&base)
            .unwrap()
            .keep();
        fs::create_dir(directory.join("regular")).unwrap();
        directory
    }
    #[test]
    fn req_sav_05_alias_guard_and_canonical_regular_partial_io_are_separate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
        let context = crate::SaveContext::national(&root, "m1").unwrap();
        let sim = context.simulation(7).unwrap();
        let before = crate::repro::report(&sim).unwrap();
        let bytes = crate::repro::Recorder::new(&sim, &context)
            .unwrap()
            .finish(&sim, &context)
            .unwrap();
        let retained = retained_fixture();
        let regular = retained.join("regular").canonicalize().unwrap();
        let target = regular.join("target.zip");
        fs::write(&target, b"existing target").unwrap();
        let alias = retained.join("alias");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&regular, &alias).unwrap();
        #[cfg(windows)]
        {
            let result = std::process::Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-Command",
                    "New-Item",
                    "-ItemType",
                    "Junction",
                    "-Path",
                ])
                .arg(&alias)
                .arg("-Target")
                // PowerShell's junction target must use its normal path syntax;
                // Rust canonical Windows paths use the extended-length prefix.
                .arg(retained.join("regular"))
                .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
                .output()
                .unwrap();
            fs::write(retained.join("junction.stdout"), &result.stdout).unwrap();
            fs::write(retained.join("junction.stderr"), &result.stderr).unwrap();
            assert!(
                result.status.success(),
                "junction setup: {:?}",
                result.stderr
            );
        }
        let alias_target = alias.join("target.zip");
        let mut alias_write_reached = false;
        let alias_error = transaction(&alias_target, &bytes, &context, |_, _| {
            alias_write_reached = true;
            Err(std::io::Error::other("must not reach alias writer"))
        })
        .unwrap_err();
        assert!(alias_error.contains("symlink/reparse"), "{alias_error}");
        assert!(!alias_write_reached);
        assert_eq!(fs::read(&target).unwrap(), b"existing target");
        let canonical_target = alias_target.canonicalize().unwrap();
        assert_eq!(canonical_target, target);
        let mut canonical_write_reached = false;
        let error = transaction(&canonical_target, &bytes, &context, |file, bytes| {
            canonical_write_reached = true;
            file.write_all(&bytes[..17])?;
            file.flush()?;
            assert_eq!(file.metadata()?.len(), 17);
            Err(std::io::Error::other("test partial write failure"))
        })
        .unwrap_err();
        fs::write(retained.join("alias-guard.json"), serde_json::to_vec_pretty(&serde_json::json!({"alias_target":alias_target,"alias_error":alias_error,"alias_write_reached":alias_write_reached,"canonical_target":canonical_target,"canonical_error":error,"canonical_write_reached":canonical_write_reached})).unwrap()).unwrap();
        assert!(canonical_write_reached);
        assert!(error.contains("partial write failure"), "{error}");
        assert_eq!(fs::read(&target).unwrap(), b"existing target");
        assert_eq!(fs::read_dir(&regular).unwrap().count(), 1);
        assert_eq!(crate::repro::report(&sim).unwrap(), before);
        context.verify_unchanged().unwrap();
    }
    #[test]
    fn req_sav_05_partial_native_temp_write_keeps_target_input_and_authority() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland");
        let context = crate::SaveContext::national(&root, "m1").unwrap();
        let mut sim = context.simulation(7).unwrap();
        sim.enqueue(99, oh_core::NationId(0), 1, oh_sim::Command::Pause(true))
            .unwrap();
        let before = crate::repro::report(&sim).unwrap();
        let input = crate::encode(&sim, &context, 0, vec![]).unwrap();
        let bytes = crate::repro::Recorder::new(&sim, &context)
            .unwrap()
            .finish(&sim, &context)
            .unwrap();
        let retained = retained_fixture();
        let directory = retained.join("regular").canonicalize().unwrap();
        let original = directory.join("input.ohsave");
        fs::write(&original, &input).unwrap();
        let target = directory.join("target.zip");
        fs::write(&target, b"existing target").unwrap();
        let mut injection_reached = false;
        let error = transaction(&target, &bytes, &context, |file, bytes| {
            injection_reached = true;
            // Actual native filesystem prefix write, then an injected I/O failure.
            file.write_all(&bytes[..17])?;
            file.flush()?;
            assert_eq!(file.metadata()?.len(), 17);
            use std::io::{Read, Seek};
            let mut reader = file.try_clone()?;
            reader.seek(std::io::SeekFrom::Start(0))?;
            let mut prefix = Vec::new();
            reader.read_to_end(&mut prefix)?;
            assert_eq!(prefix, bytes[..17]);
            fs::write(retained.join("native-prefix.bin"), prefix)?;
            Err(std::io::Error::other("test partial write failure"))
        })
        .unwrap_err();
        let proof = serde_json::json!({"lexical_fixture":retained,"canonical_fixture":directory,"error":error,"injection_reached":injection_reached,"prefix_bytes":17});
        fs::write(
            retained.join("partial-io.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
        eprintln!("{proof}");
        assert!(
            injection_reached,
            "partial write injection not reached: {error}; target={}",
            target.display()
        );
        assert!(
            error.contains("partial write failure"),
            "{error}; target={}",
            target.display()
        );
        assert_eq!(fs::read(&target).unwrap(), b"existing target");
        assert_eq!(fs::read(&original).unwrap(), input);
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 2);
        assert_eq!(crate::repro::report(&sim).unwrap(), before);
        context.verify_unchanged().unwrap();
    }
}
