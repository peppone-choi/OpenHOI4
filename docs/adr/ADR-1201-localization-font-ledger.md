# ADR-1201 WP-12 Fluent 현지화·자체 호스팅 CJK 폰트·원장 표시 API

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-12, REQ-LOC-01, REQ-LOC-02, AC-M1-03, AC-M1-05 |

## 맥락
02 §9·§11·§12.2는 Fluent와 OFL CJK 자체 호스팅을 요구한다. M0 셸은 ko/en JSON과 시스템 폰트를 사용하며, 서버는 client/dist 전체를 빌드 시 임베드한다. M0의 연결·Notice·실패·시간 제어 계약과 JSON 제공 경로를 보존해야 한다. WP-10 원장을 WP-09가 프로토콜에 연결하기 전에 표시 전용 공용 API가 필요하다.

## 결정
`@fluent/bundle` 0.19.1을 정확한 버전으로 고정한다. ko/en `.ftl`을 Vite raw import로 번들에 넣고 React Localization context의 `t(key,args)`로 표시한다. 영어 fallback 후에도 없는 키나 인자가 잘못된 메시지는 현재 언어의 `unknown-message`를 표시한다. 문서 lang/title, 접근성 이름, 공백 표시, 서버 Notice와 실패도 키를 사용한다. M0 JSON 파일·ASSETS 항목은 보존하되 런타임 화면은 Fluent를 사용한다. 전체 팩의 누락/미사용 키 CI(REQ-LOC-03)는 WP-24 범위다.

Noto Sans KR variable TTF 원본 전체(10,414,588 bytes)를 `client/public/fonts/NotoSansKR.ttf`로 자체 호스팅한다. 파일 이름만 로컬 경로로 정하고 내부 이름·바이트는 바꾸지 않는다. font-weight 100~900, 본문 wght 400·강조 wght 700을 font-variation-settings로 명시(Windows WebKit의 두께 축 기본값 차이 대응), font-display swap, CSS 변수 테마와 공통 UI font-family를 사용한다. 원본 OFL.txt를 그대로 제공하고 셸에서 고지에 연결한다. 서버 build.rs에 font/ttf와 ftl/txt text/plain MIME만 추가한다. 런타임 파일시스템이나 외부 CDN은 필요 없다. 실행 파일이 CSS·FTL·TTF·OFL을 모두 제공한다.

GameShell, TimeControls, LedgerValue로 표시를 나눈다. App의 기존 connect/applyServerMessage/sequence/TimeCommand 흐름은 유지한다. `<details>/<summary>`로 포인터·키보드·터치에서 여는 원장 tooltip 기본형을 만든다. 포커스와 열림 상태는 브라우저의 기본 동작을 사용한다.

공용 API는 `LedgerValue({labelKey, ledger})`, `LedgerView = {base:string, final:string, tick?:string, entries:readonly {id:string,labelKey:string,operationKey:string,value:string,accumulated:string,sourceKey:string|null}[]}`다. 일반 표시 props이며 wire/protocol 타입이 아니다. WP-09는 Rust 결과의 target_stat을 labelKey, value를 final, tick과 entries의 source/operation/value/accumulated를 표시 문자열·키로 변환한다. Base/Add/Mul은 `ledger-base`/`ledger-add`/`ledger-multiply` 키를 사용하고 Base의 source 없음은 null로 전달한다. entries의 순서·값·누적값·최종값을 그대로 표시한다. 숫자 변환·반올림·합산·곱산·정렬을 하지 않는다. 실제 국가/주 조회와 게임 키는 WP-09에서 연결한다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| M0 JSON 계속 사용 | 추가 의존성 없음 | 02 Fluent 형식과 맞지 않음 |
| 폰트 CDN | 저장소·실행 파일 크기 감소 | REQ-LOC-02의 외부 font CDN 금지 |
| 폰트 subset/WOFF2 변환 | 다운로드 크기 감소 | M1은 전체 glyph 보존과 원본 OFL provenance를 우선하며 변환 도구·누락 관리가 추가됨 |
| 서버 원장 공식을 TS에 복제 | 표시 API를 빨리 연결 | 서버 권위·Rust 유일 프로토콜·REQ-NET-05와 맞지 않음 |
| hover만으로 tooltip 표시 | 짧은 스타일 | 키보드·터치에서 열기 어려움 |

## 결과와 영향
시뮬레이션과 저장 형식·Rust 프로토콜은 변경하지 않는다. 원장은 서버 문자열을 그대로 받아 큰 정수와 소수 정밀도를 보존한다. 기본 실행 파일에 약 10.4 MB 폰트가 추가되고 한 번 다운로드한다. 기본값은 기존 영어 셸을 유지하며 선택 즉시 한국어로 전환한다. 사용자 D-10 답변을 이 WP의 원본 코드(GPL-3.0-or-later)·FTL(CC-BY-SA-4.0)에 적용한다. 폰트는 원래 OFL-1.1이고, M0 원본 기록을 덮어쓰지 않는다. 상세 적용 범위·라이선스 고지는 client/LICENSE-WP12.md에 둔다.

공식 출처를 2026-10-06 확인했다:
- [Fluent 공식 package.json](https://github.com/projectfluent/fluent.js/blob/main/fluent-bundle/package.json): 0.19.1, Apache-2.0. npm registry 조회와 lockfile도 같으며 evidence/fluent-source.txt에 기록했다.
- [Fluent 공식 API](https://projectfluent.org/fluent.js/modules/_fluent_bundle.html).
- [Noto Sans KR metadata](https://github.com/google/fonts/blob/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/METADATA.pb), [TTF](https://github.com/google/fonts/blob/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf), [OFL](https://github.com/google/fonts/blob/b38c5c93af322c45f633e17ac440ec1e6c94d489/ofl/notosanskr/OFL.txt). Font name version 2.004-H2; upstream source notofonts/noto-cjk revision 523d033d6cb47f4a80c58a35753646f5c3608a78. 다운로드 Google/fonts revision b38c5c93af322c45f633e17ac440ec1e6c94d489과 SHA256은 작업 증거에 고정했다.

브라우저별 로컬 폰트 요청, MIME, 원본 SHA256 일치, document.fonts loaded 및 select computed font-family, ko/en 화면을 보존한다. 직접 cmap 검사로 ko 카탈로그 glyph·한자 예·한글 전체 음절 포함을 검사한다. 자체 테스트의 통과는 독립 검증 PASS나 main CI 증거를 대신하지 않는다.

최종 E2E 환경에서 Windows가 4173을 TCP 제외 범위에 넣어 bind를 거부했다. 기존 기본4173과 새 서버 실행 강제를 유지한 채 OH_TEST_PORT 선택을 허용해19412에서 검증했다. OS/네트워크 설정은 바꾸지 않는다. Windows Playwright WebKit은 wght400 computed axis·loaded font·glyph 검사에도 다른 엔진보다 얇게 보이며 실제 Safari 두께는 미검증이다. 캡처와 제한 사항을 작업 로그에 기록했다.
