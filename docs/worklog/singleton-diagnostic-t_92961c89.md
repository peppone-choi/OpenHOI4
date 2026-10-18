# t_92961c89: D1 읽기전용 전체 진단

- 기준 PR28: a5f12c82c129414f9e4bf3aff41cd02c0edd7121, main: 092938b5aec9b50ad056205c8066987d420f1243.
- 별도 branch: diagnostic/t_92961c89. PR28 branch/ci.yml/exporter/기존 시험/제품 입력은 바꾸지 않는다.
- 실행: `.github/workflows/world-singleton-diagnostic.yml`, GitHub-hosted ubuntu-latest, 10분 제한, contents read, 생성 보고서만 artifact.
- 명령: `python -m unittest discover -s scripts -p test_diagnose_world_singletons.py -v`, 이어 `python scripts/diagnose_world_singletons.py --input client/public/preview/world --out <새 임시 출력 경로>`.
- 전체 지도 계산은 hosted에서만 실행한다. 로컬은 구문·diff 검사만 수행한다.

## 분석 계약

D1 원본 면적>=2, 직접 4방향(가로 wrap/세로 nonwrap), 같은 종류 우선, 공유 변 개수 내림차순, 작은 ID 순위를 유지한다. 기존 exporter의 hash/직렬화/이웃 helper만 재사용하며 absorb/export는 호출하지 않는다. 모든 1단계 결정을 원본 스냅샷에서 한꺼번에 내린다. 후보 없는 ID는 실패를 숨기지 않고 별도 목록에 남긴다. 이는 exporter의 성공 결과가 아니다.

분석용 2단계만 1단계 이후 스냅샷과 그 면적을 사용한다. 2단계 내에서도 동시 결정하고 chain/원거리 탐색은 하지 않는다. 원본 후보 없는 singleton의 이웃 변화를 별도로 기록한다. 2단계 정책은 승인되지 않았으며 제품에 적용하지 않는다.

JSON은 모든 singleton의 ID/kind/좌표, 방향별 이웃 ID/원본 면적/kind/좌표, 두 mapping, 종류 변화 수, 남는 ID/종류별 개수, 원본 2~8px 프로빈스 픽셀 ID 불변 여부를 포함한다. 불변은 원래 픽셀의 ID를 뜻하며, 흡수 대상 프로빈스의 면적 증가를 금지한다는 뜻은 아니다. 원본 입력 4파일의 SHA256/바이트 크기와 실행 HEAD를 기록한다. metadata는 자기인증되지 않으며 나머지 3파일 hash만 metadata 선언과 검증한다. 실행 후 입력 hash를 다시 비교한다.

합성시험은 후보 유무, 동률·종류·공유변 순위, 가로 wrap, 세로 nonwrap, chain 차단, 가상 2단계 이후 잔존, 입력 보존, hash 변조, zero-based index 복원, 출력 결정론, 역순 분석 독립성을 검사한다. 고정 seed의 작은 입력 100개를 별도 구현 oracle로 비교한다. Hosted에서는 알려진 4037 (2766,1889)의 이웃 13790/13791/13792/12153 및 전체 보고서 집계/실행 SHA를 대조한다.

## 결과와 인계

실제 run URL/SHA/artifact/측정값은 실행 후 프로젝트 evidence/t_92961c89/DIAGNOSTIC.md 및 results/t_92961c89-RESULT.json에 기록한다. 아직 이 문서 자체는 실행 성공을 주장하지 않는다. 다음 담당 openhoi-review의 독립 검토 전 정책 근거 승인 아님.

productComplete=false. Q1 국가·시대·소유 미정 유지. 제품 QA/M1/AC10/actual pack/gameplay/merge NOT_RUN. Windows/HOI4/GCP/운영/배포/초기화는 실행하지 않는다.
