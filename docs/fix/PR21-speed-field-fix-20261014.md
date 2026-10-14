# PR21 필수 지적 수정

- taskId: openhoi-pr21-fix-20261014-0340 / project: openhoi / stage: fix
- repository: peppone-choi/OpenHOI4
- 작업 기준: 원격 PR21 head 470e122d39fb1bf4e9d23fce7614b9954df01610 재확인.
- 필수 지적 1건: crates/oh_sim/src/lib.rs의 등록 입력과 저장 검증 속도 참조를 division.normal().stats.speed_kmh로 변경.
- 자료형 근거: military_templates::Normal.stats는 oh_data::military_templates::Stats이며 speed_kmh는 Fx. 변환/반올림/새 속도 계산 없음.
- 수정 범위: 두 필드 접근과 이 수정 근거 문서만. 기존 미커밋 리뷰 작업실과 기획 문서에 쓰지 않음.
- D1/D2 이동 허용 정책, 간선 보정, same-id, rollback, roster 동기화, codec 불변식 변경 없음. 실제 세계지도와 사단 이동 인수조건 유지.
- 정적 검증: git diff --check PASS, 기존 잘못된 필드 참조 0건, 수정 참조 2건.
- 빌드/회귀는 기존 GitHub-hosted Actions로 실행. 로컬 빌드/무거운 테스트 NOT_RUN.
- Windows/Android 제품 QA, 원본 지도 형상 비교, WebGPU 실제 플레이 QA NOT_RUN.
- 자체 승인/병합/GCP/배포/게임 실행 없음. 다음 담당 openhoi-review가 새 head 필수 지적 재검증 후 openhoi-verify에 인계.
