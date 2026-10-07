# WP-25 재현·성능 인수 대조

2026-10-07. W1-a 동안 준비한 오케스트레이터 문서다. WP-25는 WP-13까지 exact 독립 PASS·P-07·같은 main CI 후 시작하며 실제 통합된 command/save/loader 타입을 인수한다. 아래는 기존 REQ-SAV-05·REQ-PERF-03의 검증 경계이며 완성된 제품 스키마가 아니다.

## 실제 기록과 재생

02 §7.3의 ZIP은 bundle.toml, 기계·사람이 읽는 명령 로그, 선택 시작 저장으로 구성된다. seed만 다시 돌리는 기능은 시작 상태와 명령 로그의 재현 증거가 아니다. 기록 시작 시의 전체 권위 snapshot·기존 pending queue·pack/context/version, 기록 후 실제 입력의 순서, 명령 거부 결과와 pump/step 경계, 종료 상태와 기대 hash를 설계에 대응시킨다. CLI record의 실제 입력원과 게임 실행 연결을 코드 전에 명시한다. 테스트 fixture를 기록한 결과와 실제 실행에서 얻은 기록은 구분한다.

tick만으로 명령을 재생하면 Pause 중 여러 번의 pump와 같은 tick에 새로 들어온 명령의 순서를 잃을 수 있다. 예정 명령의 `(tick,nation,sequence)` 정렬과 실제 host 입력·step 순서는 별개다. 기존 pending 명령을 시작 snapshot과 새 로그 양쪽에서 중복 enqueue하지 않는다. 사람이 읽는 로그는 기계 로그와 같은 명령을 설명하며 실행 권위 원문과의 관계를 정의한다. malformed/변조 로그는 replay 전 거부하며 정상 종료·paused 종료·종료 조건·예약 이동/Stop·flags 등 실제 존재하는 신규 상태를 fresh native process로 재개해 전체 DTO/canonical/hash를 비교한다.

실제 게임/서버 기록 연결이 oh_cli/oh_save 소유만으로 불가능하면 구현자가 구체 호출 경로와 필요한 추가 파일을 부모에게 보고한다. 부모는 다른 producer와 겹치지 않게 소유를 인계한다. 연결되지 않은 exporter나 fixture-only record를 실제 게임 기록 완료로 선언하지 않는다. 구현자는 범위 밖 제품을 직접 바꾸지 않는다.

## ZIP·파일 실패

| 입력 | 설계와 독립 검사 |
|---|---|
| member 경로 | 절대 경로·drive/UNC·`..`·역슬래시 별칭·NUL·중복/정규화 충돌·symlink/reparse를 검사한다. 읽기만으로 가능한 경우 unpack하지 않는다. |
| 자원 | compressed/uncompressed/member별/총합 크기와 member count의 상한, 선언 크기 불일치·잘린 ZIP·압축 폭탄을 실제 검사한다. 상한은 의미가 있는 기술 예산으로 기록한다. |
| schema | 필수/null/unknown field/미래 version, ID·scenario·seed·end tick·정렬·중복 명령·참조 오류와 한 비트 hash 변조를 검사한다. |
| pack/save | caller가 임의 호환 purpose를 고르지 못한다. 실제 bounded save format/context와 팩 원천 identity, 현재/역사 source를 구분하고 codec 거부를 유지한다. |
| 실패 상태 | target 기존파일/원 inputsave/pack bytes, 기존 sim·queue·clock의 보존 범위를 정한다. 부분 write와 기존 파일 교체 실패는 실제 native I/O 증거로 확인한다. |

## 같은 러너 성능 비교

02 §14.1은 Linux에서 기준 commit과 current commit을 같은 러너에서 연속 실행하고, 15% 초과가 두 회 재현되면 실패하도록 정했다. 기준 SHA는 측정 전에 고정하며 current를 자기 기준으로 삼지 않는다. 기준에 아직 없는 CLI/시스템을 없다고 성공 처리하지 않는다. 공통의 실제 workload를 두 source에 실행할 방법과 신규 M2 workload의 별도 기준을 설계한다. 외부 동일 driver를 쓰면 두 source의 실제 library를 각각 빌드한다는 증거가 필요하다.

build/download와 warmup은 측정에서 분리한다. compiler/profile/features/pack/seed/작업량/명령/state hash/host 환경/두 native process의 시간 원문을 보존한다. wall clock은 host 측정에만 사용한다. 측정 누락·실패·0·비유한 값은 오류이며 비교를 건너뛰어 green으로 만들지 않는다.

독립 산술 예제는 baseline 100과 current 115이면 정확히 15%여서 통과, current 116이면 초과다. 두 쌍이 모두 116이면 실패, 116과 115이면 두 회 재현이 아니다. 이 수치는 측정 단위와 무관한 비교 예제이며 실제 baseline이나 표본 예산 승인이 아니다. 정수 overflow를 피하는 비교를 명시하고 기준/현재 각 실행의 완료 workload와 hash를 함께 확인한다. CI workflow는 기존 네 CI를 보존하고 실제 native 비교·artifacts를 실행해야 한다.
