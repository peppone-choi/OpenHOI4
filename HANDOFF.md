# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, M0 독립 게이트 검증 파견 전 스냅샷 |
| 현재 마일스톤 | M0 — 모든 WP 통합됨, P-12 판정 대기 |
| 기본 브랜치 CI | 녹색: 마지막 제품 통합 f12b303의 [일반 CI 37447307839](https://github.com/peppone-choi/OpenHOI4/actions/runs/37447307839), [코어 CI 37447307865](https://github.com/peppone-choi/OpenHOI4/actions/runs/37447307865), [시뮬레이션 CI 37447307966](https://github.com/peppone-choi/OpenHOI4/actions/runs/37447307966) 모두 success. 이후 README·문서·증거만 변경했으며 제품 소스는 같다. |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

WP-05가 64789f5에 구현됐고 새 독립 검증에서 PASS를 받았다. 검증 전후 HEAD와 추적 파일 550개의 SHA256이 같았다. f12b303으로 통합한 뒤 모든 main CI를 직접 확인했다. M0 완료 선언은 새 P-12 게이트 판정까지 기다린다. docs/gates/M0.md는 판정 대기 초안이다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP-00 | CLI FAIL, 사용자 승인 앱 호출 시험 정상 종료 | CLI 시험 worktree·orch/selftest 삭제 | done 1 기록 보존 |
| WP-01 | 통합됨 | wp/01-skeleton; worktree 정리됨 | PASS·main CI 증거 보존 |
| WP-02 | 통합됨 | wp/02-core; worktree 정리됨 | PASS·코어 3 OS 증거 보존 |
| WP-03 | 통합됨 | wp/03-data; Git worktree 정리됨 | 제외된 검증 산출물 일부가 삭제 실패로 남아 있어 보존 |
| WP-04 | 통합됨 | wp/04-simulation; worktree 정리됨 | PASS·실제 1,000틱 3 OS 증거 보존 |
| WP-05 | 통합됨 | wp/05-network; .orchestrator/wt/WP-05와 WP-05-verify | 원본 보존·통합 확인 후 worktree 정리 |
| WP-06 | 통합됨 | main; worktree 정리됨 | 첫 FAIL과 수정 후 PASS 모두 보존 |
| M0 게이트 | 검증 대기 | gate/M0·M0-gate-verify 생성 예정 | 새 P-12 검증 → 판정 전문 반영 |

## 세션과 실행 경로

- tools/orch.sh status의 WP-00.impl은 done 1이다. HTTP 400 모델/ChatGPT 인증 미지원이며 CLI 작업 성공으로 기록하지 않는다. 모델·인증·샌드박스·승인·네트워크 설정은 변경하지 않았다.
- 사용자는 현재 앱 설정의 별도 채팅 실행을 선택했다. 앱 local 채팅에 WP별 Git worktree를 명시했고 모든 명령의 경로를 확인했다. 새 독립 검증의 HEAD/index/diff/status와 SHA256 확인을 보존했다. CLI 자동 exit 4를 앱 검증 성공 증거로 쓰지 않았다.
- WP-05 구현 01a11089-5718-7031-8d8a-d573f41de28e와 검증 01a110a2-e3b2-7e40-8965-9598532aeb90은 종료했다. 검증 대상은 64789f5이며 PASS다.
- WP-04 구현 01a11070-1ae3-7e83-9e0d-324570f7ecf9와 검증 01a11080-8a1e-7671-8663-d584c2f0d275도 종료했고 후자는 88d730a에 PASS다.
- WP-02·03·01·06의 채팅 ID·검증 대상·main CI는 .orchestrator/evidence/app-threads.json과 각 리포트에 기록했다. 실제 종료 상태를 확인했다.
- 첫 자동 worktree 요청 client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849의 실제 threadId는 미확인이다. C:/Users/user/.codex/worktrees/788a/openhoi detached checkout과 orch/app-selftest 브랜치는 보존했으며 제품 구현에 사용하지 않는다.
- 확인한 제품 구현·WP 검증 채팅은 모두 종료했다. 게이트 채팅은 이 스냅샷 이후 생성할 예정이며 wait_threads와 registry로 확인한다.

## 마지막 로컬 검증

| 명령 | 결과 |
|---|---|
| npm --prefix client ci / build, cargo test --workspace --locked, npm --prefix client test (WP-05 통합) | 모두 exit 0 |
| 실제 CLI 1,000틱 2회·최종 main 3 OS 아티팩트 비교 | 모두 ff921fd8148e699d |
| 최종 main 코어 3 OS 아티팩트 비교 | 모두 0dd81b8754bcc3f9, 게임 시뮬레이션 해시와 구분 |
| WP-05 독립 검사 | Rust 일반 테스트 46개·doc-test 2개, client 3개, 5개 브라우저 E2E 35개 통과; 타입 stale와 클라이언트 산출물 누락은 기대 실패 |
| 서버 수명주기 | 실제 HTTP·활성 WS·Ctrl+C·exit 0·포트 반환. 원격 main 세 OS에서도 통과 |
| python tools/check_docs.py --write-trace | exit 0, 오류 0·경고 0, M0 필수 REQ 14개 |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D-10 | 기존 기획 결정 | — | M0·허용 의존성·M1 기반 | 정식 제품 배포 라이선스 확정 |
| D-12 | 기존 기획 결정 | — | 저장소명 OpenHOI4·M0·M1 | 최종 제품 공개 명칭 확정 |

D-14는 사용자 지시로 PUBLIC 저장소로 확정했다. 01 §9에서 M1 착수를 차단하는 OPEN은 없다.

## 알려진 문제와 미확인 범위

| 문제 | 재현 방법·범위 |
|---|---|
| 원래 CLI 모델/인증 미지원 | docs/worklog/evidence/WP-06 원본; 사용자 승인 앱 대체 경로 사용 |
| 전체 CLI·게임 시스템 미구현 | M0는 run·빈 시나리오·시간 셸이다. validate/AI/bench/repro·경제/전투/지도·저장·멀티는 각 후속 WP |
| 브라우저 제품별 추가 확인 | 3 엔진과 Windows Chrome·Edge 확인. Safari 실제품과 Opera·Brave·Whale 미확인 |
| --open 검증 범위 | Windows 실제 launcher exit 0 확인. 시각적 브라우저 탐색 성공 판정은 하지 않음 |
| WP-03-verify 제외 파일 정리 실패 | Git worktree 등록은 제거됐으나 일부 target/tools 파일이 남았다. 제품/미병합 변경을 강제 삭제하지 않음 |

## 사용자 추가 지시

저장소는 PUBLIC https://github.com/peppone-choi/OpenHOI4 이며 한국어 설명과 게임 중심 README를 적용했다. 서버 3 OS·웹 주요 브라우저 지원 및 실행 편의를 요구사항/CI에 반영했다. GPU 지도는 WebGPU 우선·WebGL2 대체와 백엔드별 셰이더·장치 제한을 WP-08/M1에서 검증한다. 기술 결정 문서는 docs/adr에 있다. 최종 게임 개발 완료(M6 이후) 시 docs/, tools/, AGENTS.md를 정리하며 M0에는 운영·검증 자료를 유지한다.

## 다음 세션이 처음 할 일 (3개 이내)

1. 현재 증거 초안과 최신 main CI를 확인하고 gate/M0의 새 독립 P-12 검증 채팅을 생성·확인한다.
2. 판정 표 전문을 docs/gates/M0.md에 반영한다. FAIL이면 P-06/P-10으로 처리하고 미완료 이유를 기록한다.
3. 최종 HANDOFF를 현재 상태로 다시 쓰고 푸시 후 최신 main CI 녹색을 확인한다. M1은 새 요청에 따라 착수한다.