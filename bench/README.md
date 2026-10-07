# WP-25 Host 도구

현재 national Testland를 실제 CLI 세션으로 실행하며 입력을 기록한다:

```text
cargo run -p oh_cli -- repro record --out target/session.zip
{"op":"enqueue","tick":0,"nation":0,"sequence":1,"command":{"Pause":true}}
{"op":"pump"}
{"op":"enqueue","tick":0,"nation":0,"sequence":2,"command":{"Pause":false}}
{"op":"step"}
```

EOF로 종료한다. 각 입력 직후 stdout event가 나오고, ZIP commit 뒤 최종 전체 상태 report가 나온다. 기존 save로 시작하려면 `--load FILE --pack ROOT`, 새 national 세션은 `--pack ROOT --scenario ID --seed N`을 쓴다. `tick`은 예약 명령의 실행 시점이며 `sequence`는 Host 입력이 제공하는 도착 순번이다. 같은 tick의 여러 pump 순서는 로그에서 보존한다. `Move`, `Stop`, `Effects`는 기존 simulation과 실제 저장에 존재하는 문맥에서만 가능하다. 신규 부대나 효과 정의를 CLI가 만들지 않는다.

```text
cargo run -p oh_cli -- repro run target/session.zip
cargo run -p oh_cli -- bench --pack data/packs/testland --scenario m1 --seed 1000 --steps 240000
python -X utf8 bench/run.py --out target/performance
```

`repro run`은 각 실제 결과, 종료 DTO/canonical/hash를 대조하고 0/1을 반환한다. ZIP32 stored만 지원하며 압축된 start.ohsave의 zstd는 기존 bounded codec이 처리한다. 외부 ZIP의 deflate·ZIP64·추가 파일은 거부한다. Active 팩 검증을 우회하는 force/purpose 옵션은 없다. 현재 M0 예제의 원 현지화 오류는 거부하며 기존 `run`/`resume` 호환 경로를 바꾸지 않는다.

성능 runner는 `defines.toml`의 고정 기준 SHA와 current library를 같은 Linux runner에서 별도 release build한다. 각 source의 기존 lock에서 만든 external driver lock은 원 registry 버전/checksum과 대조하고 빌드는 --locked다. 실제 pack 파일 bytes/SHA, 환경·컴파일러·원 native 명령/exit/stdout/stderr·전체 종료상태를 보존한다. 로드/빌드/warmup을 측정에서 제외하고 baseline/current 두 쌍 중 모두 15%를 초과할 때 실패한다. 0·누락·실행 실패·workload/state 차이는 오류다. Windows는 `--local-evidence`로 driver 동작을 확인할 수 있으나 Linux gate 증거가 아니다. 새 디렉터리를 사용하며 이전 측정이나 baseline source를 덮어쓰지 않는다.

`performance.yml`의 필수 `bench` job은 main/PR, 허용된 WP-25 사전검증 브랜치 push, workflow_dispatch에서 같은 검사를 실행한다. 기존 6 workflows/jobs는 변경하지 않는다. 최초 branch-only dispatch 대신 부모가 검토한 exact 브랜치를 게시해 Linux 증거를 얻는다.
