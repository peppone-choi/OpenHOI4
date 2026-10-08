# WP-14 M2-r3 P-06-2 target MIN 반례와 유효 control

원 P05-2의 MIN 저장469B SHA da9462945dca0d308f23b1caa0778de7fd59ec681482a44ff2c399dcc69db359와 유효 target2 control SHA1c07adbd4b7e145ef38efa87b46966cf5a5c484659dc304e20ac5cc19dbb8275를 그대로 보존한다. INPUT_MANIFEST.json의 원 팩22파일/5617B는 F01 원 팩5681B와 두 TOML의 CRLF64B 차이로 다르다. 각 저장에 맞는 context를 사용하며 save/header/packhash를 변경하지 않는다.

`economy_target_native.py --out <새 P06-2 namespace> --bin-dir <실행 전 복사한 immutable CLI/server 폴더>`는 MIN·유효 control·F01의 actual normal/force를 실행한다. 각 원 manifest·header/pack identity를 먼저 확인하고 MIN economy:InvalidValue/native1, F01 economy:TargetConflict/native1, control native0/HTTP200/query target2를 구분한다. PackMismatch 또는 panic을 semantic 성공으로 계산하지 않는다.
