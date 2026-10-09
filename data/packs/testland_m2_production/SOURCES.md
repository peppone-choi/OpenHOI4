# 합성 생산 팩 출처와 잠정 수치

작성: 최병호 · 2026-10-09

기준은 OpenHOI4 main `ef6d1fa1fcd9b7cd6350649b25efd11995d2e7ad`의 `data/packs/testland_m2`와 `crates/oh_data/tests/fixtures/production/{valid.toml,defines.toml}`이다. 외부 게임·위키·역사 수치·이미지 서비스를 사용하지 않았다. 기존 자작 입력과 새 정의/현지화의 저작자는 OpenHOI contributors이며 CC-BY-SA-4.0과 출처·변경 표기를 유지한다. 프로젝트 코드 라이선스를 변경하는 기록이 아니다.

| 입력 | 출처·변경 | 상태·단위 |
|---|---|---|
| 지도 PNG/CSV, 상태/프로빈스/국가/소유/경제/시간 설정 | 기존 `testland_m2` 자체 완결 복사. 지도/경제 수치 그대로, 시나리오 이름과 생산 선택 추가 | 자작6국/120프로빈스/12주, 잠정 |
| model/family/name ID | fixture의 `test_model_1/2`, `test_family`를 `m2_equipment_1/2`, `m2_equipment`와 새 ko/en 키로 변경 | 순수 콘텐츠 식별자; 전투 효과 없음 |
| generation, unit_cost, resources_per_item | fixture의 generation1/2, cost1/2, steel1/2 그대로 | IC-days/item, steel units/item, 잠정 |
| allowed_models/stock |6국에 명시한 허용 목록(README), 모든 허용 모델 재고0. fixture의 양수 참조와0 재고 계약 확장 | 허용성 음성 대조를 위한 합성 콘텐츠; 새 생산 규칙 아님 |
| initial/cap efficiency | fixture의0.25/1 | Fx, 잠정 |
| daily_efficiency_growth | fixture의0.0078125 | 실제 양수 생산일당1/128; 초기부터cap까지96생산일, 반/두배의 민감도192/48일 |
| same_family_newer_retention/other_retention | fixture의0.75/0.25 | 기존 모델 전환 정책 계수, 잠정 |
| stability_output_low/high | fixture의0.5/1.5 | stability0.5에서 multiplier1; 양 끝은0.5/1.5, 잠정 |
| inventory_count_limit/lines_max | fixture의9223372036854775807/1024 | 정수 연산/메모리 기술 한도; 창고나 훈련 정책 아님 |
| ko/en pack/장비 이름 | 새 자작 합성 설명, 기존 namespace 키 복사 보존 | 영구 세계관·역사 주장 없음 |

지도/기존 경제의 더 이른 자작 계보와 생성법은 기준 commit의 `data/packs/testland_m2/{SOURCES.md,README.md}`를 따른다. 이 팩에는 상속이나 원작 파일을 연결하지 않는다. 제조 수치의 반올림·자원flow·효율/분수carry·overflow·V6 계약은 `docs/adr/ADR-1501-production-contract.md`를 사용하며 이를 변경하지 않는다. 군사훈련과 O1 상한 정책은 입력도 수정도 포함하지 않는다.
