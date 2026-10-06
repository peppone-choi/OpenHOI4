fn main(){let s=oh_sim::Simulation::new("x".into(),oh_sim::Date::new(2024,1,1).unwrap(),1,oh_sim::TimeConfig::new([1;5],1).unwrap()).unwrap();s.state.tick=8;let mut snap=s.snapshot();snap.speed=5;}
