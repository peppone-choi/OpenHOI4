# M1-r3 독립 P-12 원본과 부모 보관 검사

검증 대상은 main `89964f4400c1affbfc484a60f486af51788c7298`, 판정은 AC 7 PASS/0 FAIL/사용자 보류0·필수 REQ14 PASS다. [게이트 전문](../../M1.md)은 실제 앱 최종 메시지 원문 그대로다. 검증 앱 `01a113b4-4bc0-7740-ba9e-952792592294`는 idle/completed11이며 앱 실행을 CLI exit0으로 기록하지 않았다.

[원본 ZIP](evidence.zip)은 15,439,305bytes·1,568members·SHA256 `453e414e525feeb40520fd49c6214189e2947bf01b0c63b11e174509ecb7c916`이다. [manifest](evidence-manifest.json)의 1,567 source 파일 bytes/SHA와 모든 archive member를 부모가 전수 비교해 불일치0을 확인했다. 실제 GUI18캡처·행동/기대/실제·own native 정상종료, 정확 HEAD 네CI의 API/로그/artifact·독립 PNG 재계산, 정책 환경 FAIL 두 건과 원 테스트의 저장소 밖 process TMP 1회 PASS, 준비 오류와 이전 봉인 기록을 모두 포함한다.

[부모 원본 ZIP](parent-originals.zip)은 최초/최종 snapshot JSON·raw index 원문, 실제 앱 API 최종 메시지, 정확한 전달 프롬프트와 read_thread 인자 오류를 보존한다. [부모 보관 검사](parent-custody.json)는 최초9,722개 추적 파일·HEAD·semantic/raw index·목록·각 SHA·diff/cached/status의 최종 동일성, JSON/index bytes 일치 및 게이트 전문 동일성을 기록한다. 원 raw SHA256은 `6077990c30eac37851cab7b8c4bad07a8cb4c12df30069a137755fdc23b76d92`다. 원 baseline을 교체하거나 index를 복구하지 않았다. 부모 보관 스크립트의 최초 suffix 경로 오류는 검사만 실패했고 올바른 원 경로로 전수 검사를 수행했다.

원 경로 `E:/openhoi/.orchestrator/wt/M1-M1-r3-P12/target/m1-r3-gate`를 유지한다. ZIP member 이름은 이 경로 기준 상대 경로다. 링크가 없는 다른 환경에서도 원문을 추출해 열 수 있다. 게이트 본문의 Windows P0560/205는 제품248blob 동일성을 입증한 새 독립P05 원 전체 결과이고, P12의 GUI와 직접 명령은 별도 증거다.

[이전 M1-r2 FAIL](../M1-r2/original-gate-report.md)과 [원 bytes 보관](../M1-r2/original-gate-report-identity.json), [이번 eb87 정상 CI FAIL](../../../verify/evidence/WP-08-main-eb87-CI-FAIL/README.md)을 유지한다. 현재 PASS를 과거 Edge 초기 화면 원인이나 모든 DPR 간헐 원인 해결로 확대하지 않는다. P08 문서 커밋의 최종 HEAD·CI 관측은 부모의 RUN_RESULT와 CEO 기록으로 별도 확인한다.
