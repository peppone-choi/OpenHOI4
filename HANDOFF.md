# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, M1-r1 W1 통합 후 W2 실행 중 |
| 현재 마일스톤 | M0 PASS 보존. M1 진행 중 |
| 기본 브랜치 CI | main b1da8f46fa7d2e849121424d5d2cf1ac36bfacd3의 [CI](https://github.com/peppone-choi/OpenHOI4/actions/runs/37474935905), [Core](https://github.com/peppone-choi/OpenHOI4/actions/runs/37474935898), [Simulation](https://github.com/peppone-choi/OpenHOI4/actions/runs/37474935881) 모두 success. 실제 M1 3OS 아티팩트 SHA와 hash60448355cecffa9d 일치도 직접 확인. 이후 문서 HEAD CI는 별도 확인 |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

M1 준비만 하라는 최초 요청 뒤 CEO가 사용자 승인 본 실행을 전달해 같은 실행에서 진행했다. RUN 종료 조건은 M1 게이트 판정·묶음 두 개 통합·결정으로 가능한 WP 없음 중 먼저 오는 것이다. W1 통합 1개를 판정했다. W1은 WP07/10/12, W2는 공용 파일 충돌을 피한 WP09→08, W3 저장 WP11이다. 묶음 두 개가 먼저 통합되면 WP11과 M1 게이트는 다음 RUN이다.

WP07 지도와 WP10 원장 기반은 독립 PASS 후 main에 통합했다. WP12는 malformed Snapshot이 화면을 비게 하는 첫 독립 FAIL을 보존하고 P06을 배정했다. 수정8831613에 새 독립 PASS를 받아 main14fa031에 통합했다. 국가/주 원장 query와 GPU 지도는 아직 후속 연결이며 합성 원장fixture를 전체AC로 판정하지 않는다. 실제 국가/주 상태 hash·save-resume는 M0 시간 hash와 구분한다.

M0 게이트 `d6ae2c8`의 AC7 PASS와 당시 동일 HEAD CI15jobs·7artifactZIP SHA는 재확인했다. [M0 게이트](docs/gates/M0.md)와 보존registry는 유지한다. M0 제품 골든·독립 예제팩은 바꾸지 않는다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP00 | CLI done1 실패 보존 | 과거 M0 기록 | 앱 정상 종료를 CLI0으로 바꾸지 않음 |
| WP01~06 | M0 통합됨 | 과거 worktree 정리·일부 제외 파일 잔여 | M0 증거 보존 |
| WP01 AI정책 후속 | 독립 PASS·main CI 성공 | codex/wp01-reviewed-ai-assets, WP-01-ai-policy | 별도 M1 묶음 횟수에 미산입 |
| WP07 | 통합됨 | codex/wp07-map-data, WP-07 | 데이터검증 범위 PASS, 실제 표시 WP08/09 |
| WP10 | 통합됨 | codex/wp10-ledger, WP-10 | 계산 기반 PASS, 실제 적용·조회 WP09 |
| WP12 | 수정 독립 PASS·통합됨 | codex/wp12-localization, WP-12 | 실제 원장 WP09 후속 |
| WP09 | 통합됨 | codex/wp09-national-state, WP-09; 구현206fb45/mainb1da8f4 | 실제 국가/주·원장·M1 hash/CLI/3OS PASS, 지도선택 WP08 후속 |
| WP08 | 구현 중 | codex/wp08-map-rendering, WP-08, baseb1da8f4 | GPU 지도·선택·패널 연결·첫화면 즉시 전달 |
| WP11 | 다음 RUN 예정 | 아직 없음 | 실제 M1 mutable state·예약명령 save-resume |
| M1 게이트 | 미판정 | 없음 | WP11까지 전체통합·독립 새검증 |

## 세션 상태와 실행 경로

사용자가 승인한 앱 별도 local 채팅·WP별명시 Git worktree 경로를 썼다. 모델·인증·샌드박스·승인·네트워크 설정은 변경하지 않았다. 모든 구현 명령은 WP worktree, 검증은 정확한 구현커밋 detached별도 worktree다. 부모도 검증 전후 HEAD/index/추적목록/각SHA/diff/status를 비교했다. 현재 채팅ID/실제상태/경로/커밋은 `.orchestrator/app-threads.json` 및 종료 시 보존복사본을 참조한다. 원본 첫 WP12 FAIL 검증 worktree는 보존했다. 현재 실행 중인 채팅은 WP08 구현 `01a11186-8f53-75c2-b9fb-873b27d8af82` host local이며 W1과 WP09 구현·검증은 모두 종료했다. [보존 registry](docs/plans/evidence/M1-r1/app-threads.json)는 이 시점 사본이다. 미병합자료를 삭제하지 않았다.

## 마지막 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| W1 통합 fmt/clippy/workspace tests·npmci/test123/build/type/license | 14fa031에서 모두 exit0 |
| M0 365일seed1 hash2 | b039d35666b77fc2 두 번 같음; M1국가hash 증거로 사용하지 않음 |
| WP07 독립 지도오류 | 94개 독립 입력, map11·Rust59(Doctest 포함) PASS |
| WP10 독립 원장계산 | 경계27436·순열120·serde/mismatch114 PASS |
| WP12 새 독립 실제 서버 UI | 5브라우저75검사·wire489+누락41+transport10 PASS |
| assets --release·정책32tests·checkdocs | 오류0·문서경고0 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D10·D12 | REQUEST0001 | 2026-10-06 | 실제답변 확정반영 | 없음; 코드GPL3later/직접데이터CCBYSA4/OpenHOI4 |
| OPEN01/03/04/05/11 상세 | REQUEST0002 | 2026-10-06 | M1~M3·미승인 문서후보 | M4/5구체규칙 |
| OPEN02 평화회의 상세 | REQUEST0003 | 2026-10-06 | M1~M3·간소화후보 | M4점수/참가/요구처리 |
| OPEN07/08 상세 | REQUEST0004 | 2026-10-06 | M1~M3·별도후보 | M4weather/occupation/spy세부 |
| OPEN12 업적 상세 | REQUEST0005 | 2026-10-06 | M1~M5·소수업적후보 | M6조건·기록·mod/multi |

OPEN01~12는 일괄 기본안승인 철회 후 모두 개별 사용자답변으로 범위를 반영했다. 포함 결정과 구체 규칙의 잠정 채택은 다르다. 준비방식 A만 CEO잠정채택이며 현재의 미완성 defines/공식 후보는 미승인이다. REQUEST가 있다는 이유로 모든 세부 공식을 사용자 전용으로 분류하지 않는다. 기존 승인 범위의 가역적 추천안은 구체 입력·거부·상태변경·출력·경계 기대값·저장호환/되돌림 비용을 완성해 CEO 검토를 받는다. 채택 뒤 01에 잠정 규칙을 기록하고 구현한다. 확정 상태·범위·복구 어려운 삭제·설정·비용·공개·법적 판단은 사용자 전용이다. 함선설계와 공개호스팅·계정은 후속로드맵에 보존한다. 공개배포·태그·실제운영비용은 승인하지 않았고 실행하지 않았다. 이름 확정을 법적상표검증으로 주장하지 않는다.

후속 검토 순서: M1을 우선 끝낸 뒤 M4 차량/기상/점령/첩보와 간소화 평화 회의 후보의 계산·경계를 완성하고 CEO의 권한/가역성 검토로 넘긴다. 다음은 M5 장비/공군/함대/특수무기와 스크립트 기술 선택, 마지막은 M6 업적이다. REQUEST0005의 서버 설치 공유 프로필은 다인 플레이에서 타인 기록을 합치는 한계가 있어 기존 세션·국가 배정 기준의 국가 기록과 개인별 표시/보존 대안을 비교한다. 공개 계정을 전제하지 않는다. 최종 M6에서는 기존 후순위 표기와 별개로 승인 확장 목록의 WP·독립 PASS·CI 증거를 대조한다.

## 알려진 문제

| 문제 | 재현 방법·범위 |
|---|---|
| WP12 최초 빈화면 오류 | originala91d324 malformed Snapshot, 첫FAIL 보존·8831613수정·새PASS |
| Windows WebKit 폰트 얇음 | loaded/axis/glyph는통과, 실제Safari 미검증, 화면/제한기록 |
| 일부 M0 provenance/라이선스설명 구식 | ASSETS/deny의 D10미정 당시문구·workspace metadata 후속정리; 확정정책은01/02/README |
| validate/AIbench/repro/완결게임/저장 일부 후속 | 미구현명령을 실행했다고 기록하지 않음 |
| 초기 앱 자동worktree·WP03verify 잔여 제외파일 | 이전 HANDOFF 기록의 미확인thread/제외target 보존, 강제삭제하지 않음 |

사용자 지리 지시 반영: 자료는 취득 단계에 고정파일·URL/date/version/license/SHA로 보존하고 게임 중 외부지리API·타일·CDN을 쓰지 않는다. DEM능선/산맥/고개/강 자료를 offline생성·검수에 포함한다. 원작공개화면은 분할원칙/밀도만참고하는 사용자한정예외이며 경계복사/트레이싱/파일추출/수치표는금지한다. NE/OHM라이선스예외·ETOPOCC0후보 공식자료를 확인했으나 원천취득/채택/전세계산출은 후속 WP32/45다. 새이동·전투규칙을 만들지 않는다.

## 다음 세션이 처음 할 일 (3개 이내)

1. 현재 registry와 WP08 실제 HEAD/상태/증거를 확인한다. 첫지도 즉시 전달, 별도 실제 UI플레이와 새독립 검증을 진행한다. WP09 실제M1 3OS 증거는 이미 확인했으며 저장재개와 혼동하지 않는다.
2. 두 묶음이 통합되면 이 RUN을 끝내고 HANDOFF 커밋·푸시 후 정확한 최종 HEAD CI를 확인한다. 저장 WP11은 다음 RUN에서 실제 M1 state와 새독립 DT/3OS 증거로 진행한다.
3. 전체필수14REQ/AC7를새독립게이트로판정하고 기록. M1이PASS될때까지M1완료라고쓰지않음.

WP09 후속 검증: M1 실제server15/M0server60·독립브라우저110·TOML변조29·Fx최하위bit/expiry/queue/config/다중주실패원자성 PASS. 부모1515 tracked SHA/index/list/diffstatus 무변경. M1 365일seed1 두회 b595dc2a1e5b4f8c, 1000tick60448355cecffa9d. [전문](docs/verify/WP-09.md), [세OS 원본ZIP metadata](docs/verify/evidence/WP-09/main-ci/same-head-ci.json).
