# M1 새 독립 게이트 준비 목록

2026-10-07, M1-r2 오케스트레이터 준비 자료다. 게이트 판정이 아니다. 최신 WP-11 통합·같은 main HEAD CI가 충족된 뒤 정확한 HEAD의 새 detached worktree와 새 독립 앱에 P-12를 배정한다. 과거 PASS와 최신 CI 실패는 별도로 확인한다.

| 필수 REQ | 담당 증거 | 게이트에서 직접 확인할 범위 |
|---|---|---|
| REQ-TIME-01 | WP-04/05·WP-09/08 연결 | 실제 일시정지·속도1~5·시간 단위, 저장 복원 config/paused/speed와 UI |
| REQ-TIME-02 | WP-04·실제 M1 hash/phase | 틱/일/월 고정 순서·국가/주/큐·저장 재개, M0 hash로 대신하지 않음 |
| REQ-MAP-01 | WP-07 | 비트맵·정의 ID/육지/바다/호수/지형 |
| REQ-MAP-02 | WP-07 | 생성 인접·명시 덮어쓰기·골든 보존 |
| REQ-MAP-03 | WP-07/09 | 육지의 정확히 한 주·인구/자원/건물/인프라·유효 참조 |
| REQ-MAP-04 | WP-08 | 실제 GPU 정치/지형/주 모드·권위 서버 조회 |
| REQ-MAP-05 | WP-08 | 국가/주/프로빈스 국경·선택/호버·카메라 |
| REQ-NAT-01 | WP-09 | 국가·주 mutable 상태/조회·원장 rawbits·패널, 전체 후속 모델과 구분 |
| REQ-SAV-01 | WP-11 신규 독립 전문 예정 | OHSV 헤더·버전/팩/hash·완전한 본문·전체 유효 import·파일 교체 원자성 |
| REQ-SAV-02 | WP-11 신규 독립 전문 예정 | 실제 M1 연속 실행 대 native 새 process 저장/로드/재개·예약큐/config/expiry·세OS |
| REQ-PLAT-03 | WP-08 | 기존 full browser 범위·실제 양backend·resize/DPR/native cleanup·최신 CI timeout 해결 증거 |
| REQ-NET-02 | WP-05/08 | 실제 단일 서버 실행 파일의 정적HTTP/WS 지도·브라우저 접속 |
| REQ-LOC-01 | WP-12 | ko/en 전환·문자열 키·Fluent runtime |
| REQ-LOC-02 | WP-12 | 자체 CJK 폰트·실제 표시·출처·ASSETS |

AC-M1-01 지도데이터/인접, 02 서버/지도/국경/선택, 03 패널/원장, 04 저장/재개, 05 현지화/폰트, 06 시간/순서, 07 위14 필수 REQ·독립 PASS·기본브랜치 통합을 빠짐없이 판정한다. 관련 [현재 계획](M1-r2.md), [이전 독립 지도 전문](../verify/WP-08.attempt6.md), [최신941 실패 원본](../verify/evidence/WP-08-main941-CI-FAIL/README.md), [직접 UI 탐색](../play/M1-r1.md)을 확인한다. 실패 원본을 과거 PASS로 덮지 않는다.

## 저장 독립 예제 초안

이 절은 기존 02 §6.3 Fx 계산과 scheduler 계약의 검사 입력이다. 게임 효과·수치를 추가하지 않으며, 실행 결과나 구현 PASS로 취급하지 않는다. WP-11 실제 설계/인터페이스에 맞춰 새 검증자가 별도 기준 계산과 입력을 만들게 한다.

`Q=2^32=4294967296`, base rawbits `2Q+1`, Add rawbits `Q`(expires=24), Mul rawbits `3Q/2`(만료 없음)를 쓴다. Add→Mul과 매 단계 floor로 tick23 최종 rawbits는 `19327352833`, tick24/25는 `12884901889`다. 전체 modifier 입력에는 만료 Add가 남아 있어야 하며 원장에는 tick24부터 빠져야 한다. 같은 대상/op/source의 active 중복·overflow·원장 bit 위조는 전체 상태/큐 교체 없이 거부되는지 검사한다.

동일 tick의 `(NationId, sequence)` 정렬은 nation 우선이다. 낮은 nation의 seq9 SetSpeed2와 높은 nation의 seq1 SetSpeed5를 역순 삽입해도 최종 speed5가 되어야 한다. 현재 `enqueue`는 잘못된 SetSpeed도 큐에 넣고 실행 때 거부하므로, WP-11은 실제 live 큐의 export·restore 정책과 오류를 명확히 설계해야 한다. 무효 명령을 조용히 제거하거나 hash를 바꾸지 않는다. M0 world=None과 M1 nation 참조 경계를 구분한다.

tick23 Pause(true) 적용 뒤 tick23·date/hour가 유지되고 향후 tick24 명령은 남는다. 현재tick Unpause(false)를 처리한 다음 tick24·만료 경계의 원장/큐/hash가 연속 실행과 같아야 한다. 원장 평가는 정지 상태에서도 saved tick에서 수행된다. 실제 동일 저장 직후 hash, 이어 실행 hash, 국가/주/프로빈스/base/Fx/ledger/config/큐 구조를 각각 대조한다.

포맷0/미래버전·길이/truncation·trailing·zstd손상/팽창·헤더/본문 불일치·pack/defs mismatch·비연속 ID/ID0/물null·중복/미존재 참조를 검사한다. `--force`는 팩 identity 정책의 명시 허용만 적용하고 손상/잘못된 참조를 우회하지 않는다. 원자적 저장 실패 전후 기존 파일SHA와 live 상태/큐/hash를 확인한다. 동일 fixture bytes/SHA·정확 HEAD·각 OS 명령·저장 직후/재개 hash를 남긴다. M3 브라우저 업다운로드·자동저장과 현재없는 RNGcursor를 구현 증거로 쓰지 않는다.
