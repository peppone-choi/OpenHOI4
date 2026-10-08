# 경제 target 범위 회귀 입력

`project-target-min.ohsave`는 I32 최소 target을 가진 의미 오류 입력이며 SHA256은 `da9462945dca0d308f23b1caa0778de7fd59ec681482a44ff2c399dcc69db359`다. `control-valid-target2.ohsave`는 target2 유효 control이며 SHA256은 `1c07adbd4b7e145ef38efa87b46966cf5a5c484659dc304e20ac5cc19dbb8275`다. 이들은 직접 작성한 합성 Testland 입력이며 게임 콘텐츠 기본값이 아니다.

`INPUT_MANIFEST.json`의 상대 경로·길이·SHA256과 해당 팩22파일/5617bytes를 유지한다. restore-stage 팩5681bytes와는 두 TOML의 CRLF64bytes 차이가 있으므로 각 save에 맞는 팩을 사용한다. save/header/pack identity·기대값·개행을 바꾸지 않는다.

저장소 루트에서의 재현 명령이며 이번 인계에서는 실행하지 않았다:

```sh
python crates/oh_server/tests/economy_target_native.py --out target/evidence/WP-14-M2-r3-P06-2/economy-target-repro --bin-dir target/debug
```

CLI/server가 미리 빌드돼 있어야 한다. 출력은 코드가 허용하는 위 접두어 아래의 새 폴더여야 한다. 도구는 각 입력을 새 출력 폴더에 복사하고 MIN `economy:InvalidValue`/exit1, restore-stage `economy:TargetConflict`/exit1, control exit0/HTTP200/query target2를 구분한다. PackMismatch 또는 panic을 의미 검증 성공으로 계산하지 않는다.
