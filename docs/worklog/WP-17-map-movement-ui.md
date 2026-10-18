# WP-17 S2 지도 이동 UI 구현 후보

- taskId: openhoi-s2-map-movement-ui-20261016
- contract: a69fdb74-3960-4ee6-a9bd-309d8a7adc3f
- project/stage: openhoi / implement
- repository: https://github.com/peppone-choi/OpenHOI4
- branch: buzz/openhoi-s2-map-movement-ui-20261016/openhoi-implement
- base: 876375c58c411f1641eb98844f3903e8ba7b1285
- 계획 SHA256: 2327316289cf583677aea6cf7161323ee47f0247caefab1a70ab176907c5a413
- 계획 인계 SHA256: f2668b6c856204576809cdcc45993f70b9ca5baadd5563e18afacda82cb7db4d

## 범위와 서버 권위

확정 계획 R1–R9, D1–D7만 구현했다. 자기 국가 사단 선택 후 우클릭 또는 1회성 목적지 지정, 정지, 사단별 수락/거부/미전송을 제공한다. 묶음은 id 오름차순이며 CommandLedger 공용 sequence와 64건 한도를 사용한다. 마지막 ACK 이후 발행한 movement 조회가 도착해야 다음 조작을 허용한다.

SVG 점은 서버 province와 route 순서, 색은 world nation 색이다. 자기 국가만 화살표를 그리며 이동 마커는 모든 국가에 표시한다. 이동 비율은 서버 FixedValue.bits의 BigInt 나눗셈 후 0~1 표시값으로 변환하며 조회 사이 보간·경로 탐색·시간 예측·통제 판정을 하지 않는다. 남은 시간은 remaining_hours.value 그대로다. invalid-message의 실제 원인은 추정하지 않는다.

군사 패널 열기, Snapshot/Delta, 마지막 ACK 때만 조회하며 한 건 대기와 dirty follow-up으로 합친다. 닫기/연결/Join 변경은 선택과 이동 상태를 폐기한다. 오류 참조는 map-data-error로 전체 오버레이를 숨긴다. transport/protocol/서버/팩/기존 브라우저 spec·baseline·CI workflow는 바꾸지 않았다. Q1과 M-1 지도 국가 소유 결정은 하지 않았다.

## 검증 근거

실제 로그는 담당 작업실 reports/dispatch/a69fdb74-3960-4ee6-a9bd-309d8a7adc3f에 보존한다.

- RED: f80e82e, 빈 골격에서 신규 9개 실패, 기존 마커 5개 PASS. unit-red.txt.
- GREEN: 로컬 `cd client && npx vitest run src/map/movementOverlay.test.ts src/movementCommands.test.ts src/map/divisionMarkers.test.ts`, 15개 PASS. unit-green.txt.
- 로컬 `npm --prefix client run typecheck`: 최초 fixture fractional_bits 누락 FAIL(typecheck-first.txt) 후 보완 PASS(typecheck-green.txt). 실패 로그 보존.
- `git diff --check`: PASS.
- hosted CI / 실제 브라우저: 제출 뒤 exact head 영수증은 최종 인계 reports를 따른다. 현 문서 시점 NOT_RUN. 기존 CI의 m1 config에 movement.spec.ts만 추가했다. native host마다 finally SIGINT/SIGKILL과 PID 부재 근거를 저장한다.
- protocol 생성 no-diff, 기존 networkValidation, 기존 e2e·baseline 및 전체 회귀: hosted CI에서 확인. 현 문서 시점 NOT_RUN.

## NOT_RUN / 다음 게이트

로컬 Rust 빌드·서버·브라우저는 계획대로 NOT_RUN. Windows GUI, HOI4, 실기기, GCP, 비용/credential/권한 변경, 배포·초기화 NOT_RUN. 실제 세계 지도 인수 AC10은 M-1 병합 후 openhoi-verify의 별도 게이트로 NOT_RUN이다. testland 통과는 실제 지도 완료가 아니다.

독립 리뷰·검증·병합 NOT_RUN. 다음 담당 openhoi-review가 exact head를 독립 확인하고 openhoi-verify → openhoi-merge로 넘긴다. 구현 담당은 직접 병합하지 않는다. 신규 정책·국가 소유 결정 필요 없음.

## Hermes 재개: PR24 실패 CI 수정 (t_a87b80c5)

- 보존 worktree는 그대로 두고 별도 `hermes/t_a87b80c5`에서 PR HEAD `317ffdaf8561c4a133da1f5fe87164b9cafb1bef`를 이어갔다. main `876375c58c411f1641eb98844f3903e8ba7b1285`의 CI `39058171744` SUCCESS와 비교했다.
- 원본 CI `38862287948`/save `38862287979`: Windows/macOS의 `MovementOverlay.tsx`와 `movementOverlay.ts` 모듈 대소문자 충돌을 `MovementDisplayOverlay.tsx`로 분리했다. map 모듈 stem 충돌 회귀시험을 추가했다.
- DPR 시험은 S2 contextmenu 등록/해제 한 쌍을 반영해 정확한 listener 기대값을 6에서 7로 바꿨다. dispose 후 모든 자원이 0인 검사는 유지했다. 기존 이미지 baseline/CI workflow 변경 없음.
- 원본 wire에서 무제한 speed5가 관측 전에 목적지(tick 27008, province 14, route 빈 배열)에 도착했다. 중간 비율/Stop 관측은 유한 speed3로 바꾸고 경로·비율·pause 고정·도착·거부 검사는 보존했다. 서버/팩/속도 규칙 변경 없음.
- 긴 남은 시간 버튼이 다른 사단 클릭을 가로막아 마커의 보이는 텍스트는 사단 id로 줄였다. R9의 서버 남은 시간은 ko/en aria-label과 title에 그대로 보존하며 SSR 회귀시험을 추가했다.
- 첫 수정 `a09f96dadf00690dfacb1a03f98ef13fc17f780d`: 로컬 typecheck 및 대상 unit 17 PASS. hosted CI `39089868162`의 server 3OS PASS, save `39089868176`의 3OS/compare PASS. M1 Chromium/WebKit 97/98 PASS: 원 실패 10개는 해소됐으나 WebKit 관전자 시험에서 패널 열기 클릭 미반영(군사 조회 0건)이 발생했다. 관전자 시험의 패널 열기를 키보드 활성화로 바꾸고 military ready 확인을 추가했다. 실패 원문/trace를 보존했다.
- 최종 exact-head hosted 결과는 `/srv/hermes-workspace/projects/openhoi/results/t_a87b80c5-RESULT.json`, 상세 로그는 `/srv/hermes-workspace/projects/openhoi/evidence/t_a87b80c5/`를 따른다. 이 이력의 앞선 커밋 결과를 최종 HEAD 결과로 대신하지 않는다.
- 실제 세계 지도 M-1/AC10, Windows PC/GUI, 로컬 Rust 빌드/서버/브라우저, GCP, 배포/초기화는 NOT_RUN. 독립 리뷰·제품 인수·병합은 다음 담당 게이트다.
