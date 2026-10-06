# WP-11 착수 전 저장 경계 점검

2026-10-07. M1-r1 오케스트레이터가 WP-09 통합 소스에서 확인한 다음 RUN 준비 자료다. 저장 구현·스키마 확정·검증 PASS가 아니다. WP-11은 착수 전에 [스키마 기획](schema-planning.md)에 따라 헤더/본문의 필드별 계약과 버전·복원 API를 설계하고 ADR에 남긴다. 근거는 02 §7, REQ-SAV-01/02, AC-M1-04다.

## 현재 권위 상태와 빠뜨리기 쉬운 입력

| 현재 소스 경계 | 의미·표현 | 복원 시 대조할 사항 |
|---|---|---|
| Simulation.state | scenario string, seed/tick u64, Date, hour u8, paused bool, speed u8 | 시나리오 비어 있음 거부, 유효 달력·hour 0..23·speed 1..5. 날짜·tick의 원래 관계와 저장 시점 보존 |
| Simulation.config | speed_ms_per_tick [u64;5], initial_speed u8 | host pacing도 현재 canonical hash 입력이다. 로드 때 현재 defines로 조용히 대체하지 않음 |
| Simulation.queue | BTreeMap<(tick u64, NationId u16, sequence u64), Command> | 정렬과 모든 예약 명령 보존. 과거 tick·중복키·잘못된 명령 인수 검증, 단순 deserialize 덮어쓰기 금지 |
| World.definitions_hash | u64 불변 정의 identity | 헤더의 팩 경로/바이트 content_hash와 서로 다른 의미. 재로드한 Defs의 identity와 대조 |
| World.inputs.nations | NationId, tag, government, BTreeMap<string,Fx> support | ID 정렬·중복/미존재·정의 대응, Fx raw bits 보존 |
| World.inputs.states | StateId, owner NationId, population i64, resources/buildings BTreeMap<string,i64>, base Fx, modifiers Vec, infrastructure Fx, ledger | 인구는 가용 인력과 다르다. 입력/적용값/원장 tick·순서·최종 raw bits 일치, 모든 주를 검증한 뒤 공개 |
| World.inputs.provinces | ProvinceId, state/owner/controller Option<ID> | null과 ID0 구분. 육지와 수역 정책·주 참조·소유와 통제를 각각 검증. ID를 Vec 위치로 해석하지 않음 |
| Modifier | source/target_stat string, Add 또는 Mul, value Fx, expires Option<u64> | expires는 첫 제외 tick. 만료된 항목도 입력에서 임의 삭제하지 않음. active 중복·overflow·실제 적용값 불일치 거부 |
| World.defs | Arc<Defs>, Serialize skip | 지도·국가·팔레트는 저장 본문에서 제외하고 검증된 팩에서 재구성 |

현재 Simulation에는 지속되는 RNG 객체나 draw cursor가 없다. oh_core의 RngKey는 seed/system/day/entity로 스트림 시작을 재구성하며, Simulation은 seed·날짜·tick을 보존한다. 이 현재 구조에서 존재하지 않는 RNG cursor를 저장했다고 주장하지 않는다. 이후 시스템이 틱 사이에 진행 중인 스트림/작업을 유지한다면 그 실제 상태와 재개 경계를 별도 설계해야 한다. 시작 seed만으로 임의 중간 스트림 위치가 복원되는 것은 아니다.

State·WorldInputs·World·Simulation에는 외부 임의 Deserialize 복원 경계가 아직 없다. oh_save는 자리 모듈이다. Serialize 가능한 객체를 그대로 읽을 수 있다고 가정하거나 private 상태 검증을 해제하지 않는다. save DTO→전체 유효성 검사→파생 재계산/일치 확인→원자적 live 교체 순서를 구현 세션이 구체화한다. 원장의 재생성 여부는 현재 canonical 직렬화와 같은 hash를 보존하는지 명시한다.

## 포맷과 실패 사례를 먼저 설계할 것

02 §7.1의 OHSV, format_version u16, header_len u32, postcard 헤더, zstd(postcard 본문)를 기준으로 엔진/포맷 버전·시나리오·팩 목록/버전/content_hash·날짜/tick/seed/state_hash·player_nations·표시용 saved_at_utc를 필드별 정의한다. 바이트 순서, 길이/압축 해제 상한, trailing bytes, 헤더/본문 중복값 불일치, 파일 원자적 교체와 실패 시 이전 파일 보존을 ADR에서 정한다. 새 format의 이전/미래 버전은 명시 거부 fixture로 시작하며, 실제 구버전이 생기면 02 §7.2의 변환 또는 명확한 거부와 독립 fixture를 추가한다.

팩 불일치 기본 거부와 명시 --force는 규격대로 구분한다. --force로 손상·지원하지 않는 포맷·잘못된 참조·길이 제한을 우회하지 않는다. identity를 강제 수용했을 때 정의 참조와 원장 재계산/hash 처리도 모호하게 남기지 않는다. saved_at_utc와 I/O는 host/save 계층이며 sim canonical hash에 넣지 않는다.

| 독립 fixture | 명시 기대 결과 |
|---|---|
| M1 연속 N+M일 vs N일 저장→새 process 로드→M일 | 실제 국가·주·owner/controller·원장입력·큐 포함 hash 동일. M0 시간만의 hash로 대체하지 않음 |
| 정지·5속도·동일 tick 다국가/순번 명령·향후 예약 | snapshot/config/ordered queue와 이어 실행 결과 동일 |
| Fx 최하위 bit·만료 직전/동일/직후 tick·만료 항목 잔존 | 입력 raw bits와 원장/적용값·hash를 각각 대조 |
| ID0/65535·비연속 국가/주·물 null·참조 불일치 | 유효 안정 ID 보존, 관계 오류는 live 세계/큐 변경 없이 거부 |
| magic/version/길이/truncation/zstd 오류·압축 팽창·trailing bytes | 정해진 오류와 상한, 기존 파일·세계·큐 보존 |
| 헤더 날짜/seed/hash·팩 identity·body 불일치 | 중복 필드 정책에 따른 명시 거부, 숨은 기본값 금지 |
| 세 OS 저장/재개 | 동일 fixture bytes/SHA와 실행 명령·정확 HEAD·재개 hash 비교 |

이 표는 검사 목록이며 실행 결과가 아니다. 새로운 mutable 필드 추가 때 저장/해시/wire/조회 연결표를 갱신하고 누락을 독립 검증에서 확인한다. 자동저장·브라우저 파일 업로드/다운로드와 이후 마이그레이션 작업은 WP-37, 재현 번들은 WP-25다. M1 최소 저장 연결의 실제 CLI/server API와 사용자 동선은 WP-11 설계에서 명시한다. 아직 없는 경제·인력·전투·외교 필드나 저장 UI를 구현된 것으로 기록하지 않는다.
