# WP-24 M2-r1 first 구현 원 증거

source `c2c2f638fe5179d11ce5a318f6d991b4b2a0cd4d`의 구현 producer manifest와 원 파일 2722개를 보존한다. 부모가 당시 남은 원 files의 bytes/SHA를 전수 대조했고 mismatch 0개다. 독립 검증/통합/같은 main HEAD CI는 별도 증거이며 이 아카이브를 PASS로 쓰지 않는다. 불일치가 있으면 실제/기대 쌍을 parent-custody에 보존하고 원문을 복구하거나 삭제하지 않는다.
