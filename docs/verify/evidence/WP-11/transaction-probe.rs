#[cfg(test)]
mod fresh_independent_transaction {
 use super::*;
 struct Inject{at:usize,calls:Vec<usize>}
 impl Inject{fn stage(&mut self,n:usize)->std::io::Result<()>{self.calls.push(n);if self.at==n{Err(std::io::Error::other(format!("fresh-{n}")))}else{Ok(())}}}
 impl TransactionIo for Inject{
  fn write(&mut self,f:&mut fs::File,b:&[u8])->std::io::Result<()>{if self.at==0{f.write_all(&b[..3])?;}self.stage(0)?;FilesystemIo.write(f,b)}
  fn flush(&mut self,f:&mut fs::File)->std::io::Result<()>{self.stage(1)?;FilesystemIo.flush(f)}
  fn sync_file(&mut self,f:&fs::File)->std::io::Result<()>{self.stage(2)?;FilesystemIo.sync_file(f)}
  fn persist(&mut self,t:tempfile::NamedTempFile,p:&Path)->std::result::Result<fs::File,tempfile::PersistError>{if let Err(error)=self.stage(3){return Err(tempfile::PersistError{error,file:t})}FilesystemIo.persist(t,p)}
  fn sync_directory(&mut self,p:&Path)->std::io::Result<Option<String>>{self.stage(4)?;FilesystemIo.sync_directory(p)}
 }
 fn names(p:&Path)->Vec<String>{let mut a:Vec<_>=fs::read_dir(p).unwrap().map(|x|x.unwrap().file_name().to_string_lossy().into_owned()).collect();a.sort();a}
 #[test]
 fn shared_production_branches_old_new_cleanup_and_commit_warning(){
  let c=SaveContext::national(Path::new("E:/openhoi/.orchestrator/wt/WP-11-verify3/target/wp11-independent/own/pack"),"m1").unwrap();
  let mut s=c.simulation(999).unwrap();s.enqueue(10,oh_core::NationId(0),88,oh_sim::Command::SetSpeed(5)).unwrap();
  let canonical=oh_core::canonical_bytes(&s).unwrap();let dto=s.export_save().unwrap();let q=s.pending_commands().clone();let h=s.state_hash().unwrap();
  let old=crate::encode(&s,&c,3,vec![0,65535]).unwrap();let new=crate::encode(&s,&c,4,vec![0,65535]).unwrap();assert_ne!(old,new);
  let root=Path::new("E:/openhoi/.orchestrator/wt/WP-11-verify3/target/wp11-independent/transactions");fs::create_dir_all(root).unwrap();
  for at in 0..6{for exists in [false,true]{
   let dir=root.join(format!("stage-{at}-existing-{exists}"));fs::create_dir_all(&dir).unwrap();let target=dir.join("save.ohsave");if exists{fs::write(&target,&old).unwrap();}
   let initial=names(&dir);let mut io=Inject{at,calls:vec![]};let r=write_transaction(&target,&new,&c,&mut io);
   if at<4{assert!(r.is_err());assert_eq!(names(&dir),initial);assert_eq!(io.calls,(0..=at).collect::<Vec<_>>());if exists{assert_eq!(fs::read(&target).unwrap(),old);assert_eq!(crate::decode(&old,&c,false).unwrap().simulation.export_save().unwrap(),dto);}else{assert!(!target.exists());}}
   else{let outcome=r.unwrap();assert_eq!(io.calls,[0,1,2,3,4]);assert_eq!(names(&dir),["save.ohsave"]);assert_eq!(fs::read(&target).unwrap(),new);assert_eq!(crate::decode(&new,&c,false).unwrap().simulation.export_save().unwrap(),dto);if at==4{assert!(outcome.durability_warning.unwrap().contains("committed; directory sync: fresh-4"));}}
   assert_eq!(oh_core::canonical_bytes(&s).unwrap(),canonical);assert_eq!(s.export_save().unwrap(),dto);assert_eq!(s.pending_commands(),&q);assert_eq!(s.state_hash().unwrap(),h);
   println!("stage={at} existing={exists} calls={:?} complete_old_new_bytes=true cleanup=true",io.calls);
  }}
  fs::write(root.join("old.bin"),old).unwrap();fs::write(root.join("new.bin"),new).unwrap();
 }
}
