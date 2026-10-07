# Actual producer output boundaries

Exact source root: E:/openhoi/.orchestrator/wt/WP-08-M1-r3-verify. Commands and actual child Node cwd/env are in each label/context.json. All eight configurable OH_*_EVIDENCE variables are absolute target/m1-r3-p05/e2e/<label>/<variable>. Each child receives GIT_OPTIONAL_LOCKS=0; reporter and TMP/TEMP/PWTEST_CACHE_DIR also use ignored absolute paths. Guards reject tracked output destinations before executing producers, including a negative probe of client/src/proto/protocol.ts.

| Producer/source | Effective location | Checked boundary |
|---|---|---|
| npm pretest / crates/oh_proto/examples/wire_fixtures.rs | manifest-root ../../target/wp05/wire-fixtures.json and target/wp09/national-wire-fixtures.json | fixed source paths, ignored; not falsely env-configurable |
| client/src/i18n.test.tsx | client cwd ../target/wp12/ledger-ko.html and ledger-en.html | original fixed ignored unit outputs |
| client/e2e/network.spec.ts | client cwd ../target/wp05/<product>-browser.json and running/paused/ko.png | fixed ignored outputs, collected under evidence root later |
| localization.spec.ts and national.spec.ts | OH_E2E_EVIDENCE absolute per suite | default docs/worklog/evidence/WP12 forbidden |
| malformed.spec.ts and malformed-world.spec.ts | OH_MALFORMED_EVIDENCE absolute per suite | default docs/worklog/evidence/WP12/p06 forbidden |
| map/redirect/capability/resize/pipeline/dpr specs | six OH_MAP_* evidence variables, absolute per suite | mkdir/write paths inspected in exact source |
| Playwright configs | source client cwd, original testDir/testMatch/projects/order; --output and JSON reporter absolute | defaults client/test-results and playwright-report also ignored; no full-config copy |
| npm build / oh_server/build.rs | ignored client/dist; Rust OUT_DIR assets.rs and target/debug/oh_server.exe | served bytes independently match dist files |
| cargo tests/save fixtures | original fixed target subdirs and per-command ignored TEMP; native capture under target/m1-r3-p05/save/capture | source reviewed; no tracked save/golden writes |
| protocol standalone harness | ignored proto-harness Cargo.toml/Cargo.lock/src; stdout proto-valid/command.log | tracked generate example never executed |
| independent original fault copies | ignored faults/*.spec.ts, original assertions preserved; observation values/actual DOM deliberately corrupted | no tracked source/index mutation; expected failures preserved |

Before producers, nonexistent directory check-ignore without trailing slash failed; corrected ignored wrapper only. Actual npm build returned0 before wrapper console UnicodeEncodeError; log/result/invariance retained without repeating build. prepare.py first UTF8 source read used platform encoding and failed before tests; fixed ignored preparation only. Initial protocol harness dependency path was one directory too high (cargo101 before code execution), preserved proto/; corrected harness ran under proto-valid/. These are preparation failures, not product test retries.
