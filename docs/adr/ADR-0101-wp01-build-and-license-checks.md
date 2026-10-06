# ADR-0101 WP-01 골격과 라이선스·에셋 검사

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-06 |
| 관련 WP·REQ | WP-01, REQ-PLAT-01, REQ-LEG-02, REQ-LEG-03 |

## 맥락

02 §4.1과 §14는 크레이트 계층, CI와 허용 라이선스를 정한다. 빈 크레이트의 의존 관계, npm 검사 도구, 배포 원천 파일 탐색 범위는 정하지 않았다. D-10은 미결정이므로 제품 코드에 라이선스를 임의로 부여할 수 없다. 서버 기능·프로토콜 타입은 WP-04/WP-05에서 구현한다.

## 결정

- Rust 1.99.0, edition 2024와 resolver 3을 고정한다. 8개 크레이트를 선언하고 02 §4.1의 단방향 로컬 의존 관계만 연결한다. 라이브러리는 모듈 문서, 실행 파일은 빈 빌드 진입점만 가진다. 외부 Rust 라이브러리는 후속 WP에서 사용 시 추가한다.
- 모든 자사 크레이트는 `publish = false`, npm 루트는 `private: true`다. LICENSE와 라이선스 선언을 만들지 않는다. cargo-deny의 private workspace 예외는 미결정 자사 라이선스만 위한 것이다. 외부 의존성은 허용 목록 검사 대상이며 `bincode`는 이름으로 금지한다.
- npm 11.4.2가 생성한 lockfile v3의 모든 package 항목을 `tools/check_npm_licenses.cjs`로 검사한다. 개발 의존성 및 다른 OS의 optional 의존성도 검사한다. 루트 자사 패키지만 제외한다. 식별자, 괄호, AND·OR를 해석하며 AND는 두 조건 모두, OR는 허용되는 선택지 하나가 있어야 통과한다. 알 수 없는 라이선스, 잘못된 표현, 목록에 없는 WITH 예외와 `+`는 거부한다. 검사 전 `npm ci`가 manifest와 lockfile의 일치를 확인한다.
- 추가 npm 검사 의존성을 사용하지 않는다. 조사한 `license-checker-rseidelsohn` 5.0.1 자체는 BSD-3-Clause였으나 실제 설치된 하위 패키지에 BlueOak-1.0.0, CC-BY-3.0 등이 있어 기존 허용 목록에 맞지 않았다. `spdx-expression-parse` 5.0.0도 하위 라이선스가 이 정책에 맞지 않아 채택하지 않는다. 허용 목록을 확대하거나 예외를 추가하지 않았다.
- 에셋 검사는 `assets/`, `data/packs/`, `client/public/`, `client/src/`의 파일을 탐색한다. 매니페스트 자체와 클라이언트 소스 코드 확장자 `.ts`, `.tsx`, `.js`, `.jsx`, `.css`만 제외한다. 따라서 데이터의 설명·출처 문서와 소스에서 import되는 이미지·현지화도 등록해야 한다. 개발·빌드 산출물과 node_modules는 탐색하지 않는다.
- 모든 레코드에 경로·제작자·출처·라이선스·수정 여부·AI 생성 여부·notes를 요구한다. 경로 이탈, 중복, 없는 파일, 미등록 파일과 허용 목록 밖 라이선스를 거부한다. 직접 제작은 `original`, 퍼블릭 도메인은 `public-domain`으로 표기한다. 폰트는 OFL-1.1만 허용한다. `--release`는 OPEN-09 결정 전 AI 생성물을 거부하며 CI assets 작업이 이 모드를 사용한다.
- CI는 Windows/Linux 매 push·PR, macOS는 공개 저장소에서 매 push·PR로 설정한다. 사용자 공개 전환 지시(2026-10-06)를 반영하여 현재 세 OS가 함께 실행된다. 비공개로 전환될 경우에는 macOS가 야간·태그·수동 실행으로 제한되는 분기도 유지한다. 표준 GitHub-hosted runner만 사용한다. 서버 build/fmt/clippy/test, 클라이언트 typecheck/Vitest/build/3 브라우저 smoke, 문서·에셋·라이선스·위반 픽스처를 실행한다. 게임 해시·TS 생성·데이터 검증·벤치·서버 E2E는 아직 지원하지 않아 호출하지 않는다.
- `cargo metadata`와 `cargo tree`로 결정론 계층의 네트워크·비동기 의존성 경계를 검사한다. 클라이언트에는 텍스트·게임 규칙·프로토콜의 임시 복제 타입을 넣지 않는다.

## 검토한 대안

| 대안 | 장점 | 버린 이유 |
|---|---|---|
| npm 라이선스 검사 라이브러리 설치 | 설치된 트리에서 라이선스를 추론할 수 있음 | 실제 하위 의존성이 02 §14.3 허용 목록 밖임 |
| npm `--production`만 검사 | 범위가 작음 | 개발·플랫폼별 의존성 누락 |
| package.json의 직접 의존성만 검사 | 구현이 단순함 | 전이 의존성을 검사하지 못함 |
| 프로젝트에 MIT 선언 | 자사 코드가 일반 라이선스 검사에 통과함 | 사용자만 결정할 수 있는 D-10을 임의로 확정함 |
| static public 디렉터리만 에셋 검사 | Vite public 파일을 쉽게 탐색 | import된 이미지와 데이터 파일을 놓침 |

## 결과와 영향

시뮬레이션·게임 수치·저장 포맷은 추가하지 않는다. 후속 WP는 기존 크레이트와 lockfile을 확장한다. npm 검사 범위는 배포 시 설치되는 플랫폼에 한정하지 않으며, 실제 라이선스 문서의 법적 해석을 대신하지 않는다. WebGL2 지도, 실제 Chrome/Edge/Safari 호환 확인, 게임 해시와 프로토콜 계약 검증은 해당 후속 WP에 남는다. 라이선스 결정과 independent 검증·기본 브랜치 CI 실행은 이 구현 세션이 판정하지 않는다.

2026-10-06 공식 API 및 문서 확인:

| 구성 | 확인값·라이선스 | 출처 |
|---|---|---|
| Rust | 1.99.0, MIT/Apache-2.0 | [공식 설치](https://rust-lang.org/tools/install/), [공식 릴리스](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) |
| cargo-deny | 0.20.2, MIT OR Apache-2.0 | [공식 registry](https://crates.io/api/v1/crates/cargo-deny), [라이선스 설정](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html) |
| npm CLI | 실행 버전 11.4.2, Artistic-2.0 (외부 개발 도구; 제품 의존성 아님) | [공식 배포](https://registry.npmjs.org/npm/11.4.2), [lockfile 문서](https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/) |
| React·react-dom·types | 19.3.0, MIT | [React](https://registry.npmjs.org/react/latest), [DOM](https://registry.npmjs.org/react-dom/latest), [React types](https://registry.npmjs.org/%40types%2Freact/latest), [DOM types](https://registry.npmjs.org/%40types%2Freact-dom/latest) |
| Vite·React plugin | 8.3.3·6.1.2, MIT | [Vite](https://registry.npmjs.org/vite/latest), [plugin](https://registry.npmjs.org/%40vitejs%2Fplugin-react/latest) |
| TypeScript | 7.0.2, Apache-2.0 | [공식 registry](https://registry.npmjs.org/typescript/latest) |
| Vitest | 5.0.3, MIT | [공식 registry](https://registry.npmjs.org/vitest/latest) |
| Playwright | 1.63.0, Apache-2.0 | [공식 registry](https://registry.npmjs.org/%40playwright%2Ftest/latest) |
| 조사 후 미채택 검사 도구 | license-checker-rseidelsohn 5.0.1, BSD-3-Clause; spdx-expression-parse 5.0.0, MIT | [checker registry](https://registry.npmjs.org/license-checker-rseidelsohn/latest), [parser registry](https://registry.npmjs.org/spdx-expression-parse/latest) |
| Actions | checkout v7.0.1; setup-node·setup-python v7.0.0 | [checkout](https://github.com/actions/checkout/releases/tag/v7.0.1), [node](https://github.com/actions/setup-node/releases/tag/v7.0.0), [python](https://github.com/actions/setup-python/releases/tag/v7.0.0) |
| actionlint | 1.7.12, MIT; 로컬 CI 문법 검사 도구 | [공식 릴리스](https://github.com/rhysd/actionlint/releases/tag/v1.7.12) |

실행 도구 버전·출처는 `docs/worklog/evidence/WP-01/versions.txt`에도 기록한다. 기존 02 버전 표와 일치하므로 그 표와 추적 매트릭스는 수정하지 않는다.
