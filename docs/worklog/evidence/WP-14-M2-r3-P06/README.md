# WP-14 M2-r3 P-06 증거

제품 source48067b702f8434decf37ad294d34ce5b0b557976, 실제 clean Windows 실행. a933 원 P05 FAIL과 작업 중 후보 결과는 별도 원문이다. 최종 문서 HEAD는 제품 source와 구분한다.

evidence.json의 ignored ZIP locator·길이·SHA를 확인하고, ZIP의 MEMBER_PHYSICAL.json으로 각 원 물리 경로·길이·SHA·CRC를 대조한다. 원 root는 target/evidence/WP-14-M2-r3-P06이며 부모는 worktree 정리 전에 ignored ZIP/원물리 자료를 보존해야 한다.

첫 client whole245/246 timeout과 같은 전체·같은5s 단1회 진단246 통과를 함께 보존한다. 첫 seal은 후속 cargo build로 CLI SHA가 달라 중단됐으며 원 오류는 seal-initial-failure.log에 byte-exact 복사했다. 최종 binary snapshot과 native-final의 원458B CLI/server normal/force 네 거부를 별도로 바인딩했다. 원 자료를 고쳐 성공으로 만들지 않았다.

이번 bundle은 새 독립 P05/actual Linux/3OS/source7CI/mainCI/통합 판정이 아니다. 원 a933 원격 결과를 새 source에 소급하지 않는다. 벤치/CI/원 frozen fixture/생산콘텐츠/의존성 및 게임 결정 상태는 변경하지 않았다.
