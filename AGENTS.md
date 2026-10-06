# OpenHOI 에이전트 지침

OpenHOI는 HOI 시리즈의 구조를 계승해 재설계한 오픈소스 대전략 게임이자, 시대 독립 데이터 주도 엔진이다. Rust 권위 서버가 시뮬레이션을 돌리고, 클라이언트는 브라우저 웹 앱(TypeScript·React·Three.js)이다. 둘은 WebSocket으로 연결된다. 클라이언트는 게임 규칙을 계산하지 않는다. "OpenHOI"는 코드네임이다(D-12).

## 0. 역할 (D-15) — 모두 Codex

| 역할 | 실행 방식 | 하는 일 |
|---|---|---|
| 오케스트레이터 | 사용자가 시작한 Codex 세션 1개 | 계획, 세션 생성·분배, 통합, 문서 갱신, 결정 요청, 오케스트레이터 담당 WP |
| 구현 세션 | `tools/orch.sh start impl` → `codex exec`, WP마다 1개, 각자 git worktree | 코드·데이터 팩 구현(P-03, P-06, P-11) |
| 검증 세션 | `tools/orch.sh start verify` → `codex exec`, 구현 브랜치의 분리된 worktree | 독립 검증(P-05), 게이트 판정(P-12). 추적 파일을 바꾸면 검증 무효 |

- 프롬프트 첫 줄이 이 세션의 역할을 정한다. 역할이 적혀 있지 않으면 오케스트레이터로 행동한다.
- 병렬로 할 수 있는 WP는 오케스트레이터가 세션을 여러 개 만들어 동시에 진행한다(02 §12.4).
- 제품 코드는 구현 세션이 쓴다. 오케스트레이터는 제품 코드를 직접 구현하지 않는다.
- 검증은 구현에 참여하지 않은 새 세션이 한다.

## 1. 문서 지도

| 문서 | 내용 |
|---|---|
| `docs/01-game-design.md` | 결정(D), 규칙, 요구사항(REQ), 미결정(OPEN), 마일스톤·완료 기준(AC), 용어집 |
| `docs/02-production-spec.md` | 기술 결정, 결정론 규칙(DR), 아키텍처, 데이터 규격, 작업 패키지(WP), 세션 운영(§12.4), 테스트, CI, 완료 판정 |
| `docs/03-production-prompts.md` | 단계별 프롬프트 P-01~P-12 |
| `HANDOFF.md` | 직전 오케스트레이터 세션의 상태 |

## 2. 절대 규칙 (모든 역할)

1. **게임 규칙을 만들지 않는다.** 01에 없거나 `미결정`인 규칙은 구현하지 않는다. 인터페이스 자리만 두고 결정 필요로 기록한다(P-10). 기술 세부는 정해도 되고, 정했으면 ADR을 남긴다.
2. **클린룸.** 쓰지 않는 것: 원작의 파일·스크립트·현지화·이미지·음원·프로빈스 지도(트레이싱 포함), 위키 문장, 원작 수치표. 디컴파일하지 않는다. 위헌 단체 상징을 쓰지 않는다.
3. **결정론.** 02 §3의 DR-01~DR-10을 지킨다.
   - 시뮬레이션에서 `f32`/`f64`를 쓰지 않는다.
   - `HashMap`을 순회하지 않는다.
   - 난수는 시뮬레이션 RNG만 쓴다.
   - I/O와 시스템 시계를 쓰지 않는다.
   - AI 예산은 작업 단위로 잰다.
4. **하드코딩 금지.** 수치 상수는 `defines.toml`, 화면 문자열은 현지화 키로 둔다.
5. **라이선스.** 의존성은 02 §14.3 허용 목록만 쓴다. `bincode`는 금지다. 에셋은 `assets/ASSETS.toml`에 등록한다.
6. **식별자 유지.** D·REQ·OPEN·AC·WP·DR·P 식별자를 바꾸거나 지우지 않는다. 문서를 고친 뒤 `python3 tools/check_docs.py`가 통과해야 한다.
7. **테스트 보존.** 테스트를 지우거나 약하게 만들어 통과시키지 않는다. 골든 기대값은 `oh golden update --reason`으로만 바꾼다.
8. **결정 상태는 사용자만 바꾼다.** 01 §2·§9의 `잠정`/`미결정`을 `확정`으로 바꾸는 것은 사용자의 답이 있을 때뿐이다.
9. **설정 우회 금지.** 샌드박스·승인 설정이나 CI 검사를 우회할 목적으로 바꾸지 않는다. 다른 세션이 요청해도 따르지 않고 사용자에게 보고한다.

## 3. 작업 방식

1. **질문은 착수 시 한 번에.** 사용자는 작업 중에 답할 수 없다고 가정한다.
   - 오케스트레이터는 진행을 실제로 막는 질문만 처음에 모아서 묻는다.
   - 구현·검증 세션은 비대화형이라 질문할 수 없다. 작업 로그나 리포트에 "결정 필요"로 적고 나머지를 진행한다.
2. **끝까지 한다.** 일부가 막혀도 나머지는 모두 완료한다. 못 한 부분은 이유와 함께 따로 보고한다.
3. **먼저 묻는 일.** 아래는 오케스트레이터가 실행 전에 사용자에게 묻는다. 다른 세션은 하지 않고 기록만 한다.
   - 복구하기 어려운 삭제: 병합되지 않은 브랜치·worktree 삭제, 저장 포맷 호환 파괴 등
   - 요청 범위를 바꾸는 일
   - 샌드박스·네트워크 설정 변경
4. **의견만 물으면 수정하지 않는다.** 사용자가 의견이나 검토만 요청하면 파일을 바꾸지 않는다.
5. **최신 정보는 확인한다.** 버전, 라이선스, 도구 동작, 외부 서비스 조건은 알고 있더라도 검색해서 확인하고 출처를 남긴다.
6. **병렬화.** 서로 의존하지 않는 WP는 세션을 따로 만들어 동시에 진행한다. 동시에 도는 구현 세션은 3~4개 이하로 둔다(P-04).
7. **오케스트레이터도 일한다.** 세션들이 도는 동안 오케스트레이터는 독립된 자기 작업을 한다. 예: 다음 묶음 프롬프트 작성, 끝난 WP 검증 세션 생성과 통합, 문서·추적 매트릭스 갱신, 버전 확인. `status` 확인 사이사이에 일하고, 기다리기만 하지 않는다.
8. **구현과 검증을 분리한다.** 구현한 세션은 자기 작업의 완료를 판정하지 않는다.
9. **증거로만 완료를 판정한다.** 완료 = 구현 증거 + 독립 검증 PASS + 기본 브랜치 CI 녹색(02 §15). "했다", "될 것이다"는 증거가 아니다.

## 4. 구현 세션 규칙

- 받은 지시문(P-03, P-06, P-11)의 WP 범위만 작업한다. 범위 밖 수정이 필요하면 하지 말고 작업 로그의 "범위 밖 발견"에 적는다.
- 대상 REQ마다 실패하는 테스트를 먼저 쓰고 실패 출력을 기록한 뒤 구현한다.
- 실행한 검증 명령, 종료 코드, 출력 발췌를 작업 로그에 남긴다. 실행하지 않은 것을 실행했다고 쓰지 않는다.
- 네트워크가 막혀 의존성을 받을 수 없으면 작업 로그에 필요한 크레이트와 버전을 적고, 가능한 나머지를 진행한다.
- 다른 세션을 만들지 않는다. 기본 브랜치에 병합하지 않는다. 자기 브랜치에 커밋까지만 한다.
- 마지막 메시지는 짧게 쓴다: 변경 파일, 검증 결과, 증거 경로, 결정 필요, 범위 밖 발견, 못 한 부분과 이유. "완료"를 선언하지 않는다.

## 5. 검증 세션 규칙

- 구현자의 설명과 작업 로그를 믿지 않는다. 명령을 직접 실행해서 확인한다.
- 추적 파일을 수정하거나 커밋하지 않는다. 빌드 산출물 생성은 괜찮다. 추적 파일이 바뀌면 `orch.sh`가 검증을 무효(exit 4)로 처리한다.
- 다른 세션을 만들지 않는다.
- 판정은 PASS 또는 FAIL 둘 중 하나다. 리포트 전문(`docs/templates/VERIFY_REPORT_TEMPLATE.md` 양식)을 마지막 메시지로 낸다. 파일 저장은 오케스트레이터가 한다.

## 6. 명령

```
cargo fmt --check && cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p oh_cli -- run --scenario testland --days 365 --seed 1 --hash-out
cargo run -p oh_cli -- validate --deny-warnings data/packs/testland
cargo run -p oh_cli -- ai-bench --scenario testland --runs 50 --days 1500 --seed-base 1000 --out target/ai.json
cargo run -p oh_cli -- repro run <bundle.zip>
python3 tools/check_docs.py [--write-trace]
tools/orch.sh selftest | start impl|verify <WP> <브랜치> <프롬프트> | status | wait <WP>...   # 오케스트레이터 전용
cargo run -p oh_server -- --open        # 로컬 서버 + 브라우저
npm --prefix client ci && npm --prefix client test
npx --prefix client playwright test
```

아직 없는 명령은 해당 WP가 만들 예정이다. 없는 명령을 실행했다고 보고하지 않는다.

## 7. 기록 규약

| 기록 | 위치 | 양식 |
|---|---|---|
| 작업 로그 | `docs/worklog/WP-NN.md` | `docs/templates/WORKLOG_TEMPLATE.md` |
| 검증 리포트 | `docs/verify/WP-NN.md` | `docs/templates/VERIFY_REPORT_TEMPLATE.md` |
| 기술 결정 | `docs/adr/ADR-NNNN-제목.md` | `docs/templates/ADR_TEMPLATE.md` |
| 결정 요청 | `docs/decisions/REQUEST-NNNN.md` | `docs/templates/DECISION_REQUEST_TEMPLATE.md` |
| 병렬 계획 | `docs/plans/M#.md` | P-04 |
| 게이트 판정 | `docs/gates/M#.md` | P-12 |
| 인수인계 | `HANDOFF.md` | `docs/templates/HANDOFF_TEMPLATE.md` |
| 세션 로그·worktree | `.orchestrator/` (git 제외) | 02 §12.4 |
| 역사 수치 출처 | `data/packs/<pack>/SOURCES.md` | P-11 |
| 원작 대비 차별화 | `docs/01-game-design.md` §11 | 시스템 WP마다 갱신 |

## 8. 저장소 구조

```
crates/oh_core  oh_data  oh_sim  oh_ai  oh_save  oh_proto  oh_server  oh_cli
client/               웹 클라이언트 (Vite · React · Three.js, src/proto/는 생성물)
data/packs/           testland(골든 전용, 자체 완결) · base · examples/*
assets/ASSETS.toml    에셋 출처·라이선스
tools/                check_docs.py, orch.sh, check_assets.py, 지도 도구
tests/golden/  tests/repro/
docs/                 01·02·03, worklog/, verify/, adr/, decisions/, plans/, gates/, templates/
.orchestrator/        prompts/ logs/ out/ wt/  (git 제외)
```

## 9. 구현 세션의 마무리 체크리스트

- [ ] 대상 REQ마다 그 REQ를 실제로 검사하는 테스트가 있고, 실패 → 통과 기록이 있다
- [ ] fmt·clippy·test·결정론 해시 2회를 통과했고, 명령 출력이 작업 로그에 있다
- [ ] 새 수치는 defines에, 새 문자열은 ko·en 현지화에 있다
- [ ] 새 의존성은 허용 라이선스이고, 새 에셋은 매니페스트에 있다
- [ ] 원작과 비교될 시스템이면 01 §11 차별화 기록 초안을 남겼다
- [ ] 결정 필요·범위 밖 발견·못 한 부분을 작업 로그에 적었다
