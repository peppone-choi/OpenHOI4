# 스키마 조사 출처와 열람 범위

확인일 2026-10-06. 사용자 요청에 따라 시스템/필드 분류 참고와 기술 스키마 의미를 조사했다. 원작의 규칙·수치·코드를 채택한 기록이 아니다.

| 공개 위키 대상 | 이번 직접 열람 결과 | 사용 범위 |
|---|---|---|
| [Countries](https://hoi4.paradoxwikis.com/Countries) | 웹 도구 401 | 본문 미확인·미사용 |
| [State](https://hoi4.paradoxwikis.com/State) | 웹 도구 401 | 본문 미확인·미사용 |
| [Politics](https://hoi4.paradoxwikis.com/Politics) | 도구 접근 실패 | 본문 미확인·미사용 |
| [Production](https://hoi4.paradoxwikis.com/Production) | 웹 도구 401 | 본문 미확인·미사용 |
| [Logistics](https://hoi4.paradoxwikis.com/Logistics) | 웹 도구 401 | 본문 미확인·미사용 |

위 공식 경로를 대상으로 분류 검색도 시도했으나 결과가 없었다. 접근하지 못한 내용을 읽었다고 기록하지 않는다. 복제된 원작 데이터/코드 저장소·수치표를 대체 조사 자료로 사용하지 않았다. 권한·네트워크 설정을 바꾸지 않으며 실제 접근 가능한 공개 경로가 확인되면 일시·범위·분류 참고 내용을 별도로 기록한다. 이 제한 때문에 현재 01/02에 있는 자체 규칙과 기술 설계를 중단하지 않는다.

2026-10-07 추가 정상 공개 경로 점검: [Country](https://hoi4.paradoxwikis.com/Country)와 [States](https://hoi4.paradoxwikis.com/States)도 웹 도구에서 접근 불가였다. 이 별칭 경로의 실제 페이지 존재나 본문을 확인한 것으로 취급하지 않는다. 인증·차단 우회 없이 현재 자체 문서와 열람한 기술 자료로 WP-08 계약 및 WP-11 착수 경계를 검토했다.

기술 문서는 정상 열람했다. [JSON Schema annotations](https://json-schema.org/understanding-json-schema/reference/annotations)는 default가 검증 중 누락값을 채우는 기능이 아니며 readOnly도 annotation임을 설명한다. 적용값과 변경 권한은 실제 로더/명령 경계에서 검사한다.

[JSON Schema object](https://json-schema.org/understanding-json-schema/reference/object)와 [null](https://json-schema.org/understanding-json-schema/reference/null)은 필수 속성, 추가 속성, 누락과 null을 구분한다. 필드별 required/null/default 정책을 명시하고 Rust·생성 TS·runtime 동작과 대조한다. 객체 타입 선언만으로 모든 키·관계가 검증된다고 주장하지 않는다.

[Schemars 1.2.2 공식 문서](https://docs.rs/schemars/1.2.2/schemars/)는 Rust 타입에서 JSON schema를 생성하는 기술 근거다. 생성 구조와 serde 표현을 확인하되 게임상의 참조·권위·갱신·해시/저장 연결은 자체 설계와 독립 검사로 대조한다. 새로운 외부 의존성을 도입한 조사는 아니다.

계획 문서의 구체 게임 필드는 01의 해당 REQ/규칙·02 해당 WP를 근거로 한다. 공개 위키는 향후 정상 열람 시 개념 분류/누락 조사에만 참고하며 문장·원작 수치표·파일·스크립트 금지와 지도 분할 원칙만의 한정 예외를 유지한다.
