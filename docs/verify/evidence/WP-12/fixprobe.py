from pathlib import Path
p=Path('E:/openhoi/.orchestrator/wt/WP-12-verify2/target/wp12-verify2/wire.mjs')
s=p.read_text(encoding='utf-8').replace("import ts from '../../client/node_modules/typescript/lib/typescript.js';","import {stripTypeScriptTypes} from 'node:module';")
s=s.replace("ts.transpileModule(src,{compilerOptions:{module:ts.ModuleKind.ESNext,target:ts.ScriptTarget.ES2022}}).outputText","stripTypeScriptTypes(src,{mode:'transform'})")
p.write_text(s,encoding='utf-8')
