use oh_core::{NationId,canonical_bytes};
use oh_sim::{Simulation,Command,save_state::SimulationSaveV1};
use oh_save::{SaveContext,encode,decode,write_atomic,Limits};
use std::{path::Path,fs};
fn main(){
 let args:Vec<String>=std::env::args().collect();
 let c=SaveContext::national(Path::new(&args[2]),"m1").unwrap();
 let out=Path::new(&args[3]);fs::create_dir_all(out).unwrap();
 if args[1]=="capture" {
  let mut s=c.simulation(123).unwrap();
  for (t,n,q,v) in [(23,65535,1,Command::Pause(false)),(23,0,9,Command::Pause(true)),(23,0,2,Command::SetSpeed(2)),(23,65535,3,Command::SetSpeed(4)),(49,42,0,Command::SetSpeed(1))] {s.enqueue(t,NationId(n),q,v).unwrap();}
  dump(&s,out,"initial");
  let mut continuous=s.clone();for _ in 0..54 {assert!(continuous.step().unwrap().advanced);}dump(&continuous,out,"continuous");
  for _ in 0..23 {assert!(s.step().unwrap().advanced);}dump(&s,out,"split");
  write_atomic(&out.join("saved.ohsave"),&encode(&s,&c,0,vec![1,2]).unwrap(),&c).unwrap();
  println!("capture PID {}",std::process::id());
 }else if args[1]=="resume" {
  let mut s=decode(&fs::read(out.join("saved.ohsave")).unwrap(),&c,false).unwrap().simulation;
  dump(&s,out,"loaded");
  for i in 0..31 {let step=s.step().unwrap();assert!(step.advanced);if i==0 {assert_eq!(step.commands.iter().map(|r|(r.nation.0,r.sequence)).collect::<Vec<_>>(),[(0,2),(0,9),(65535,1),(65535,3)]);dump(&s,out,"expiry");}}
  dump(&s,out,"resumed");println!("resume PID {}",std::process::id());
 }else if args[1]=="cases"{
  let live=c.simulation(123).unwrap();let before=canonical_bytes(&live).unwrap();
  let cases:serde_json::Value=serde_json::from_slice(&fs::read(out.join("cases.json")).unwrap()).unwrap();
  for case in cases.as_array().unwrap(){
   let dto:SimulationSaveV1=serde_json::from_value(case["dto"].clone()).unwrap();
   let result=Simulation::from_save(dto.clone(),c.restore_context());
   let ok=case["ok"].as_bool().unwrap();assert_eq!(result.is_ok(),ok,"{} {:?}",case["name"],result.err());
   if ok {let s=result.unwrap();assert_eq!(s.export_save().unwrap(),dto);let bytes=encode(&s,&c,0,vec![]).unwrap();assert_eq!(decode(&bytes,&c,false).unwrap().simulation.export_save().unwrap(),dto);}
   assert_eq!(canonical_bytes(&live).unwrap(),before);println!("{} {}",case["name"],if ok {"accepted"}else{"atomic rejection"});
  }
  let mut errors=0;for speed in [0,6]{let mut s=live.clone();s.enqueue(0,NationId(65535),0,Command::SetSpeed(speed)).unwrap();let before=canonical_bytes(&s).unwrap();assert!(encode(&s,&c,0,vec![]).unwrap_err().contains("InvalidQueue"));assert_eq!(canonical_bytes(&s).unwrap(),before);let step=s.step().unwrap();assert!(step.commands[0].result.is_err());assert!(s.pending_commands().is_empty());errors+=1;}println!("invalid live export/executor cases {errors}");
  let bytes=encode(&live,&c,0,vec![]).unwrap();let h=oh_save::inspect_header(&bytes,&Limits::default()).unwrap();fs::write(out.join("header.json"),serde_json::to_vec_pretty(&h).unwrap()).unwrap();
  let end=10+u32::from_le_bytes(bytes[6..10].try_into().unwrap()) as usize;fs::write(out.join("body.bin"),zstd::stream::decode_all(&bytes[end..]).unwrap()).unwrap();
 }else if args[1]=="blobs" {
  let live=c.simulation(123).unwrap();let before=canonical_bytes(&live).unwrap();
  let cases:serde_json::Value=serde_json::from_slice(&fs::read(out.join("blob-cases.json")).unwrap()).unwrap();
  for (j,case) in cases.as_array().unwrap().iter().enumerate(){
   let bytes=if !case["full"].is_null(){unhex(case["full"].as_str().unwrap())}else{
    let header=unhex(case["header"].as_str().unwrap());let body=unhex(case["body"].as_str().unwrap());
    let mut bytes=b"OHSV".to_vec();bytes.extend(1u16.to_le_bytes());bytes.extend((header.len() as u32).to_le_bytes());bytes.extend(header);bytes.extend(zstd::stream::encode_all(body.as_slice(),3).unwrap());bytes
   };
   fs::write(out.join(format!("probe-{j}.ohsave")),&bytes).unwrap();
   let mut limits=Limits::default();if let Some(n)=case["body_cap"].as_u64(){limits.body_max_bytes=n;}if let Some(n)=case["file_cap"].as_u64(){limits.file_max_bytes=n;}
   for force in [false,true] {
    let r=oh_save::decode_with_limits(&bytes,&c,force,&limits);let ok=case["ok"].as_bool().unwrap();
    assert_eq!(r.is_ok(),ok,"{} force{} {:?}",case["name"],force,r.err());
    if ok {let restored=r.unwrap();assert_eq!(restored.simulation.export_save().unwrap(),serde_json::from_slice::<SimulationSaveV1>(&fs::read(out.join("split.json")).unwrap()).unwrap());}
   }
   assert_eq!(canonical_bytes(&live).unwrap(),before);println!("{} accepted={} atomic=true",case["name"],case["ok"]);
  }
 }else {panic!("unknown mode");}
}
fn dump(s:&Simulation,p:&Path,n:&str){fs::write(p.join(format!("{n}.json")),serde_json::to_vec_pretty(&s.export_save().unwrap()).unwrap()).unwrap();fs::write(p.join(format!("{n}.canonical")),canonical_bytes(s).unwrap()).unwrap();println!("{n} {:016x}",s.state_hash().unwrap());}

fn unhex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|b|u8::from_str_radix(std::str::from_utf8(b).unwrap(),16).unwrap()).collect()}
