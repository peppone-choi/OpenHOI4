import pathlib
p=pathlib.Path(r'E:/openhoi/.orchestrator/wt/WP-09-verify/target/wp09-verify/probe/src/main.rs')
t=p.read_text();needle=' println!("hash oracle/ordered inputs/modifier expiry/queue error state PASS");'
insert='''
 let mut config=d.clone();config.pack.defines.0.get_mut("time").unwrap().insert("initial_speed".into(),oh_data::Number::Integer(2));assert_ne!(make(&config,1).state_hash().unwrap(),h);
 let mut d2=d.clone();d2.map.states.iter_mut().find(|s|s.id==2).unwrap().infrastructure=1_500_000_000;
 d2.scenario.state_modifiers.insert(1,vec![ModifierInput{source:"north-add".into(),target_stat:"infrastructure".into(),operation:"add".into(),value:"1".into(),expires:Some(2)}]);
 d2.scenario.state_modifiers.insert(2,vec![ModifierInput{source:"south-add".into(),target_stat:"infrastructure".into(),operation:"add".into(),value:"-1000000000".into(),expires:Some(2)},ModifierInput{source:"south-mul".into(),target_stat:"infrastructure".into(),operation:"mul".into(),value:"2".into(),expires:None}]);
 let mut atom=make(&d2,1);atom.step().unwrap();atom.enqueue(1,NationId(1),5,Command::SetSpeed(5)).unwrap();let before=oh_core::canonical_bytes(&atom).unwrap();assert!(atom.step().is_err());assert_eq!(oh_core::canonical_bytes(&atom).unwrap(),before);assert_eq!(atom.world().unwrap().state(StateId(1)).unwrap().infrastructure(),Fx::from_num(2));assert_eq!(atom.snapshot().speed(),1);assert_eq!(atom.snapshot().tick(),1);assert_eq!(atom.pending_commands().len(),1);
 println!("multi-state partial ledger failure preserves world/time/config/queue canonical bytes PASS");
'''
assert needle in t;p.write_text(t.replace(needle,insert+needle))
