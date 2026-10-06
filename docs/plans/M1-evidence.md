# M1 증거 연결표

2026-10-07, M1-r1 WP-08 수정 독립 PASS·main 통합 후. 게이트 판정이 아니라 다음 독립 게이트가 실제 증거를 찾기 위한 연결표다. W1 main4ed5e57·WP09 mainb1da8f4의 CI 세 종류 success와 실제 M1 세 OS 해시를 확인했다. WP08의 정확한87c1a99는 새 독립 PASS·부모2458파일 불변 확인 후 main9dcf199에 통합했고 P07 로컬16검사를 통과했다. 통합 후 정확한main HEAD CI는 별도 확인한다.

| AC | 현재 증거 | 아직 필요한 연결 |
|---|---|---|
| AC-M1-01 | WP-07 717b78f 독립 PASS, 94개 지도 입력·인접/주/ID/거리 검사. main 통합됨 | 후속 팩 변경이 있으면 최종 게이트에서 다시 대조 |
| AC-M1-02 | WP08 87c1a99 새 독립 PASS: 실제 WebGPU/WebGL2 픽셀·세 국경·선택/호버·팬줌·단일 서버/HTTP·165브라우저 회귀. 첫f58 metadata/index redirect FAIL과 새차단 증거 보존 | 최종main 동일HEAD CI·새게이트에서 최신팩/화면 대조 |
| AC-M1-03 | WP10 계산·WP12 표시·WP09 실제 Query·WP08 지도 선택과 국가/주/원장1/0 연결 독립 PASS, main 통합 로컬 검사 통과 | 저장 후 조회까지의 연결은 WP11 이후 새게이트에서 대조 |
| AC-M1-04 | 없음. 저장 없이 M0 시간 hash 2회는 이 AC 증거가 아님 | WP-11 실제 M1 상태 저장→재개와 연속 실행 hash 일치, 신규 국가/주·원장입력·큐 포함 |
| AC-M1-05 | WP12 ko/en·Fluent·자체 CJK PASS, WP09/08 실제 국가/주/지형 키·오류 안내와 언어 전환 회귀 PASS | 저장 UI/후속문자열 추가 때 최신게이트 회귀 |
| AC-M1-06 | WP04 고정 처리순서, WP12/09/08 실제 시간5단계·정지 회귀 PASS. 직접 UI 플레이와 자동검사 별도 | WP11 재개 이후 동일 순서·조회 결과 대조 |
| AC-M1-07 | WP07/10/12/09/08 독립 PASS·통합됨. WP08 P07 로컬 통과 | WP11 독립 PASS·통합, 최종필수14 REQ/같은HEAD CI·새P12 |

| M1 필수 REQ | 증거 소유·현재 범위 |
|---|---|
| REQ-TIME-01, REQ-TIME-02 | WP-04 처리 순서 + 새 UI의 WP-09/08 회귀. host clock과 sim 시간 분리 |
| REQ-MAP-01, REQ-MAP-02, REQ-MAP-03 | WP-07 로드·인접·주. 최종 팩에서 불변/오류 경계 대조 |
| REQ-MAP-04, REQ-MAP-05 | WP-08 실제 GPU/대체 경로의 픽셀·국경·선택/호버·카메라 |
| REQ-NAT-01 | WP-09 국가/주 정의·상태·query·패널, WP-08 선택 연결 |
| REQ-SAV-01, REQ-SAV-02 | WP-11 v1 헤더·실제 M1 state·저장 재개 DT |
| REQ-PLAT-03, REQ-NET-02 | WP-08 실제 WebGPU 우선/WebGL2 대체와 단일 서버 지도 실행. 소프트웨어 렌더링/실제 장치를 구분 |
| REQ-LOC-01, REQ-LOC-02 | WP-12 ko/en·CJK + 새 국가/주/지도 문자열 회귀 |

서버 원장 최종값과 실제 적용값의 raw bit 일치, private state·immutable Defs 경계, MessagePack 형상 거부와 실패 시 화면/정상 상태 보존은 WP-09/08 연결 검증에 포함한다. 클라이언트의 숫자 문자열 표시를 서버 시뮬레이션 계산으로 대신하지 않는다.

직접 UI 플레이는 자동 E2E·헤드리스 AI와 별도 기록이다. 정확한 커밋·시나리오·시드·게임 날짜·브라우저/백엔드·행동 이유·기대/실제·캡처·결함을 남긴다. 구현된 UI 동선에서 선택/시간/언어/원장을 조작하고 원시 상태 변조를 플레이로 기록하지 않는다. 저장 UI가 없는 동안은 미실행 범위를 명시한다. M1 Testland는 합성 지도이며 실제 지리 생성은 후속 WP-32/45다.

M1-r1은 두 묶음 통합이 먼저 끝나면 WP-11과 게이트를 다음 RUN에 남긴다. 이 표에서 미완료가 사라졌다는 이유만으로 게이트 PASS를 선언하지 않는다. 새 독립 검증 채팅의 P-12 전문·같은 HEAD CI가 필요하다.

WP09 실제 M1 CI 추가 증거: b1da8f4의 3OS ZIP digest를 직접 검증하고 compare --m1 exit0, 세 OS hash60448355cecffa9d 일치를 확인했다. `docs/verify/evidence/WP-09/main-ci/`를 참조한다. WP08 GPU/HTTP 증거는 [새 독립 전문](../verify/WP-08.md), [최초 FAIL](../verify/WP-08.attempt1.md), [통합 로그](../verify/evidence/WP-08-integration/summary.json)에 있다. 저장 재개 DT는 WP11 후속이며 실제 M1 시간 hash로 대신하지 않는다.
