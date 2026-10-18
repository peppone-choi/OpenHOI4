# WP-32 M-1a 서버 팩 선택

- taskId: t_8df84cc9, project: openhoi, stage: implement
- repository: https://github.com/peppone-choi/OpenHOI4
- base: ba9cb0305c15afbb772659f147bf00a1ccbdf377
- branch: implement/t_8df84cc9
- draft PR: https://github.com/peppone-choi/OpenHOI4/pull/26

## 계약과 변경 범위

`--pack <pack-ID>`로 pack-root 아래 팩을 선택한다. 생략하면 testland와 기존 복사 fixture 동작을 유지한다. 명시한 ID는 oh_data::valid_id로 검증하고 manifest.id와 일치해야 한다. 시나리오 선택, 활성 팩 검증, 로딩, 지도 메타데이터, 해시, 저장 복원은 같은 선택 경로를 사용한다. wire/save/hash 형식과 게임 규칙은 바꾸지 않는다.

## 시험

- RED: a72b4ae4b335d260540cc7c5d0f8b51c43b99e4d. 기존 CI 첫 push의 Ubuntu job https://github.com/peppone-choi/OpenHOI4/actions/runs/39109004835/job/117620889096 에서 새 시험 2개가 `startup error: unknown option --pack`으로 실패했다. 기존 팩 시험 4개는 통과했다. fmt/clippy/build 통과 뒤 cargo test exit 101. 원문은 외부 evidence/t_8df84cc9/red-ubuntu.log에 보존한다.
- 인수 1: 기존 workspace 시험, 결정론 workflow 및 client E2E 회귀. 골든 수정 없음.
- 인수 2: 실제 실행 파일 2개에 Join·Snapshot 수신 후 기존 world Query의 WorldResult.map_id와 metadata pack_hash 비교. 실제 Snapshot은 TimeState만 포함하므로 기획의 "Snapshot world.map_id"를 그대로 접근할 수 없다. wire를 확장하지 않고 기존 조회 경로로 검증한다.
- 인수 3: 기존 movement_query.cjs 소스의 팩 인자만 주입해 같은 실제 wire assertion 전체를 재사용. N01 job 0 Deploy 1, Move 2와 Stop/도착/거부 포함. 저장 fixture 내용 불변 확인.
- 인수 4: 잘못된 ID 2종, 없는 팩, 중복 옵션, 없는 시나리오, manifest 불일치, 값 누락. 점유 포트로 실행해 startup error가 bind 시도보다 앞서는지 확인.
- 인수 5: A(m1) 저장을 B(m2_military)에서 load-save할 때 기존 `MilitaryModeMismatch: save/local definitions` 거부와 원본 저장 불변 확인.
- M0 원본 manifest.id는 m0_testland이고 디렉터리는 testland다. 생략 시 기존 hash를 보존하며, 명시적 --pack testland는 ID 불일치로 거부하는 것이 계약이다. M1 testland는 명시/생략 hash 동등성을 검사한다.
- 로컬: cargo fmt --check 시도는 cargo 미설치로 exit 127. 로컬 빌드/서버/브라우저/시험 NOT_RUN. 실제 빌드·시험은 기존 GitHub-hosted CI만 사용한다.

## 범위 밖과 미실행

M-1 본체·실제 세계 팩 생성·국가/소유 레이어·AC10·독립 리뷰·제품 QA·병합 NOT_RUN. Windows PC/HOI4/GCP/배포/초기화/유료 구매 없음. Q1 답은 이번 M-1a의 선행조건이 아니다.

최종 exact commit, hosted run URL·결과·오류 원문은 프로젝트 evidence/t_8df84cc9 및 results/t_8df84cc9-RESULT.json에 남긴다. 다음 담당 openhoi-review.
