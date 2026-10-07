# M2-r1 증거와 실제 통합 상태

2026-10-07, W1-a P-07 수정 단계. WP-24 merge `0c58e4f8c78c1485cfee4ad9c7c4fe1c83b82898` 뒤 WP-17 merge `9a2302540cbe29610d052ad9286e3b8f74939933`를 만들었다. ASSETS 기존30행 원 bytes를 유지하고 서로 다른21+21행만 합집합으로 병합했다. 제품 코드 판단 충돌은 없었다. exact `500ba0f91cc53ed10660fca14294d4deb5a146d4` 통합 검사에서 npm ci/test/build·fmt·clippy는0, workspace는101이었다. 변경 팩 시험이 en에만 새 키를 추가하여 새 presentFTL parity 검사가 ko 누락을 거부했다. [원 P-07 실패](evidence/M2-r1-W1a-P07-500/README.md)의15파일 bytes를 보존했고 남은24명령은 미실행이다. WP-17에 같은 ko 키를 추가하는 유효 시험 입력 보완만 인계했으며 모든 원 단언·codec·validator·frozen fixture를 유지한다. 새 exact P-05/P-07/같은 main CI 전에는 통합 완료로 기록하지 않는다. W1 전체 통합 0개, W2 전체 통합 0개, M2 게이트 판정 없음. 소묶음 W1-a를 C-02의 별도 묶음으로 세지 않는다.

| WP / source | 구현 원 증거 | 독립 P-05 | P-07 / 같은 main CI |
|---|---|---|---|
| WP-24 최초 `c2c2f638fe5179d11ce5a318f6d991b4b2a0cd4d` | [2722파일 원 봉인](../worklog/evidence/WP-24-M2-r1-first/README.md) | [유효 FAIL](../verify/WP-24.M2-r1.attempt1.md), 실제 CLI legacy defines 및 native dependency/FTL 세 누락 | 미통합. 수정 source로 이 실패를 대체하지 않음 |
| WP-24 P-06 `4aecc2287019ef96df99f8c3d03232c8a482fe8f` | [6268 regular 원 봉인](../worklog/evidence/WP-24-M2-r1-P06/README.md), reparse fixture는 원 manifest metadata만 보존하고 따라가지 않음 | [새 유효 PASS](../verify/WP-24.M2-r1.attempt2.md). 실제final12645chars, 최초9780/raw `b5c720316f2f2756f749ac55467c262c3dc49bc3e5e5f91deb9e56723731284e` 전후동일, manifest2551/ZIP2552 전수 불일치0, 실제native70 | P-07 예정. 원 first FAIL과 일반 locale/역사warning 구분 유지 |
| WP-17 최초 `97f688718dabed46bc477fa249798ac09e999581` | [217파일 원 봉인](../worklog/evidence/WP-17-M2-r1-first/README.md) | 이 source의 P-05 앱은 만들지 않음. 승인 해협 추가 후 최종 source를 검증 | 미통합 |
| WP-17 P-06 `7f8cb7635ad10beb815503ca5a8137da0500faa3` | [234파일 원 봉인](../worklog/evidence/WP-17-M2-r1-P06/README.md) | [유효 scoped PASS](../verify/WP-17.M2-r1.attempt1.md). 실제final10836chars·최초9770/raw `182b8651813e1cf22211586db52b0cb98eae48e599d3e690109d30f2859549b9` 전후동일·manifest740/ZIP741 전수 불일치0 | WP-24 뒤 통합 예정. 실제 unit 생성/public wire/UI·M2 전체 완료 증거는 후속 |
| WP-17 통합 입력 보완 `c28f90425905774c23b1d90a5d900c596ade2745` | [188파일 원 봉인](../worklog/evidence/WP-17-M2-r1-P06-integration/README.md), 같은 ko 시험키5줄과 기록만 변경 | [FAIL·불변성 무효](../verify/WP-17.M2-r1.attempt2.md). 필수 Save CI가 역사 CLI-only 팩을 서버 정상 입력으로 사용. tracked TS generator가 같은bytes를 재작성했고 raw entry mtime가 달라짐. index 갱신 주체는 미확인, 최초/최종 원raw 보존. manifest1597/ZIP1598 전수bytes 불일치0 | 유효 PASS 아님. WP-24가 current capture/서버 CI 연결을 좁은 P-06으로 보완한 뒤 새 source 독립 검증 필요 |
| WP-13 / WP-25 | 실제 선행 producer 인수와 W1-a 통합을 기다림 | 미착수 | 미통합 |
| WP-14 / WP-23 | W1 전체 후 W2 producer 인수 | 미착수 | 미통합 |

원 증거의 권위는 original ZIP의 member bytes와 실제 앱 final API다. 인접 JSON은 Git 줄바꿈 정규화가 가능한 표시 사본이다. WP-17/24의 manifest·ZIP-IDENTITY 원 bytes는 각 producer-sidecar-originals.zip에도 보존했다. target full-report와 actual 앱 final은 각각 별도 원문이며 docs/verify 리포트는 actual final 전문과 동일한 내용을 기록한다.

부모의 준비 실패도 남긴다. WP-17 actual read의40000자 요청은 도구 최대20000 거부였고, 최초 custody parser는 scope가 붙은 PASS cell을 읽지 못해 exit1이었다. 원 parent-after와 최초 before/raw를 그대로 유지한 custody-only 재개로 전수0을 확인했다. 새 검증·baseline/index 복구·제품 변경으로 통과시킨 것이 아니다. 문서 CI 수집의 session57027 완료 전에 summary를 읽은 FileNotFound1은 실제 session exit0 이후 원 실패 메모와 구분해 기록했다. 이전 CLI HTTP400·DPR/Edge 원인 미확정 기록은 보존한다.

REQUEST-0006/0007/0008은 한정 CEO 잠정 채택이며 사용자 D/OPEN 추가 확정이 아니다. 역사 이름 정책의 source3+mutable19·caller3·actual v1 context·정확한 두 진단과 strict/headless/current 경계는 새 WP-24 P-05에서 직접 검사한다. 원 v1·원97f v2·current v2/v3의 pack identity와 codec 보존은 합쳐진 실제 source의 P-07에서 다시 대조한다. 01 §11의 구현 차별화 행은 독립 PASS 후 통합 시 반영한다.

2026-10-07T12:21 CEO 한정 기술 인수에 따라 원 frozen 두 capture·fixture/CLI/3OS 비교를 유지하고 별도 current capture 두 회·fresh native 전체 state/canonical/hash·원장/신원·3OS 비교를 서버 정상 input에 연결한다. 원 HTTP/WS/query/JS/정상 종료 단언을 유지하고 역사 mutable-v1 서버 거부는 별도 native1 negative로 남긴다. 정책 caller/codec/header/expected/골든·게임 규칙은 바꾸지 않는다. 검증에서는 tracked TS writer를 실행하지 않고 정확 read-only equality 또는 actual typescript()의 ignored 출력과 원 bytes를 비교한다. 원 raw 불변성 실패를 복구·면제·baseline 교체로 해소하지 않는다.
