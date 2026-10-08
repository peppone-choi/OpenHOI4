# 공개 인계 기술 메모

이 메모는 공개 WIP의 기술 경계다. 내부 결정 요청·실행 계약·검증 보고서의 사본이 아니며, 새 게임 규칙이나 완료 판정을 추가하지 않는다. 규칙과 식별자는 01 기획서, 기술 구조·DR·WP와 추적 매트릭스는 02 제작용 문서, 작업 절차 P-01~P-12는 03 제작용 프롬프트에 유지한다.

- WP-14 경제·정치: `crates/oh_data/schema/economy.schema.json`, `crates/oh_data/src/economy.rs`가 정의·초기값을, `crates/oh_sim/src/economy.rs`가 가변 상태·명령·날짜 경계 갱신을 담당한다. 정수/Fx와 checked 연산, phase 실패 시 전체 상태·큐·시계·RNG rollback을 유지한다. save v5·canonical·조회 DTO와 클라이언트 shape 검사가 연결되지만 최종 통합 판정은 미완료다.
- WP-13 종료·flags와 REQUEST-0009: inclusive end_date, 명시 root가 있는 단일 AST, 초기 1회와 성공 시간전진 뒤 평가, typed 종료 원인·효과 transaction, 종료 후 조회·저장은 유지하고 step/enqueue를 거부한다. 기존 v1~v4 저장과 미래 큐 보존을 지킨다. 결과 UI·최종 점수는 후속 작업이다.
- WP-17 이동과 REQUEST-0006/0007: 진행 구간의 Stop/reroute 및 trusted 양수 방향별 Strait 계수를 다룬다. 공개 unit 생성·동적 접근·보급·전투·해상 수송 규칙은 후속이며 내부 모델 존재를 UI 지원으로 확대하지 않는다.
- WP-24와 REQUEST-0008: active 팩은 엄격히 검증한다. 역사적 M0/mutable-v1 입력의 호환 진단은 정확한 입력 identity·caller·bounded header/context일 때만 한정하며 blanket legacy 면제가 아니다.
- WP-25: 재현·성능 도구와 입력/메타데이터 검사는 `bench/README.md`, `bench/run.py`, `bench/compare.py` 및 `crates/oh_cli/tests/repro_native.py`를 참조한다. 고정 과거 성능 workload는 현재 M2 콘텐츠/UI 성능을 대신하지 않는다.
- REQUEST-0010 경제 잠정과 차량 REQUEST-0002 관련 후속안은 사용자 확정으로 승격하지 않는다. 01 §4.3·§4.9, 02 §5.11·§6.2의 범위·처리 순서를 따른다. 스키마 설계는 필드 의미·단위·범위·참조·수명·권위·실패 원자성·hash/save/wire/UI 대응과 독립 경계 예제를 명시한다.

현재 지원 범위와 중단·실패·미실행 검사는 [인수인계](../HANDOFF.md)에 정리한다. 이 tree에는 이전 내부 실행 증거를 포함하지 않는다. 기존 공개 조상 commit의 증거 이력은 이번 정상 새 commit으로 소거되지 않는다.
