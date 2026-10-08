# WP-14 M2-r3 F-01 전용 저장 복원 반례

원 독립 P05 a933 source의 industry-level4.ohsave 458bytes를 그대로 복사했다. SHA256 c67f5cefbba865be78643b920536678f789b8375b418b9034ddd2cd2b4db1e53. packs/testland는 원 own-inputs/plain 전체 22파일/5681bytes다. INPUT_MANIFEST.json이 각 원 경로·길이·SHA를 보존한다. 기존 production/frozen fixture를 수정하지 않는다.

실제 권위 현재 level4, 경제 costs/slots max3. 기존 CLI/server normal/force는 수락했고 초기 loader는 거부했다. 기대는 모든 복원 입구에서 의미 오류 거부다. 이 파일은 journal ZIP이 아니라 v5 저장이므로 `oh_cli resume --load ... --pack ... --ticks 0 --hash-out`으로 재현한다.

`python -X utf8 crates/oh_server/tests/economy_restore_native.py --out target/evidence/WP-14-M2-r3-P06/native-<새이름>`은 actual CLI·server normal/force를 실행하며 각 exit/argv/cwd/PID/exeSHA/원 stdout/stderr와 입력 불변성을 남긴다. source 수정 전에는 native1(거부 기대 위반), 수정 뒤 native0이어야 한다. synthetic 경제/지도/국가 입력이며 새 게임 수치/콘텐츠 기본값이 아니다.
