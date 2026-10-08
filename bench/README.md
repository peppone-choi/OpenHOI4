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

성능 runner는 `defines.toml`의 고정 prior `81deb803944cf6297f13b3f14ca454774cea141b`와 실제 current HEAD의 library를 각각 release/default features로 빌드한다. 입력은 prior의 Git archive에서 추출한 **전체 testland 팩의 동일 경로**다. current 콘텐츠를 편집하거나 m1 파일만 선택하지 않는다. source별 원 lock/registry 버전·checksum, external driver metadata/path dependency·linked source 파일 SHA·binary SHA, 원 native 명령/exit/stdout/stderr/timeout을 보존한다. 전체 팩의 정렬 상대 경로·길이·SHA와 빈 디렉터리도 build/load/warmup/측정 전후 대조한다. 링크·Windows reparse·경로 이탈·추가/누락/변조 또는 빌드 이후 source/binary 변경은 거부한다.

WP-14의 driver/compare/test_compare 변경은 초기 좁은 소유 범위 밖에서 관측되어 부모가 보류한 뒤, CEO의 구체 기술 검토를 거쳐 추가 인수됐다(ADR-1401). 원 driver bytes와 동일하지 않다. 새 driver의 같은 bytes/SHA를 양 library의 `src/main.rs`에 사용한다. 실제 `oh_save::pack_hash`를 load/Instant 이전과 elapsed/기존 완료 검사 이후에 측정한다. 출력 `pack_hash`/`pack_hash_after`는 필수 16자리 소문자 hex이고 동일해야 한다. warmup prior-native hash에 current warmup과 모든 실측 표본을 바인딩한다. 전체 DTO/canonical/FNV state hash·기존 완료 검사도 계속 대조한다. 이전 driver와 역사 출력은 별도로 보존하며 새 필드를 소급 채우거나 이전 시간 표본을 섞지 않는다.

원 m1/seed1000/240000 steps/warm2400/두 쌍/엄격한 두 쌍 >15%/native240초/build1200초를 유지한다. pack 계측 비용은 타이머 밖이며 전체 native timeout에는 포함된다. Windows `--local-evidence`는 Linux gate 증거가 아니다. 새 디렉터리를 쓰며 이전 증거를 덮어쓰지 않는다. 이 고정 m1 검사는 신규 경제 활성 부하나 후속 UI 성능을 증명하지 않는다.

`performance.yml`과 기존 일곱 workflow bytes/조건/한도는 변경하지 않는다. 부모가 existing workflow를 exact candidate `--ref`로 dispatch하고 API/job/env/artifact의 실제 SHA와 Linux 양 쌍 원증거를 인수한다. 로컬 회귀 성공을 원격 실행이나 독립 PASS로 기록하지 않는다.
