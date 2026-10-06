use oh_core::{Fx,NationId,StateId};
use oh_data::national::{load_scenario,LoadedNational,ModifierInput};
use oh_sim::{Simulation,TimeConfig,Date,Command};
use std::path::Path;
fn make(d:&LoadedNational,seed:u64)->Simulation{Simulation::with_world("m1".into(),Date::new(2000,1,1).unwrap(),seed,TimeConfig::from_defines(&d.pack.defines).unwrap(),oh_sim::world::World::from_loaded(d).unwrap()).unwrap()}
fn main(){
 let args:Vec<_>=std::env::args().collect();let root=Path::new(&args[1]);
 if args.len()>2 {match load_scenario(root,"m1"){Ok(d)=>{match oh_sim::world::World::from_loaded(&d){Ok(_)=>println!("ACCEPT"),Err(e)=>{eprintln!("WORLD ERROR: {e}");std::process::exit(2)}}},Err(e)=>{eprintln!("REJECT: {e}");std::process::exit(1)}};return;}
 let d=load_scenario(root,"m1").unwrap();let s=make(&d,1);let h=s.state_hash().unwrap();
 std::fs::write("target/wp09-verify/generated-protocol.ts",oh_proto::typescript()).unwrap();
 assert_eq!(std::fs::read("client/src/proto/protocol.ts").unwrap(),oh_proto::typescript().as_bytes());
 println!("generated TS bytes match without writing tracked file");
 assert_ne!(make(&d,u64::MAX).state_hash().unwrap(),h);
 let bytes=oh_core::canonical_bytes(&s).unwrap();let oracle=bytes.iter().fold(14695981039346656037u64,|a,b|(a^(*b as u64)).wrapping_mul(1099511628211));assert_eq!(oracle,h);
 let mut reordered=d.clone();reordered.nations.reverse();reordered.map.states.reverse();reordered.map.provinces.reverse();assert_eq!(make(&reordered,1).state_hash().unwrap(),h);
 let p=oh_proto::WorldView::from_sim(&s).unwrap();assert_eq!(p.province_ids,vec![10,20,30,40,50,60]);assert_eq!(p.provinces[1].id,20);assert_eq!(p.provinces[1].owner,Some(1));assert_eq!(p.provinces[1].controller,Some(2));for sea in [50,60]{let v=p.provinces.iter().find(|p|p.id==sea).unwrap();assert_eq!((v.owner,v.controller,v.state),(None,None,None));}
 let mut modified=d.clone();modified.scenario.state_modifiers.insert(1,vec![ModifierInput{source:"z-add".into(),target_stat:"infrastructure".into(),operation:"add".into(),value:"0.00000000023283064365386962890625".into(),expires:Some(2)},ModifierInput{source:"a-mul".into(),target_stat:"infrastructure".into(),operation:"mul".into(),value:"2".into(),expires:None}]);
 let mut sim=make(&modified,1);let state=sim.world().unwrap().state(StateId(1)).unwrap();assert_eq!(state.infrastructure().to_bits(),Fx::ONE.to_bits()*2+2);state.ledger().verify_applied_value(state.infrastructure()).unwrap();
 let view=oh_proto::WorldView::from_sim(&sim).unwrap();let l=&view.states[0].infrastructure;assert_eq!(l.entries.iter().map(|e|e.operation_key.as_str()).collect::<Vec<_>>(),vec!["ledger-base","ledger-add","ledger-multiply"]);assert_eq!(l.entries[1].value.parse::<Fx>().unwrap().to_bits(),1);assert_eq!(l.final_value.parse::<Fx>().unwrap().to_bits(),state.infrastructure().to_bits());
 let msg=oh_proto::ServerMessage::WorldResult{request:"probe".into(),supported:true,reason_key:None,world:Some(view)};let wire=oh_proto::encode(&msg).unwrap();assert_eq!(oh_proto::decode_server(&wire).unwrap(),msg);std::fs::write("target/wp09-verify/precise-world.msgpack",wire).unwrap();
 println!("sparse IDs/null sea/lake and bit-precision Add/Mul wire PASS");
 let baseline=sim.state_hash().unwrap();let mut x=modified.clone();x.scenario.state_modifiers.get_mut(&1).unwrap()[0].expires=Some(3);assert_ne!(make(&x,1).state_hash().unwrap(),baseline);
 for field in ["source","target","value","op"]{let mut x=modified.clone();let m=&mut x.scenario.state_modifiers.get_mut(&1).unwrap()[0];match field{"source"=>m.source="changed".into(),"target"=>m.target_stat="unused-stat".into(),"value"=>m.value="0.25".into(),"op"=>m.operation="mul".into(),_=>unreachable!()};assert_ne!(make(&x,1).state_hash().unwrap(),baseline);}
 let mut x=modified.clone();x.scenario.state_modifiers.get_mut(&1).unwrap().reverse();assert_eq!(make(&x,1).state_hash().unwrap(),baseline);
 sim.step().unwrap();sim.step().unwrap();assert_eq!(sim.world().unwrap().state(StateId(1)).unwrap().infrastructure(),Fx::from_num(2));assert_eq!(sim.world().unwrap().state(StateId(1)).unwrap().ledger().tick(),2);
 for (tick,nation,seq,cmd) in [(5,1,1,Command::Pause(true)),(6,1,1,Command::Pause(true)),(5,2,1,Command::Pause(true)),(5,1,2,Command::Pause(true)),(5,1,1,Command::SetSpeed(5))]{let mut a=make(&d,1);a.enqueue(tick,NationId(nation),seq,cmd).unwrap();assert_ne!(a.state_hash().unwrap(),h);println!("queue input hash {:016x}",a.state_hash().unwrap());}
 let mut a=make(&d,1);a.enqueue(2,NationId(1),1,Command::Pause(true)).unwrap();let before=a.state_hash().unwrap();assert!(a.enqueue(2,NationId(1),1,Command::SetSpeed(3)).is_err());assert_eq!(before,a.state_hash().unwrap());a.step().unwrap();assert!(a.enqueue(0,NationId(1),2,Command::Pause(false)).is_err());let before=a.snapshot();a.enqueue(1,NationId(1),3,Command::SetSpeed(0)).unwrap();let step=a.step().unwrap();assert!(step.commands[0].result.is_err());assert_eq!(before.speed(),a.snapshot().speed());

 let mut config=d.clone();config.pack.defines.0.get_mut("time").unwrap().insert("initial_speed".into(),oh_data::DefineValue::Number(oh_data::Number::Integer(2)));assert_ne!(make(&config,1).state_hash().unwrap(),h);
 let mut d2=d.clone();d2.map.states.iter_mut().find(|s|s.id==2).unwrap().infrastructure=1_500_000_000;
 d2.scenario.state_modifiers.insert(1,vec![ModifierInput{source:"north-add".into(),target_stat:"infrastructure".into(),operation:"add".into(),value:"1".into(),expires:Some(2)}]);
 d2.scenario.state_modifiers.insert(2,vec![ModifierInput{source:"south-add".into(),target_stat:"infrastructure".into(),operation:"add".into(),value:"-1000000000".into(),expires:Some(2)},ModifierInput{source:"south-mul".into(),target_stat:"infrastructure".into(),operation:"mul".into(),value:"2".into(),expires:None}]);
 let mut atom=make(&d2,1);atom.step().unwrap();atom.enqueue(1,NationId(1),5,Command::SetSpeed(5)).unwrap();let before=oh_core::canonical_bytes(&atom).unwrap();assert!(atom.step().is_err());assert_eq!(oh_core::canonical_bytes(&atom).unwrap(),before);assert_eq!(atom.world().unwrap().state(StateId(1)).unwrap().infrastructure(),Fx::from_num(2));assert_eq!(atom.snapshot().speed(),1);assert_eq!(atom.snapshot().tick(),1);assert_eq!(atom.pending_commands().len(),1);
 println!("multi-state partial ledger failure preserves world/time/config/queue canonical bytes PASS");
 println!("hash oracle/ordered inputs/modifier expiry/queue error state PASS");
}

