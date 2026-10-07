# WP-14 M2-r2 배정·producer 인수 계약

2026-10-08. W1-c 검증 중 작성한 착수 전 계약이다. 아직 WP-14 구현 앱·worktree·base를 배정하지 않았다. W1 전체의 WP-25 유효 독립 PASS·P07·같은 main 필수 CI를 인수한 뒤 exact base와 최초 snapshot을 고정한다.

## 규칙과 단일 파일 소유

01 §4.2·§4.3 및 [REQUEST-0010 A-CEO-r1](../decisions/REQUEST-0010.md)의 **수정판**이 게임 의미의 기준이다. 원후보 PC 선지급 순서는 채택하지 않았다. 사용자 확정·D/OPEN 변경으로 기록하지 않는다. 새 규칙·누락 후속 공식은 담당 schema에서 발명하지 않고 부모에 구체 입력/결과 후보를 보고한다.

| 소유자 | 파일·책임 |
|---|---|
| WP-14 | 새 oh_data 경제·정치 정의 모듈과 lib/national/pack_validation의 명시 연결, 새 정의 tests/schema 출력 |
| WP-14 | 새 oh_sim 경제·정치/수량 원장/저장 상태 모듈 및 lib/world/formula/trigger/trigger_save/save_state의 필요한 연결, 전용 tests/fixtures |
| WP-14 | oh_save의 새 additive 경제 저장 codec/context·명시 bounds 및 lib/file/context/repro의 새 형식 연결, 새 호환·repro tests/fixture exporter |
| WP-14 | oh_proto 타입·생성 출력과 oh_server session/lib의 실제 권위 command/query/restore 연결 및 native tests, 필요한 CLI 새 경제 시나리오/record/replay 입구 연결 |
| WP-14 | client는 필요한 명령·snapshot runtime/type guard·generated proto와 대응 tests만; 새 경제 화면/지도/HUD/3D는 후속 UI WP 소유 |
| WP-14 | docs/plans/WP-14-schema.md, WP-14 ADR, docs/worklog/WP-14.md, 차별화 초안 |
| WP-23 | production testland의 신규 m2 시나리오/전용 지도/국가·현지화·콘텐츠·SOURCES, 신규 에셋 등록 **초안** |
| 부모 | 01/02/03/HANDOFF·M2 공유 계획·추적표·ASSETS master; 공통 manifest/lock·CI/bench 정책은 별도 인계 전 수정 금지 |

기존 M0/M1 팩·fixture·expected·골든·None/v1~v4 codec의 원 byte 계약, 7개 CI와 성능 prior/한도는 보존한다. 필요한 기존 파일 변경은 위 책임의 새 경제 연결에 한정한다. 파일 단위 판단이 더 필요한 변경은 부모에 최소 경로/이유를 먼저 보고한다. WP-23 파일을 WP-14가 임시로 고치지 않고 자기 독립 fixtures를 쓴다. 둘은 다른 작업의 변경을 되돌리지 않는다.

## 구현 전 필드·참조 표

아래는 실제 스키마를 만들기 전 체크 항목이다. 필드명·새 저장 version·최대값을 이미 구현됐다고 주장하지 않는다. 구현자는 WP-14-schema에 실제 타입/단위/범위/누락·null/default/수명·권위·참조·load/command/phase/restore/wire 경계를 먼저 고정한다.

| 계층 | 필수 인수 항목 | 단위·경계 |
|---|---|---|
| 불변 정의 | 공업 시설 ID·IC 단가·보정, 건물 단계 비용/cap/슬롯, 법령 범주/ID/정렬 단계·PC 비용·AST 조건·비율/하한 계수 | 비율·계수 Fx(I32F32), 누적량 Qty(I48F16), ID/조건/locale 참조 검증; 명시 정의 없음과 빈 정의 구분 |
| 초기 입력 | 국가 선택 법령·PC/안정도/동원도·네 비율·committed/reserved, 주 인구/건물 수준·슬롯과 불변 정의 연결 | 새 경제 존재 시 필요한 값은 명시; 원 시나리오 부재를 자동0 신규 상태로 바꾸지 않기 |
| 가변 권위 | 현재 법령/PC/비율/정치 지표, 건설 queue·현재수준+1 목표·누적 진행·예약·휴면 사유, 인력 committed/reserved | private 검증 입구·정렬·중복/타국 참조 거부·checked overflow·multi-effect 원자성 |
| 파생/원장 | 계산 tick/소유 주별 기여·IC/자원/인구, 소비재 하한·네 실제 IC·raw 잔여, 건설 원IC 소비/초과·남음, capacity/가용/초과예약 | 실적 snapshot과 다음 일일 산출 구분; Qty를 Fx로 조용히 좁히지 않고 원장 최종 bits=적용 bits |
| 명령/phase | 비율 변경·한 단계 건설/취소/순서 변경·법령 변경, 실제 지원 정치 효과·조건/산업 점수 adapter | 기존 Commands/일일 Phase 순서 유지·같은 틱 PC 선지급 금지·예정 의미 오류 소비/phase 전체 rollback 구분 |
| 호환/소비 | 새 version/presence·정의 identity·full state/pending/future queue·dto/canonical/hash/save/repro/wire/query | 원None/v1~v4 read/write/hash 보존·fresh native 재생·잘못된 mode/force/미지원 조건값0 거부 |

실제 producer가 없는 생산 효율·항복·군사 효과·훈련/보충·자원 부족 생산·시장 거래는 명시 후속 인수다. 관련 REQ 전체를 WP-14 producer 단계만으로 완료하지 않는다. 내부 인력 API를 임의 플레이어 인력 생성/반환 명령으로 노출하지 않는다.

## 독립 검증과 WP-23 전달

REQ별 RED→구현→통과 원명령/native 출력, 법령/하한 transaction·cap0/보정0 skip·단계 중복/슬롯·소유권 상실/회복/목표충돌·원IC 올림/초과폐기·인력 과예약/감소·정치 날짜 경계/PC 상한·wholephase rollback을 실제 입구와 전체 상태로 검사한다. source exact 새 독립 앱은 자체 입력·수량 산술로 검사하고 구현자 설명을 판정 근거로 쓰지 않는다.

WP-14가 먼저 실제 schema·지원 파일 경로/필드·ID/조건 범주·locale 요구·loader/validate·새 저장/호스트 계약을 커밋 단위로 제공한다. WP-23는 그 전에 기존 map/nation/locale 계약으로 신규 기하·6개국만 준비할 수 있으며 경제 입력을 추측해 성공 로드하지 않는다. producer 통합 뒤 최신 기준을 콘텐츠 브랜치에 반영하고 새 m2 경제 입력과 실제 strict validate·native load/query/save/repro를 검사한다. 원 m1 파일 bytes와 frozen pack identity를 보존하고 팩 전체 hash 변경을 별도로 기록한다.

WP-14→WP-23 각 유효 독립 PASS·부모 P07·같은 main 7개 workflow/전 필수 job·실제 3OS 저장/trigger 및 고정 prior Linux 성능을 인수한 뒤 W2 통합을 기록한다. 평시→전쟁→항복·전체 M2 게이트는 후속 WP들의 실제 증거가 필요하다.

## 고정 성능 입력의 후속 콘텐츠 경계

WP-23의 새m2 파일은 같은 testland 전체 content_hash를 바꾼다. WP-14에 bench/run.py·관련독립입력/metadata검사·README·기술ADR 소유를 추가 인계한다. baseline81deb와240000/warm2400/m1/seed1000/two pairs15%/native240s/build1200s를 유지하고, 양쪽 별도 실제라이브러리에 **고정prior81deb에서 추출한 동일immutablepack**을 입력한다. 새 current source는 현재 실제라이브러리이며 입력만 고정한다. 원driver/profile/lock/registry·입력provenance/list/bytes/SHA와완료fullDTO/canonical/hash를대조하고검사삭제/가짜packhash/기준교체로통과시키지않는다. 신규콘텐츠는자체strictvalidate/native/save/repro로검증한다. 현재WP25코드는변경하지않고다음담당이실제입력분리schema/ADR·회귀부터작성한다. 부모가defaultmain에존재하는performanceworkflow의exactsourceRef를한번dispatch해실Linux를검사하며구현자는CI브랜치조건/설정을바꾸지않는다.

### CEO 추가 기술 인수 조건

고정입력은prior팩**전체**이며m1파일만선택하거나current팩을편집해prior처럼만들지않는다. 동일path공유또는전복사의정렬path/byte길이/SHA/실제packhash exact·extra/missing/손상/symlink/reparse/참조이탈거부·load/warmup/측정전후immutableidentity를검사한다. 실제current/baseline pathdependency/lock/registry와binarylinkage/provenance를보존하고뒤바뀜회귀를거부한다. 고정m1벤치는신규경제활성부하·후속M2/M3/UI성능증거를대신하지않는다. 기존workflow_dispatch에는source_ref입력필드가없다. 부모가후보branch/ref의직전SHA를확인해기존workflow --ref로한번dispatch하고actualAPI/job/env/artifact exactSHA를다시검사한다. 설정/브랜치조건/한도변경·미실행PASS·묵시적반복은없다. 원CEOreview는아래새evidence에원bytes로보존했다.
