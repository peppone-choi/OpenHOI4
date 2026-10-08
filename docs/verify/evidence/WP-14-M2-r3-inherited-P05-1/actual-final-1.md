# 검증 리포트 — WP-14 M2-r2 P05-1

| 항목 | 값 |
|---|---|
| 판정 | **FAIL — v5 복원의 건물 단계 상한 검증 누락** |
| 검증자 | Codex 독립 검증 세션, 구현 비참여·추적 파일 무변경 |
| 브랜치·커밋 | `codex/wp14-economy-politics-m2r2` / `a9333419de89caf3ba95da5097ea2e3d0fa5f694` |
| 제품 source | `01119ae05013a1974e5ca096c705bf39846c9cff`와 non-doc Git 동일 |
| 검증 일시 | 원검증 및 이번 재개 종료: 2026-10-08 KST |

이번 재개에서는 원증거와 whole/raw 불변성만 확인했다. 새 소스·제품 테스트는 실행하지 않았다. 중단된 원 turn에 final/completedAt가 있었다고 소급하지 않는다.

## 직접 실행한 명령

아래 제품 검사 결과는 보존된 원실행 기록이다. 전체 절대 argv·cwd·종료 코드·출력은 [원 판정 기록](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/verification-result.json)에 있다.

| 명령 | 종료 코드 | 결과 요약 |
|---|---|---|
| `cargo fmt --check` | 0 | 통과 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | 통과 |
| `cargo test --workspace --locked` / workspace build | 0 / 0 | 기존 검사 통과 |
| client ci·typecheck·build·test | 0 | 12 files, 246 tests |
| M0 전체 E2E | 0 | Chromium/Firefox/WebKit 36개 |
| M1 전체 E2E, Chromium/WebKit | 0 | 단일 전체 실행 82개, None economy의 기존 QueryResult 거부 확인 |
| current/frozen 저장·trigger·normal/force 서버·repro | 0 | native 저장·재개·조회 검사 |
| 독립 domain probe 및 v5 presence 6조합의 실제 CLI resume/save-out/repro | 0 | 전체 DTO/canonical/hash·future queue 대조 |
| loader 입력·save-current·trigger·bench 회귀 | 기대 코드 일치 | 각각 17·25·16·18개 |
| 원 metadata를 사용한 licenses/bans/advisories/sources | 0 | 원정책·동일 registry index·fresh advisory DB 유지 |
| architecture·assets·docs 검사 | 0 | 통과 |
| Windows 고정 입력 벤치 | 0 | 로컬 결과로 별도 보존 |
| 실제 세 OS original/current-v5/trigger compare | 0 / 0 / 0 | 원격 artifact 9개 대조 |
| `restore_stage_probe` | **2** | 단계 상한 반례로 독립 predicate FAIL |
| level4 저장의 CLI resume normal / `--force` | **0 / 0** | 기대1과 달리 모두 수락 |
| level4 초기 데이터의 CLI run | 1 | 정상 거부 |
| level4 저장의 서버 normal / force | **HTTP200 / HTTP200** | 권위 조회가 industry level4 반환 |
| 이번 재개: snapshot·원 ZIP·물리 mapping·전달 custody 대조 | 0 | whole/raw 동일, 불일치0 |

## REQ별 확인

| REQ | 검사하는 테스트 | 실제 검사 여부 | 결과 |
|---|---|---|---|
| REQ-ECO-01 | 소유 주·비연속 ID·IC 정수식·wide Qty·원장·score | 직접 | 일반 산출 통과, 복원 수준 상한 FAIL |
| REQ-ECO-02 | 정확합1·잔여·하한 경계·transaction | 직접 | 검사 범위 통과 |
| REQ-ECO-04 | 소유 주 자원 흐름·통제 구분·overflow | 직접 | producer 범위 통과 |
| REQ-ECO-06 | 단계·슬롯·중복·skip·올림·소유권·휴면·복원 | 직접 | **FAIL: 3단계 정의에서 level4 복원 수락** |
| REQ-ECO-07 | capacity·committed/reserved·내부 API·rollback | 직접 | producer 범위 통과 |
| REQ-NAT-02 | 날짜 경계 PC·선지급 거부·Pause·cap·rollback | 직접 | 검사 범위 통과 |
| REQ-NAT-03 | 정치 지표·효과 transaction·종료 checkpoint | 직접 | producer 범위 통과 |
| REQ-NAT-04 | 법령 범주·참조·양수 비용·조건·직전 상태 | 직접 | 검사 범위 통과 |

WP15/16/20/29의 생산·훈련/보충·군사/항복·시장 consumer와 전체 REQ 완료를 주장하지 않는다.

## 결정론

- 해시 1: M0 `ff921fd8148e699d`, M1 `60448355cecffa9d`.
- 해시 2: 각 반복 실행이 동일.
- 저장 후 재개: v5 split `48e02ba01e8fddfe`, paused `263071b34065aa8d`, continued `373a5a6eeea13715`. 실제 세 OS 원본의 전체 DTO/canonical/hash·bytes·identity 일치.
- 초기 독립 인코더 fixture는 movement/strait presence=false에 한정하며, 별도 6조합 검사를 전체 consumer 지원으로 확대하지 않는다.
- 실제 Linux: run `37695537881`, attempt1, job `113046236165`, artifact `11514894569`. 팩 hash `3bde1bed90d3734e`, 완료 hash `3b8853acbddce251`. baseline ns `[195802551,193460349]`, current ns `[192623408,193688056]`; 회귀 없음. 원 두 쌍·15%·240s/1200s 유지.

## 금지 사항 점검

| 항목 | 결과 |
|---|---|
| 시뮬레이션의 f32/f64 | production source에서 발견 없음. Fx I32F32·Qty I48F16·인력 i64 확인 |
| HashMap 순회 | 결정론 계층에서 발견 없음 |
| thread_rng / SystemTime | 시뮬레이션에서 발견 없음 |
| 코드 내 수치 리터럴 | 신규 경제 수치는 명시 정의 입력 |
| 미결정 규칙 구현 | REQUEST-0010 A-CEO-r1 잠정 상태 유지 |
| 골든 기대값 변경 근거 | 원 테스트·골든·expected·팩·lock·일곱 workflow 보호 경로 변경0 |
| 클린룸 | 검증 중 금지 원작 자료 사용 없음 |

WT reset·제품/추적 파일·원before 수정·commit·새 앱/세션·CUA/focus/global input·정책/한도 변경 없음.

## 작업 로그 증거와 실제 결과 대조

- 원 browser FAIL과 호환 수정 후 전체 통과, trigger·준비 입력·UTF8·공간 오류를 각각 보존했다.
- `planning/remote-final`에 같은 source의 실제 원격 자료가 전달되어 있었다. source CI 7종·26 jobs 성공, 저장/trigger artifact 9개와 Linux 증거를 원검증에서 대조했다. 이번 재개에서도 전달 custody 96개 파일의 길이·SHA 불일치0을 확인했다.
- 원격 executable 객체·전체 library tree·external source.lock 복사본이 업로드되지 않은 한계는 유지한다. 기록된 SHA·metadata/inventory·Git/prior TAR·runtime guard 이상의 물리 인수를 주장하지 않는다.
- **CI 성공에도 stagecap 반례가 존재하므로 최종 판정은 FAIL이다.** 초기 loader와 v5 restore가 같은 건물 단계 상한을 적용하지 않는다.
- 최초·원 final-before/after·이번 resume의 11,833개 추적 파일과 whole JSON/raw index가 모두 동일하다. raw index SHA256: `4d77c338769c8470b36519c468147b1736cf1fae467ce05070a1697321215a1c`.
- main CI·P07·W2/M2 gate·사용자 확정을 대신하지 않는다.

원본 증거:

- [원 FAIL ZIP](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1-final-FAIL.zip): **136,217,870 bytes**, SHA256 `ddcf54590e7285f6f48d4a8285f73da6d2012470bdff7c729d6d1866dbade8c3`. 수정·재포장 없음.
- [member→physical mapping](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/bundle-final-FAIL/member-physical-CRC.json): ZIP 14,173 members CRC와 14,172개 물리 항목 대조 불일치0.
- [원 final-before](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/verifier-final-before-seal.json) · [원 final-after](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/verifier-after-final-seal.json) · [이번 whole/raw 비교](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/resume-M2-r3-whole-raw-summary.json).
- [이번 custody 인수 결과](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/resume-M2-r3-custody/audit.json).
- [상세 리포트 전문](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/resume-M2-r3-custody/VERIFY_REPORT.md): SHA256 `1f2312f23d4733c18257d1da4cd7f5fcf75c61ded76ae90fe6228b9d0e253c7d`. ignored 영역에만 저장.

## FAIL 항목

| # | 재현 명령 | 기대 | 실제 |
|---|---|---|---|
| F-01 | 원 CLI resume, level4 v5 저장, normal | 3단계 초과 거부, exit1 | exit0, 수락 |
| F-01-force | 같은 CLI resume `--force` | 의미 검증 유지, exit1 | exit0, 수락 |
| F-01-server | 같은 저장의 server normal/force | startup exit1 | 둘 다 HTTP200, 권위 query level4, 종료0 |

반례 [industry-level4.ohsave](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/restore-stage-case/industry-level4.ohsave)는 **458 bytes**, SHA256 `c67f5cefbba865be78643b920536678f789b8375b418b9034ddd2cd2b4db1e53`이다. [서버 원영수증](/E:/openhoi/.orchestrator/wt/WP-14-M2-r2-verify1/target/evidence/WP-14-M2-r2-P05-1/restore-stage-case/server-both-ready.json)과 CLI 원출력을 보존했다. source 수정 없이 부모의 P06 인수 대상으로 제출한다.
