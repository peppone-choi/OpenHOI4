# WP-13 M2-r2 first 구현 원 증거

source `ff78a7501bace6fac1741d6cc596be524f887451`의 구현 producer manifest와 원 파일 2154개를 보존한다. 부모가 당시 남은 원 files의 bytes/SHA를 전수 대조했고 mismatch 0개다. 독립 검증/통합/같은 main HEAD CI는 별도 증거이며 이 아카이브를 PASS로 쓰지 않는다. 불일치가 있으면 실제/기대 쌍을 parent-custody에 보존하고 원문을 복구하거나 삭제하지 않는다.

producer-originals.zip는 구현자의 실제 원ZIP이며 parent originals.zip는 같은2154파일+원manifest를 재포장한 별도ZIP이다. 서로 다른 이름/CRC/압축으로 SHA가 다르므로 같다고 기록하지 않는다. 부모는 양쪽과 실제source 전member를 직접대조했다. parent-originals.zip는 actualAPI final·실제prompt/createpayload·원producercustody를별도로보존한다. 새검증의 준비조회오류는initial baseline교체없이완성marker/ready를확인한뒤정정됐고app생성전이었다.
