# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, WP-05 착수 |
| 현재 마일스톤 | M0 진행 중 |
| 기본 브랜치 CI | 녹색: WP-04 통합 f102232의 [일반 CI 37442600563](https://github.com/peppone-choi/OpenHOI4/actions/runs/37442600563), [코어 CI 37442600429](https://github.com/peppone-choi/OpenHOI4/actions/runs/37442600429), [실제 시뮬레이션 CI 37442600503](https://github.com/peppone-choi/OpenHOI4/actions/runs/37442600503) 모두 success. 이후 문서 커밋 49cc1ae의 CI는 별도 확인 중. |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

WP-04까지 독립 PASS 후 통합했고 main의 실제 1,000틱 세 OS 아티팩트를 직접 확인했다. WP-05를 전용 Git worktree의 앱 채팅에서 시작했다. WP-06 첫 검증의 HANDOFF 모순을 수정한 새 검증은 PASS다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP-00 | CLI FAIL, 앱 호출 시험 정상 종료 | CLI 시험 worktree·orch/selftest 삭제 | done 1 증거 보존 |
| WP-01 | 통합됨 | wp/01-skeleton; worktree 정리됨 | docs/verify/WP-01.md |
| WP-02 | 통합됨 | wp/02-core; worktree 정리됨 | 원본 로그·리포트·해시 증거 보존 |
| WP-03 | 통합됨 | wp/03-data; Git worktree 정리됨 | 검증 폴더의 제외 산출물 일부가 삭제 실패로 남음, 보존 |
| WP-06 | 통합됨 | main; worktree 정리됨 | PASS와 첫 FAIL 리포트 보존 |
| WP-04 | 통합됨 | wp/04-simulation; 구현·검증 worktree 정리됨 | docs/verify/WP-04.md 및 main-sim 증거 보존 |
| WP-05 | 구현 중 | wp/05-network; E:/openhoi/.orchestrator/wt/WP-05 | 종료 후 새 P-05 검증 → P-07 |

## 세션과 실행 경로

- `tools/orch.sh status`의 WP-00.impl은 done 1이다. HTTP 400: gpt-6.1-sol/ChatGPT 인증 미지원. CLI 작업 성공으로 기록하지 않는다. 모델·인증·샌드박스·승인·네트워크 설정은 바꾸지 않았다.
- 사용자는 현재 앱 설정의 별도 채팅을 선택했다. 앱 local 채팅 프롬프트에 WP별 Git worktree를 명시한다. 검증 전후 HEAD·추적 파일 목록/index/diff/status는 root가 직접 비교한다. CLI 자동 exit 4를 앱 검증 증거로 주장하지 않는다.
- WP-05 구현: 01a11089-5718-7031-8d8a-d573f41de28e, hostId local. `wait_threads`로 확인한다. WP-04 구현 01a11070-1ae3-7e83-9e0d-324570f7ecf9와 독립 검증 01a11080-8a1e-7671-8663-d584c2f0d275는 종료, 후자는 88d730a에 PASS다.
- WP-02 검증 PASS: 01a11062-63a4-7a92-bd08-8b939de7c124, eb15633. WP-03 검증 PASS: 01a11063-40fc-7753-8e33-c0b71768bfa3, 70de319. 각 추적 파일 무변경을 root가 확인했다.
- WP-06 첫 FAIL: 01a11052-0946-7cd0-9b02-166427582f0b, 88ef41f. 새 PASS: 01a11058-8222-7cc0-891b-ba78fddf0ff4, 1ad982b. 두 리포트 모두 docs/verify에 있다.
- 앱 호출 시험 01a11038-2afd-7dd3-9be4-753c94a02911은 정상 종료했다. 첫 자동 worktree 요청 client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849의 threadId는 미확인이다. C:/Users/user/.codex/worktrees/788a/openhoi detached 폴더와 orch/app-selftest 브랜치는 보존하며 구현에 사용하지 않는다.
- 최신 채팅 등록은 .orchestrator/evidence/app-threads.json이다. WP-05만 현재 실행 중이며 다른 구현·검증 채팅은 종료했다.

## 마지막 로컬 검증

| 명령 | 결과 |
|---|---|
| cargo test --workspace --locked (WP-04 통합) | exit 0; 독립 검증은 통합 테스트 41개와 compile-fail 2개 통과 |
| 실제 CLI 1,000틱 2회·main 세 OS 아티팩트 compare | exit 0; 모두 ff921fd8148e699d. 코어 진단은 별개로 0dd81b8754bcc3f9 |
| npm --prefix client test (WP-04 통합) | exit 0 |
| python tools/check_docs.py --write-trace | exit 0, 오류 0·경고 0; M0 필수 REQ 14개 |
| WP-02·03·04 검증 전후 HEAD·추적 파일 목록·index·diff/status | 동일; 독립 검증자 SHA-256 확인도 PASS |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D-10 | 기존 기획 결정 | — | M0, 허용 의존성 사용 | 제품 배포 라이선스 확정 |
| D-12 | 기존 기획 결정 | — | 저장소명 OpenHOI4와 M0 | 제품 공개 명칭 확정 |

M0를 막는 미결정 게임 규칙은 없다. D-14는 사용자 지시로 공개 저장소로 확정했다.

## 알려진 문제

| 문제 | 재현 방법·대응 |
|---|---|
| CLI 모델·인증 미지원 | docs/worklog/evidence/WP-06 로그. 사용자 승인 앱 경로로 진행 |
| 문서 검사기의 숫자 ADR 부분 일치 | 핵심 검사용 문서는 환경 실행 ADR이라고 참조하여 통과; 도구 약화 없음 |
| 전체 CLI 기능은 후속 WP | M0에는 run, AI 대전·전체 validate·재현은 각 후속 WP. 현재 구현 여부를 게이트에 명시 |

## 사용자 추가 지시

PUBLIC 저장소: https://github.com/peppone-choi/OpenHOI4 . 한국어 설명과 게임 중심 README를 유지한다. 서버는 Linux·Windows·macOS, 클라이언트는 세 주요 브라우저 엔진을 검사하고 제품별 미확인은 구분한다. REQ-NET-06/WP-05는 주소·사용법·--open·시작 실패·정상 종료를 검사한다. REQ-PLAT-03/WP-08/M1은 Three.js WebGPU 우선·WebGL2 대체, 가용 GPU 활용과 초기화 실패 안내를 구현한다. M0에 지도가 구현됐다고 주장하지 않는다. 최종 게임 개발 완료(M6 이후) 시 docs/, tools/, AGENTS.md를 정리하며 현재 M0 운영·검증 자료는 유지한다.

## 다음 세션이 처음 할 일 (3개 이내)

1. WP-05 실제 앱 상태·커밋을 확인하고 새 P-05 검증, 서버·프로토콜·브라우저 증거 수집 후 P-07로 통합한다.
2. 모든 WP의 최종 main CI·실제 세 OS 1,000틱 해시·UI 캡처를 모아 docs/gates/M0.md 초안을 커밋한다.
3. 새 P-12 검증 채팅으로 M0 게이트를 판정하고 최종 HANDOFF를 다시 쓴다. 미완료와 이유를 따로 기록한다.
