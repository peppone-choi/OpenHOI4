# M1-r3 새 독립 검증의 출력·불변성 계약

2026-10-07 부모가 main688의 각 소스에서 조사했다. 이 표는 실행 전 계획이며 새 검증자의 실제 Node env/cwd·출력경로 관측과 check-ignore를 대신하지 않는다. 구현 정확 커밋이 정해지면 해당 새 소스에서 추가 producer와 변경된 기본경로도 다시 확인한다. 구현 종료 직후 새 앱·exact detached worktree를 배정하고 최초 전체/raw snapshot을 저장한다.

| producer | 소스·기본 출력 | 새 검증에서 지정할 경계 |
|---|---|---|
| npm pretest | package.json pretest→oh_proto wire_fixtures, workspace target/wp05 | 실제 cargo manifest-root와 target 출력 확인. ignored 전용 worktree 안 |
| M0 localization | e2e/localization.spec.ts, OH_E2E_EVIDENCE 없으면 ../docs/worklog/evidence/WP-12 | suite별 OH_E2E_EVIDENCE를 새 target 절대경로로 지정 |
| M0 malformed | e2e/malformed.spec.ts, OH_MALFORMED_EVIDENCE 없으면 ../docs/worklog/evidence/WP-12/p06 | suite별 OH_MALFORMED_EVIDENCE를 새 target 절대경로로 지정 |
| M0 network | e2e/network.spec.ts, 고정 ../target/wp05 | Node cwd=source/client 확인, 존재/ignored 확인 |
| M1 national/malformed-world | e2e-m1/national.spec.ts와 malformed-world.spec.ts | 같은 두 env를 M1 suite의 별도 절대경로로 지정 |
| M1 map | map.spec.ts, OH_MAP_EVIDENCE | target/m1-r3-p05/e2e/m1/map 절대경로 |
| M1 redirect | map-redirect.spec.ts, OH_MAP_REDIRECT_EVIDENCE | 위와 별도 redirect 절대경로 |
| M1 capability | map-capability.spec.ts, OH_MAP_CAPABILITY_EVIDENCE | 별도 capability 절대경로 |
| M1 resize | map-resize.spec.ts, OH_MAP_RESIZE_EVIDENCE | 별도 resize 절대경로 |
| M1 pipeline | map-pipeline.spec.ts, OH_MAP_PIPELINE_EVIDENCE | 별도 pipeline 절대경로 |
| M1 DPR | map-dpr.spec.ts, OH_MAP_DPR_EVIDENCE | 별도 dpr 절대경로 |
| Playwright reporter/실패 | 기본 client/test-results | ignored 여부와 실제 경로 확인. 원 실패를 추적 evidence에 덮어쓰지 않음 |
| client unit 원장 fixture | oh_proto/wire 및 target/wp12 생성물 | 실제 producer 경로를 조사하고 ignored 경로 확인 |
| protocol freshness | oh_proto/examples/generate는 추적 client/src/proto/protocol.ts를 재작성 | 이 example 금지. ignored standalone harness의 실제 typescript() stdout과 Gitblob/원파일 bytes 대조, 변조1byte negative |
| 임시 full config | config가 자신의 위치에서 상대 source/server 경로를 결정 | 필요시 원 config·옵션을 그대로 유지한 copy와 exact source/client webServer.cwd를 명시. 원 server command·pack-root·GPU 변경 없음 |

모든 Node subprocess가 8개 env를 실제 상속하는지 검사한다. 고정 경로 producer는 env가 있다고 주장하지 않고 cwd/manifest-root를 확인한다. 출력 후보가 git ls-files 목록과 겹치면 실행 전에 실패시킨다. 출력이 ignored인지 git check-ignore로 확인하며 증거를 비활성화하거나 기대값을 바꾸지 않는다. 과거 WP-11 verify1의 raw index 변경과 verify2의 추적40 재작성/30내용변경을 복구·소급PASS하지 않는다.

검증 최초 baseline에는 exact HEAD, index 원문 bytes·SHA, semantic ls-files --stage -z, 전체 추적목록 -z, 각 파일SHA256, unstaged/cached binary diff, tracked status를 저장한다. GIT_OPTIONAL_LOCKS=0를 처음부터 모든 Git 읽기에 사용하고 각 주요명령 경계와 최종에 동일 baseline을 비교한다. 빌드/브라우저 산출물은 ignored 안에서 허용한다. 최초baseline 교체·index 복구·같은내용 tracked 재생성은 금지다. 검증자와 부모가 전체 비교하며 하나라도 변하면 검증 무효다.

원 fullM0/원 fullM1는 Windows 5제품 OH_BROWSER_PRODUCTS=1·원 config/순서/시간30/expect5/retry0/skip0/GPU·카메라·buffer·PNG/rawGL·cleanup 단언으로 각각1회다. 새독점port 확인과 실제 ownPID/exe/servedJS/packidentity·정상종료를 기록한다. DPR 관측묶기는 동일 epoch·원단언의 실효성을 독립 negative에서 직접검사한다. Edge Snapshot 실패 원문·원인 미확정은 별도 대조하며 이번 성공만으로 해결주장하지 않는다. 정상Linux 후속 CI를 임시진단 f19로 대체하지 않는다.

유효 P-05 PASS 뒤 부모가 P-07 로컬회귀와 문서갱신·main push를 수행하고 같은 main HEAD 일반/Save/Core/Simulation CI·각필수job·artifact를 직접확인한다. 새 P-12는 그 exact HEAD를 새앱·detached worktree·최초baseline으로 검증한다. 독립 판정 전에는 게이트를 PASS로 쓰지 않는다.
