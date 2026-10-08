# WP-14 M2-r3 P-06-2 증거 인수

검사 source: `5c56a4e2ccd1d79bc943695d269d9a22a816777e`. 제품 checked target 수정: `4371bf475b8b510669398c3a27ca549bbfecbadd`. 이후 본 기록 커밋은 docs-only다. 구현자가 formal P05/마일스톤 완료를 판정하지 않는다.

- 실제 Windows raw root: `E:/openhoi/.orchestrator/wt/WP-14-M2-r2/target/evidence/WP-14-M2-r3-P06-2`
- 원 ZIP: `E:/openhoi/.orchestrator/wt/WP-14-M2-r2/target/evidence/WP-14-M2-r3-P06-2-source5c56a4e-v1.zip`
- 길이: 44,990,470B / SHA-256 `c52257875b7d53444b5c3095ca87b398cbc87d2352e59a0427c243cf7f4f747d`
- ZIP 2494 members / physical 2493 files; 전체 member→physical bytes/CRC/SHA 재검사 불일치0. `MEMBER_PHYSICAL.json`이 절대 원물 경로를 매핑한다.
- 첫 sealer 원copy는 실행 전 read-only로 보존했다. SHA `3d37d1465fecc0e54f44142c6631b31dcd1f01a85b28de4268dde208fa9541bf`. 외부 stdout/stderr/argv/PID/exit0는 자기참조를 피하기 위해 ZIP 외부 원sidecar와 custody 복사본에 있다. ZIP/script 첫 실행 실패나 덮어쓰기는 없다.
- 중간 probe target 두개 및 repro 공격 reparse alias만 제외했고 링크는 따라가지 않았다. 두 actual probe 원source/lock/linkage metadata·실행 전 copy한 executable과 stdout은 포함한다.

`command-matrix.json`과 두 pipeline JSON은 source5c에서 whole 검사38명령 각각0, 최종 pinned native12/legacyF01 4/atomic4, sourceaudit0을 보존한다. 원 stage 두파일의 byte-exact CRLF 보존에 따른 git diff --check2는 별도 진단이며0으로 기록하지 않는다. MIN 네1/InvalidValue, F01 네1/TargetConflict, control 네0/HTTP200·target2다. 원 native4panic와 Git-normalization RED 및 준비 실패도 보존했다.

첫4371 M0의 env 누락으로 생긴 자기tracked9파일은 새root로 출력·SHA를 인수한 뒤 exact HEAD bytes만 복구했다. 첫 M1 시작4371/dirtytrue와 실행중5c helper/docs commit, default ignored map 출력 및 prior bytes 미인수 한계는 worklog와 custody에 기록했다. 그 실행은 clean exact source 증거로 쓰지 않는다. 새source5c M0/M1 whole은 모든8개 출력env를 새root에 지정했고 sourceaudit에서 self-owned tracked 출력변경0이다. 실제 변수/절대값은 `custody/browser-env-source5c.json`에 있다. 새 P05는 같은8키를 자기 새로운 prefix로 지정하고 기존 public typescript() stdout을 actualsource/lock/linkage의 ignored probe에서 비교해야 한다. tracked-writing generate 예제는 실행하지 않는다.

원 P05-2 제품 FAIL와 raw-index INVALID는 별도이며 원WT/before/index/ZIP를 복구하지 않았다. 원 P03/첫P06/두P05 자료는 보존한다. actual Linux/3OS/새HEAD source7CI, 독립P05-3/P07/기본브랜치CI·동일WP23/W2/C02는 부모 pending이다. 결정 필요·새게임규칙은 없다.
