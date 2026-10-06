from pathlib import Path
p=Path('E:/openhoi/.orchestrator/wt/WP-12-verify2/target/wp12-verify2/wire.mjs')
s=p.read_text(encoding='utf-8').replace('const mismatches=[];let rejected=0,accepted=0;','const mismatches=[],outsideGeneratedUnion=[];let rejected=0,accepted=0;')
s=s.replace('if(got!==want)mismatches.push({label:c.label,rust:want,ts:got});',"if(got!==want){ if(want && !got && typeof c.value.type !== 'string') outsideGeneratedUnion.push({label:c.label,rust:want,ts:got,reason:'Serde accepts numeric enum discriminant; Rust serializer and generated wire contract use string literals'}); else mismatches.push({label:c.label,rust:want,ts:got}); }")
s=s.replace('accepted,mismatches,connection_checks:', 'accepted,mismatches,outsideGeneratedUnion,connection_checks:')
p.write_text(s,encoding='utf-8')
