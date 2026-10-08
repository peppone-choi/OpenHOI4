# 경제 restore-stage 회귀 입력

`industry-level4.ohsave`는 직접 작성한 합성 Testland의 v5 save fixture다. 458 bytes, SHA256 `c67f5cefbba865be78643b920536678f789b8375b418b9034ddd2cd2b4db1e53`. `packs/testland`는 대응하는22파일·5681bytes 입력이며 `INPUT_MANIFEST.json`은 각 상대 경로·길이·SHA256을 고정한다. save/header/pack identity와 입력 개행은 변경하지 않는다. 기존 production/frozen fixture를 대체하지 않는다.

현재 건물 level4가 경제 정의의 costs/slots max3을 초과하는 의미 오류를 검사한다. CLI/server normal/force를 포함해 복원 시 `economy:TargetConflict`로 거부해야 한다. PackMismatch나 panic은 기대하는 거부를 대신하지 못한다. journal ZIP이 아닌 저장 파일이며 합성 계수를 게임 기본값으로 쓰지 않는다.

저장소 루트에서의 재현 명령이며 이번 인계에서는 실행하지 않았다:

```sh
cargo run -p oh_cli --locked -- resume --load tests/repro/WP-14-M2-r3-restore-stage/industry-level4.ohsave --pack tests/repro/WP-14-M2-r3-restore-stage/packs/testland --ticks 0 --hash-out
python crates/oh_server/tests/economy_restore_native.py --out target/evidence/WP-14-M2-r3-P06-2/restore-stage-repro
```

native helper는 CLI/server 실행 파일을 미리 빌드한 환경에서 사용한다. 출력은 코드가 허용하는 위 접두어 아래의 새 폴더여야 한다. 종료 코드·진단·복원 상태를 구분하고 원 fixture를 덮어쓰지 않는다.
