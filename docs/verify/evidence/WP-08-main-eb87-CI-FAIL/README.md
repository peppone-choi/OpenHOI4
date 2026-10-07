# main eb87 정상 CI의 DPR 두 FAIL 원문

[일반37547131178](https://github.com/peppone-choi/OpenHOI4/actions/runs/37547131178)은 exacteb87ae974601e397ad638d2eaf4ea5b84fde8865에서 실패했다. 이 HEAD는 M1-r3 부모의 docs-only 준비이고 제품/원fixture는 dispatch688과 같다. 최신FAIL을 earlier688 SUCCESS로 덮지 않는다. 새 source0ab 관측묶기는 아직 이 HEAD에 포함되지 않았다.

원M036PASS·M180/82PASS, Chromium DPRpreferred/forcedGL 두전체30초timeout이다. 원stack은 buffer readiness poll line42와 capture후buffer poll line54를 가리킨다. 뒤 Firefoxnative/전체headed 단계는 실행되지 않았다. 저장/코어/시뮬레이션의 같은eb87 CI 성공은 일반FAIL을 상쇄하지 않는다.

[원문ZIP](original-evidence.zip)은 run/jobs/artifact API·client 원로그·identity와 내려받은 original-artifact.zip을 그대로 포함한다. [manifest](manifest.json)는 member bytes/SHA와 wrapper SHA를 기록한다. API artifact11451044255/2783545bytes/169members 원ZIP SHA7b75da81c17a83c9ac0a738fde2a99480d9088292872a7ff61a030bd28979119가 API digest와 직접 일치했다. parent의 GH수집명령은 실제exit0이다.

원실패·미실행을 소급변경/재실행하지 않는다. source0ab의 Windows205PASS·freshP05·P07·정상같은main네CI·새P12는 별도로 판정한다. 임시f19/Linux진단이나 isolated 성공을 이 원FAIL의 대체자료로 쓰지 않는다.
