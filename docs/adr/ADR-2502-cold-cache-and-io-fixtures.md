# ADR-2502 고정 lock 캐시 준비와 regular IO 검사 fixture

| 항목 | 값 |
|---|---|
| 상태 | 채택 |
| 날짜 | 2026-10-08 |
| 관련 WP·REQ | WP-25 P06-1, REQ-PERF-03, REQ-SAV-05 |

## 맥락
e85ec5b7191604e7fb1903d2cea5fb6e49060a5b의 실제 Linux performance run37654381217/job112905653439 step7은 baseline-metadata native101, runner native2였다. native 측정표본/15% 판정은 만들어지지 않았다. 부모가 원 artifact11498363427의 HTTP206 범위로 읽고 memberCRC를 대조한 preview 원문은 `failed to download crunchy v0.2.4 / attempting to make an HTTP request, but --offline was specified`다. preview stderr SHA256은 2bef6c5b5f3e8573574f4ae5d1fb6613296762150758646a49224f47c64bb291이다. 전체1.7GB artifact digest 봉인은 preview와 구분하며 부모가 담당한다. 원 prior81deb lock과 e85 lock에 crunchy0.2.4 및 체크섬 460fbee9c2c2f33933d720630a6a0bac33ba7053db5344fac858d4b8952d77d5가 있다.

직접 읽은 Mac Trigger37654381215·Save37654381473·CI37654381212 원 로그는 새 partial IO unit test의 `error.contains("partial write failure")`에서 native101을 확인했다. 원 오류 반환값/경로는 e85 로그에 출력되지 않아 그 Mac 프로세스의 앞단 오류를 확정할 수 없다. `safe_path`는 모든 ancestor의 symlink/reparse를 거부하고 기존 fixture는 OS tempdir의 lexical path를 제공했다. Rust 공식 문서는 temp_dir의 결과가 symlink일 수 있다고 설명한다. 이 경로 가설을 검사 fixture에서 분리하고 실제 writer 주입에 도달했는지를 명시적으로 증명해야 한다. 원 단언을 낮추거나 production guard를 느슨하게 만들지 않는다.

## 결정
- 각 source workspace의 실제 Cargo.toml/Cargo.lock에 `cargo fetch --locked`를 먼저 실행한다. 허용된 기존 네트워크로 정확한 lock의 캐시 자료를 준비하는 단계이며 source 파일·lock·features/profile/network 설정을 변경하지 않는다. 원1200s build/preparation timeout을 적용한다. source lock 원 bytes를 fetch 전후와 external metadata 뒤 대조한다. 실패는 즉시 오류이며 metadata/측정에 진행하지 않는다.
- 새 external workspace lock은 이전처럼 각 source lock copy를 seed로 삼고 `cargo metadata --offline --format-version 1`로 root 추가/미사용 workspace package 정리만 한다. 그 뒤 원 registry name/version/source/checksum을 감사하고 release build는 --locked다. 없는 캐시를 통과로 취급하거나 새 registry 해석을 허용하지 않는다. build/download/cache 준비/warmup은 측정 밖이다. 고정 prior81deb, current clean, 실제 library/driver, m1/seed1000/work240000/warmup2400/two pairs/>15%, native240s/build1200s를 보존한다.
- IO positive fixture는 지정한 소유 namespace의 고유 tempfile 디렉터리를 keep하고 `regular` child의 canonical path를 transaction에 제공한다. 원 partial write error 단언, 실제17bytes prefix write/flush/metadata 검사, input/기존target/authority 보존과 cleanup count를 유지하고 injection_reached 및 native prefix bytes를 추가로 검사·보존한다. 별도 symlink/junction fixture는 canonicalize하지 않은 입력의 앞단 거부와 writer 미도달을 검사한 뒤, test Host가 신뢰하는 regular target의 canonical path로 실제 partial injection을 검사한다. production read/write/safe_path/codec은 변경하지 않는다.
- Windows canonical 경로의 extended-length prefix는 PowerShell junction target 문법과 분리한다. 새 negative fixture의 최초 native101/os123은 정상 syntax의 외부 setup 경로로 고쳤고 원 실패 및 재시도 stdout/stderr를 보존했다. Windows 결과는 Mac 실행 결과로 표시하지 않는다.
- `repro_native.py --evidence-root PATH`, `wp25_evidence.py --out PATH`로 P06 원증거를 별도 namespace에 저장한다. 기존 기본 출력과 초기 whole/review/index/manifests는 보존한다. 새 namespace 포장은 P03 prior/raw 전체를 자동으로 다시 포함하지 않는다. 이전 원본 링크와 SHA는 custody 기록으로 연결하며 새로운 source/명령/선택 범위를 명시한다. 임시 fixture는 삭제하지 않는다.

## 검토한 대안
| 대안 | 장점 | 버린 이유 |
|---|---|---|
| metadata 온라인 처리만 허용 | 적은 명령 | exact source lock으로 cache 준비를 별도로 증명하기 어렵고 cold resolution 감사 경계가 불명확 |
| missing crate/metadata를 건너뜀 | 빠름 | 실제 측정·기준선 증거를 없애며 오류를 통과로 숨김 |
| symlink guard 완화 | OS temp alias 허용 | 사용자/ZIP IO 거부 경계를 약화하며 부분쓰기 검사 목적과 다름 |
| partial IO 단언 삭제/skip | CI 녹색 | 실패 주입에 도달하지 않아도 통과하는 검사로 퇴행 |

## 결과와 영향
기존6 workflows와 performance.yml root7 step/trigger/artifact 계약, oldcodec/fixture/golden/게임·sim/data/renderer/wire/assets/의존성버전/프로필/baseline source는 바꾸지 않는다. P06-1은 원 e85 P05-1을 대체하거나 원 CI FAIL/fixture gap을 지우지 않는다. 실제 Linux/Mac 새 source CI는 부모가 exact 브랜치를 게시한 뒤 판정한다.

공식 확인(2026-10-08): [Cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html) — 기본 graph가 모든 target platform을 포함하며 offline은 캐시 자료가 필요하다. [Cargo fetch](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html) — lockfile의 의존성을 준비하고 --locked는 lock 변경을 거부한다. [std::env::temp_dir](https://doc.rust-lang.org/std/env/fn.temp_dir.html) — 반환 경로는 symlink일 수 있다. [std::fs::canonicalize](https://doc.rust-lang.org/std/fs/fn.canonicalize.html) — 중간 symlink를 해소하고 Windows에서 extended-length syntax를 반환한다. 새 의존성은 없다.
