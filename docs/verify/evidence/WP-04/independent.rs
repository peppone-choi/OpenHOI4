use oh_core::NationId;
use oh_sim::{Command, Date, Error, Phase, Simulation, TimeConfig};
use oh_data::{Defines,DefineValue,Number};
use std::collections::BTreeMap;
fn sim(y:u32,m:u8,d:u8)->Simulation {Simulation::new("independent".into(),Date::new(y,m,d).unwrap(),913,TimeConfig::new([41,37,23,11,0],4).unwrap()).unwrap()}
fn main(){
 let mut s=sim(2024,4,30);
 assert_eq!(s.ms_per_tick(),11);
 s.enqueue(0,NationId(17),900,Command::Pause(true)).unwrap();
 let paused=s.step().unwrap();
 assert!(!paused.advanced);assert_eq!(paused.phases,vec![Phase::Commands,Phase::Snapshot]);
 let hash=s.state_hash().unwrap();
 for _ in 0..7 {assert!(!s.step().unwrap().advanced);assert_eq!(s.state_hash().unwrap(),hash);}
 // Invalid speed is acknowledged as an error while paused and cannot change speed/time.
 for (i,v) in [0,6,255].into_iter().enumerate(){
 s.enqueue(0,NationId(17),901+i as u64,Command::SetSpeed(v)).unwrap();
 let step=s.step().unwrap();assert_eq!(step.commands[0].result,Err(Error::InvalidSpeed));
 assert_eq!(s.snapshot().speed(),4);assert_eq!(s.snapshot().tick(),0);
 }
 s.enqueue(0,NationId(17),910,Command::Pause(false)).unwrap();
 assert!(s.step().unwrap().advanced);assert_eq!(s.snapshot().tick(),1);assert_eq!(s.snapshot().hour(),1);
 assert_eq!(s.enqueue(0,NationId(0),1,Command::Pause(true)),Err(Error::PastCommand));
 let mut ordered=sim(2024,1,1);let mut reversed=ordered.clone();
 let inputs=[(9,8,1),(2,90,3),(2,7,2),(0,999,4)];
 for (n,q,v) in inputs {ordered.enqueue(0,NationId(n),q,Command::SetSpeed(v)).unwrap();}
 for (n,q,v) in inputs.into_iter().rev(){reversed.enqueue(0,NationId(n),q,Command::SetSpeed(v)).unwrap();}
 let before=ordered.state_hash().unwrap();assert_eq!(before,reversed.state_hash().unwrap());
 assert_eq!(ordered.enqueue(0,NationId(2),7,Command::Pause(true)),Err(Error::DuplicateCommand));assert_eq!(before,ordered.state_hash().unwrap());
 let step=ordered.step().unwrap();reversed.step().unwrap();
 assert_eq!(step.commands.iter().map(|c|(c.nation.0,c.sequence)).collect::<Vec<_>>(),vec![(0,999),(2,7),(2,90),(9,8)]);
 assert_eq!(ordered.snapshot().speed(),1);assert_eq!(ordered.state_hash().unwrap(),reversed.state_hash().unwrap());
 println!("pause repeated pumps/resume, invalid speeds, past/duplicate command rejection, nation/sequence ordering: PASS");
 let hourly=vec![Phase::Commands,Phase::Movement,Phase::Combat,Phase::RetreatAndControl,Phase::Snapshot];
 let daily=vec![Phase::Commands,Phase::Movement,Phase::Combat,Phase::RetreatAndControl,Phase::Supply,Phase::Economy,Phase::Production,Phase::Construction,Phase::ManpowerTrainingReinforcement,Phase::Research,Phase::Politics,Phase::AgendaAndDecisions,Phase::Events,Phase::Diplomacy,Phase::Ai,Phase::Ledger,Phase::Snapshot];
 for (y,m,d,ey,em,ed) in [(2024,4,30,2024,5,1),(2100,2,28,2100,3,1),(2400,2,28,2400,2,29),(2400,2,29,2400,3,1),(2023,12,31,2024,1,1),(2024,5,14,2024,5,15)]{
 let mut x=sim(y,m,d);
 for _ in 0..23 {assert_eq!(x.step().unwrap().phases,hourly);}
 let step=x.step().unwrap();assert_eq!(step.snapshot.date(),Date::new(ey,em,ed).unwrap());assert_eq!(step.snapshot.hour(),0);
 let mut want=daily.clone();if ed==1 {want.insert(want.len()-1,Phase::MonthlyStatistics);}
 assert_eq!(step.phases,want);println!("calendar/phase boundary {y}-{m}-{d} -> {ey}-{em}-{ed}: PASS");
 }
 let root=std::path::Path::new("target/evidence/WP-04-verify/input-packs");let pack=root.join("other");
 std::fs::create_dir_all(pack.join("scenarios/other")).unwrap();
 std::fs::write(pack.join("manifest.toml"),"id='other'\nname_key='other'\nversion='0.1.0'\nengine='>=0.1'\n").unwrap();
 std::fs::write(pack.join("scenarios/other/scenario.toml"),"start_date='2024-04-30'\n").unwrap();
 std::fs::write(pack.join("defines.toml"),"[time]\nspeed_ms_per_tick=[81,63,47,29,0]\ninitial_speed=3\n").unwrap();
 let first=oh_cli::load_scenario(root,"other").unwrap().simulation(913).unwrap();
 assert_eq!(first.ms_per_tick(),47);assert_eq!(first.snapshot().speed(),3);
 std::fs::write(pack.join("defines.toml"),"[time]\nspeed_ms_per_tick=[93,72,55,31,0]\ninitial_speed=2\n").unwrap();
 let second=oh_cli::load_scenario(root,"other").unwrap().simulation(913).unwrap();
 assert_eq!(second.ms_per_tick(),72);assert_eq!(second.snapshot().speed(),2);assert_ne!(first.state_hash().unwrap(),second.state_hash().unwrap());
 let empty=Defines(BTreeMap::new());assert_eq!(TimeConfig::from_defines(&empty),Err(Error::InvalidTimeDefines));
 let wrong=Defines(BTreeMap::from([("time".into(),BTreeMap::from([("speed_ms_per_tick".into(),DefineValue::Array(vec![Number::Integer(0);5])),("initial_speed".into(),DefineValue::Number(Number::Integer(6)))]))]));
 assert_eq!(TimeConfig::from_defines(&wrong),Err(Error::InvalidTimeDefines));
 println!("independent file-loaded defines/date, missing and invalid time defines: PASS");
 assert!(Simulation::new("".into(),Date::new(2024,1,1).unwrap(),1,TimeConfig::new([1;5],1).unwrap()).is_err());
 println!("All independent public API checks passed");
}
