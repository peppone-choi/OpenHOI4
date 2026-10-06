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
    write_transaction(path, bytes, context, &mut FilesystemIo)
}
// The transaction owns ordering, cleanup, and the commit boundary. Operations
// are replaceable only privately, so tests can force the production branches.
trait TransactionIo {
    fn write(&mut self, file: &mut fs::File, bytes: &[u8]) -> std::io::Result<()>;
    fn flush(&mut self, file: &mut fs::File) -> std::io::Result<()>;
    fn sync_file(&mut self, file: &fs::File) -> std::io::Result<()>;
    fn persist(
        &mut self,
        temp: tempfile::NamedTempFile,
        target: &Path,
    ) -> std::result::Result<fs::File, tempfile::PersistError>;
    fn sync_directory(&mut self, parent: &Path) -> std::io::Result<Option<String>>;
}
struct FilesystemIo;
impl TransactionIo for FilesystemIo {
    fn write(&mut self, file: &mut fs::File, bytes: &[u8]) -> std::io::Result<()> {
        file.write_all(bytes)
    }
    fn flush(&mut self, file: &mut fs::File) -> std::io::Result<()> {
        file.flush()
    }
    fn sync_file(&mut self, file: &fs::File) -> std::io::Result<()> {
        file.sync_all()
    }
    fn persist(
        &mut self,
        temp: tempfile::NamedTempFile,
        target: &Path,
    ) -> std::result::Result<fs::File, tempfile::PersistError> {
        temp.persist(target)
    }
    fn sync_directory(&mut self, parent: &Path) -> std::io::Result<Option<String>> {
        #[cfg(unix)]
        {
            fs::File::open(parent)
                .and_then(|f| f.sync_all())
                .map(|()| None)
        }
        #[cfg(not(unix))]
        {
            let _ = parent;
            Ok(Some(
                "committed; directory durability not confirmed on this platform".into(),
            ))
        }
    }
}
fn write_transaction(
    path: &Path,
    bytes: &[u8],
    context: &SaveContext,
    io: &mut impl TransactionIo,
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
        io.write(temp.as_file_mut(), bytes)
            .map_err(|e| e.to_string())?;
        io.flush(temp.as_file_mut())
            .map_err(|e| format!("SaveIO: flush {e}"))?;
        io.sync_file(temp.as_file())
            .map_err(|e| format!("SaveIO: sync {e}"))?;
        Ok(())
    })();
    if let Err(error) = prepared {
        return Err(cleanup(temp, error));
    }
    if let Err(error) = io.persist(temp, &target) {
        return Err(cleanup(
            error.file,
            format!("SaveIO: commit {}", error.error),
        ));
    }
    let warning = match io.sync_directory(&parent) {
        Ok(warning) => warning,
        Err(error) => Some(format!("committed; directory sync: {error}")),
    };
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
    // Independent test-only SHA-256, not the simulation's FNV hash. No new
    // dependency or production hash/settings are introduced.
    fn sha256(input: &[u8]) -> String {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];
        let mut h = [
            0x6a09e667_u32,
            0xbb67ae85,
            0x3c6ef372,
            0xa54ff53a,
            0x510e527f,
            0x9b05688c,
            0x1f83d9ab,
            0x5be0cd19,
        ];
        let mut padded = input.to_vec();
        padded.push(0x80);
        while padded.len() % 64 != 56 {
            padded.push(0);
        }
        padded.extend_from_slice(&(u64::try_from(input.len()).unwrap() * 8).to_be_bytes());
        for block in padded.as_chunks::<64>().0 {
            let mut w = [0_u32; 64];
            for (dst, word) in w.iter_mut().zip(block.as_chunks::<4>().0) {
                *dst = u32::from_be_bytes(*word);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ (!e & g);
                let t1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let t2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(t1);
                d = c;
                c = b;
                b = a;
                a = t1.wrapping_add(t2);
            }
            for (dst, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
                *dst = dst.wrapping_add(value);
            }
        }
        h.iter().map(|word| format!("{word:08x}")).collect()
    }
    #[test]
    fn test_digest_matches_standard_sha256_vectors_and_committed_fixture() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256(include_bytes!("../tests/fixtures/m1-v1.ohsave")),
            "57b13c62ef1a6f88ade1851d0b3ae398157f95ca5bc246068eac3b2a964322fa"
        );
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Stage {
        Write,
        Flush,
        Sync,
        Persist,
        DirectorySync,
    }
    struct FaultIo {
        stage: Stage,
        calls: Vec<Stage>,
    }
    impl FaultIo {
        fn new(stage: Stage) -> Self {
            Self {
                stage,
                calls: vec![],
            }
        }
        fn check(&mut self, stage: Stage) -> std::io::Result<()> {
            self.calls.push(stage);
            if self.stage == stage {
                Err(std::io::Error::other(format!("injected {stage:?} failure")))
            } else {
                Ok(())
            }
        }
    }
    impl TransactionIo for FaultIo {
        fn write(&mut self, file: &mut fs::File, bytes: &[u8]) -> std::io::Result<()> {
            if self.stage == Stage::Write {
                // Leave a real partial temporary file for the shared cleanup path.
                file.write_all(&bytes[..bytes.len() / 2])?;
            }
            self.check(Stage::Write)?;
            FilesystemIo.write(file, bytes)
        }
        fn flush(&mut self, file: &mut fs::File) -> std::io::Result<()> {
            self.check(Stage::Flush)?;
            FilesystemIo.flush(file)
        }
        fn sync_file(&mut self, file: &fs::File) -> std::io::Result<()> {
            self.check(Stage::Sync)?;
            FilesystemIo.sync_file(file)
        }
        fn persist(
            &mut self,
            temp: tempfile::NamedTempFile,
            target: &Path,
        ) -> std::result::Result<fs::File, tempfile::PersistError> {
            if let Err(error) = self.check(Stage::Persist) {
                return Err(tempfile::PersistError { error, file: temp });
            }
            FilesystemIo.persist(temp, target)
        }
        fn sync_directory(&mut self, parent: &Path) -> std::io::Result<Option<String>> {
            self.check(Stage::DirectorySync)?;
            FilesystemIo.sync_directory(parent)
        }
    }
    fn entries(parent: &Path) -> Vec<std::ffi::OsString> {
        let mut entries: Vec<_> = fs::read_dir(parent)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        entries.sort();
        entries
    }
    fn assert_precommit_failure(stage: Stage, expected_calls: &[Stage], prefix: &str) {
        let context = SaveContext::national(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            "m1",
        )
        .unwrap();
        let mut sim = context.simulation(7).unwrap();
        sim.enqueue(5, oh_core::NationId(0), 1, oh_sim::Command::SetSpeed(5))
            .unwrap();
        let dto = sim.export_save().unwrap();
        let snapshot = sim.snapshot();
        let queue = sim.pending_commands().clone();
        assert!(!queue.is_empty());
        let hash = sim.state_hash().unwrap();
        let canonical = oh_core::canonical_bytes(&sim).unwrap();
        let old = crate::encode(&sim, &context, 0, vec![]).unwrap();
        let new = crate::encode(&sim, &context, 1, vec![]).unwrap();
        assert_ne!(old, new);
        // Exercise replacement and initial-file failure, both with real temp files.
        for existing in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("saved.ohsave");
            if existing {
                fs::write(&target, &old).unwrap();
            }
            let before = entries(dir.path());
            let mut io = FaultIo::new(stage);
            let result = write_transaction(&target, &new, &context, &mut io);
            assert!(
                result.is_err(),
                "{stage:?} must reach precommit error branch"
            );
            let error = result.err().unwrap();
            assert!(error.starts_with(prefix), "{error}");
            assert!(
                error.contains(&format!("injected {stage:?} failure")),
                "{error}"
            );
            assert_eq!(io.calls, expected_calls);
            assert_eq!(
                entries(dir.path()),
                before,
                "temporary file must be removed"
            );
            if existing {
                let after = fs::read(&target).unwrap();
                assert_eq!(after, old, "complete old bytes must survive");
                assert_eq!(sha256(&after), sha256(&old));
                assert_eq!(
                    crate::decode(&after, &context, false)
                        .unwrap()
                        .simulation
                        .export_save()
                        .unwrap(),
                    dto
                );
                println!(
                    "fault={stage:?} old_sha256={} new_candidate_sha256={} old_bytes={} cleanup=unchanged",
                    sha256(&after),
                    sha256(&new),
                    after.len()
                );
            } else {
                assert!(!target.exists());
            }
            assert_eq!(sim.export_save().unwrap(), dto);
            assert_eq!(sim.snapshot(), snapshot);
            assert_eq!(sim.pending_commands(), &queue);
            assert_eq!(sim.state_hash().unwrap(), hash);
            assert_eq!(oh_core::canonical_bytes(&sim).unwrap(), canonical);
        }
    }
    #[test]
    fn req_sav_01_partial_write_fault_cleans_temp_and_preserves_export() {
        assert_precommit_failure(Stage::Write, &[Stage::Write], "injected Write");
    }
    #[test]
    fn req_sav_01_flush_fault_cleans_temp_and_preserves_export() {
        assert_precommit_failure(Stage::Flush, &[Stage::Write, Stage::Flush], "SaveIO: flush");
    }
    #[test]
    fn req_sav_01_sync_fault_cleans_temp_and_preserves_export() {
        assert_precommit_failure(
            Stage::Sync,
            &[Stage::Write, Stage::Flush, Stage::Sync],
            "SaveIO: sync",
        );
    }
    #[test]
    fn req_sav_01_persist_fault_cleans_temp_and_preserves_export() {
        assert_precommit_failure(
            Stage::Persist,
            &[Stage::Write, Stage::Flush, Stage::Sync, Stage::Persist],
            "SaveIO: commit",
        );
    }
    #[test]
    fn req_sav_01_directory_sync_warning_keeps_committed_new_file() {
        let context = SaveContext::national(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/packs/testland"),
            "m1",
        )
        .unwrap();
        let sim = context.simulation(7).unwrap();
        let old = crate::encode(&sim, &context, 0, vec![]).unwrap();
        let new = crate::encode(&sim, &context, 1, vec![]).unwrap();
        assert_ne!(old, new);
        for existing in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("saved.ohsave");
            if existing {
                fs::write(&target, &old).unwrap();
            }
            let mut io = FaultIo::new(Stage::DirectorySync);
            let outcome = write_transaction(&target, &new, &context, &mut io).unwrap();
            assert!(
                outcome
                    .durability_warning
                    .unwrap()
                    .contains("committed; directory sync: injected DirectorySync failure")
            );
            assert_eq!(
                io.calls,
                [
                    Stage::Write,
                    Stage::Flush,
                    Stage::Sync,
                    Stage::Persist,
                    Stage::DirectorySync
                ]
            );
            assert_eq!(
                entries(dir.path()),
                [std::ffi::OsString::from("saved.ohsave")]
            );
            let committed = fs::read(&target).unwrap();
            assert_eq!(committed, new);
            assert_ne!(sha256(&committed), sha256(&old));
            assert_eq!(
                crate::decode(&committed, &context, false)
                    .unwrap()
                    .simulation
                    .export_save()
                    .unwrap(),
                sim.export_save().unwrap()
            );
        }
    }
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
            write_transaction(
                &target,
                b"partial",
                &context,
                &mut FaultIo::new(Stage::Write)
            )
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
        assert_eq!(fs::read(&target).unwrap(), bytes);
        assert_eq!(
            crate::decode(&bytes, &context, false)
                .unwrap()
                .simulation
                .export_save()
                .unwrap(),
            sim.export_save().unwrap()
        );
        let replacement = crate::encode(&sim, &context, 1, vec![]).unwrap();
        assert_ne!(bytes, replacement);
        let bytes = replacement;
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
