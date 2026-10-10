# 최소 군사 팩의 출처와 잠정 수치

작성일: 2026-10-10. 구현/데이터 조합/문서 작성은 Codex의 AI 보조로 수행했다. 독립 검토는 별도 절차이며 이 문서는 인간 배포 승인을 만들어내지 않는다.

기준 main: `0d50a83f862bf575e8051a1c7cf12da7f0ea0428`. `data/packs/testland_m2_production`의 자작 합성 입력31개를 복사하고, 군사 입력1개를 추가한32파일 팩이다. 더 이른 생산/지도 출처와 생성법은 해당 commit의 `testland_m2_production/SOURCES.md`, `testland_m2/SOURCES.md`를 따른다. 저작자 표기는 OpenHOI contributors, 데이터 라이선스는 CC-BY-SA-4.0을 유지한다. 외부 역사 자료·위키 문장·원작 자료·이미지 서비스·새 이미지 생성은 사용하지 않았다.

| 입력 | 출처/변경 | 상태·단위 |
|---|---|---|
| PNG/CSV·주/프로빈스·경제·국가·소유·시간 | 기준 생산 팩의 바이트 복사. `scenarios/m2_production` → `m2_military` 경로만 변경 | 자작6국/120프로빈스/12주, 잠정 |
| manifest/scenario | 팩 ID/이름키/시나리오 이름 변경, 기존 economy/production에 military initial 선택 추가 | 콘텐츠 식별자 |
| 생산 모델2개·가족·생산 defines9개 | m2_equipment_1/2, m2_equipment 그대로. N03/N06의 allowlist/0재고만 model1 확장 | 기존 생산 계약 유지, 잠정 |
| 템플릿/컴포넌트 | accepted `crates/oh_cli/tests/fixtures/military/training.toml` small의 인력8·장비6·훈련2일. ID를 m2_small/m2_small_component, family/model을 기존 생산 ID로 변경 | 사람/item/day, 잠정 합성 수용 입력 |
| 11개 normal stats | 같은 small의 strength/soft_fire/defense/breakthrough/frontage/supply_use/organization/speed_kmh 1, hard_fire/armor/piercing 0 | 기존 fixture 값/단위, 전쟁 밸런스 아님 |
| 군대/배경 | 새로운 합성군대6개, 지휘관 synthetic_n01..06, capacity2/priority0. 전 국가 committed/reserved0, 초기 사단0/작업0 | 기존 런타임 계약을 위한 최소 입력 |
| ko/en 문자열 | 새 팩 이름은 pack.ftl. production.ftl의 unused o2-pack-name 한 줄만 제거; 두 장비 문자열 동일 | strict0warnings를 위한 최소 보정 |

`assets/ASSETS.toml`에 복사31개 modified=true, 새 군사 입력 modified=false로32레코드를 추가했다. 원래 레코드 바이트는 변경하지 않았다. `ai_generated=false`는 OPEN-09(01의 D-11/OPEN-09, 02 §14.5)와 `tools/check_assets.py`가 다루는 **AI 생성 이미지** 분류에 따른다. 지도 PNG는 기준의 같은 바이트이며 새 이미지 생성 도구를 호출하지 않았다. 이 필드를 AI가 코드/TOML/문서 조합에 관여하지 않았다는 의미로 사용하지 않는다. AI 보조 사실은 각 새 레코드 notes와 이 문서에 명시했다. AI 생성 이미지의 승인/약관 레코드를 위조하지 않는다.

기존 군사/생산/세이브/프로토콜 정책과 로더 오류를 변경하지 않는다. 공통 모델 허용은 모든 국가에 템플릿 바인딩을 검증하는 기존 로더를 만족하기 위한 데이터이며 새 나라별 템플릿 정책이 아니다. 날짜/국가명/지휘관/장비명은 합성 식별자이며 역사 또는 현실 군사 주장과 무관하다.
