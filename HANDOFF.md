# 인수인계

| 항목 | 값 |
|---|---|
| 작성 시각 | 2026-10-06, M0 독립 게이트 PASS 반영 후 |
| 현재 마일스톤 | M0 완료. M1 미착수 |
| 기본 브랜치 CI | 독립 검증 대상 d6ae2c8의 [일반 CI 37448564387](https://github.com/peppone-choi/OpenHOI4/actions/runs/37448564387), [코어 CI 37448564385](https://github.com/peppone-choi/OpenHOI4/actions/runs/37448564385), [시뮬레이션 CI 37448564362](https://github.com/peppone-choi/OpenHOI4/actions/runs/37448564362) 모두 녹색, 15개 job success. 이후 변경은 판정·증거·인수인계 문서뿐이며 제품 소스는 같다. 최신 문서 커밋의 CI는 최종 푸시 후 확인한다. |

## 실제 상태와 이전 인수인계의 차이 (재개 시 기록)

M0의 새 독립 P-12 검증이 AC-M0-01~07 전체와 필수 REQ 14개에 PASS를 냈다. 동일 HEAD main CI와 원격 아티팩트 ZIP 7개의 digest를 직접 확인했으며, 검증 전후 추적 파일 725개의 SHA256이 같았다. root도 HEAD·index·diff/status 무변경을 확인했다. 판정 전문과 증거는 [M0 게이트](docs/gates/M0.md)에 보존했고 구현·검증 worktree는 정리했다. 제품 최종 통합은 f12b303, 게이트 검증 대상은 d6ae2c8이다.

## WP 상태

| WP | 상태 | 브랜치·worktree | 다음 행동 |
|---|---|---|---|
| WP-00 | CLI FAIL, 사용자 승인 앱 시험 정상 종료 | orch/selftest·CLI 시험 worktree 삭제 | 원본 done 1 기록 보존 |
| WP-01 | 통합됨 | wp/01-skeleton; worktree 정리됨 | PASS·main CI 증거 보존 |
| WP-02 | 통합됨 | wp/02-core; worktree 정리됨 | PASS·코어 3 OS 증거 보존 |
| WP-03 | 통합됨 | wp/03-data; Git worktree 정리됨 | 제외된 잔여 파일은 아래 기록 |
| WP-04 | 통합됨 | wp/04-simulation; worktree 정리됨 | PASS·실제 1,000틱 3 OS 증거 보존 |
| WP-05 | 통합됨 | wp/05-network; worktree 정리됨 | PASS·서버/웹·원격 CI 증거 보존 |
| WP-06 | 통합됨 | main; worktree 정리됨 | 첫 FAIL과 수정 후 PASS 보존 |
| M0 게이트 | PASS | gate/M0=d6ae2c8; 검증 worktree 정리됨 | docs/gates/M0.md와 evidence/M0 보존 |

## 세션 상태와 실행 경로

- tools/orch.sh status: WP-00.impl done 1. 모델/ChatGPT 인증 미지원 HTTP 400으로 실패했다. 정상 종료로 바꾸어 쓰지 않았다.
- 사용자가 현재 앱 설정의 별도 채팅 실행을 선택해 WP별 Git worktree와 앱 local 채팅으로 구현·검증을 조율했다. 모델·인증·샌드박스·승인·네트워크 설정은 바꾸지 않았다. CLI 자동 exit 4를 앱 검증 증거로 쓰지 않았다.
- 확인한 제품 구현·WP 검증·게이트 채팅은 모두 종료했다. 게이트 검증은 01a110b5-dbfd-7a91-952a-c84fd41880c6, WP-05 검증은 01a110a2-e3b2-7e40-8965-9598532aeb90다.
- 전체 채팅 ID·대상·판정은 [보존 registry](docs/gates/evidence/M0/app-threads.json)와 .orchestrator/evidence/app-threads.json에 있다. 진행 중인 제품 작업은 없다.
- 첫 자동 worktree 요청 client-new-thread:c7dfdf29-3fd1-49fb-a43d-d3ac11104849의 실제 threadId는 미확인이다. C:/Users/user/.codex/worktrees/788a/openhoi detached checkout(63bccbe)과 orch/app-selftest 브랜치는 보존했고 제품 구현에 사용하지 않았다.

## 마지막 로컬 검증

| 명령·검사 | 결과 |
|---|---|
| npm ci/build, cargo fmt/clippy/workspace tests, client tests | 독립 게이트에서 exit 0. Rust 46개+compile-fail 2개, client 3개 |
| 실제 1,000틱 2회 및 세 OS 아티팩트 비교 | 모두 ff921fd8148e699d; 코어 진단은 별도 0dd81b8754bcc3f9 |
| 실제 서버 E2E | 로컬 3 엔진+Chrome·Edge 35개, Linux CI 3 엔진 21개 통과 |
| 실제 서버 수명주기 | Ctrl+C·활성 WS Close·exit 0·포트 반환 통과. 원격 서버 세 OS에서도 실행 |
| 라이선스·에셋·stale·빌드 누락 | 위반 픽스처 21개, 결정론 도구 19개 통과. 의도된 위반·낡은 타입·client/dist 누락은 실제 실패 |
| python tools/check_docs.py --write-trace | 오류 0·경고 0, M0 필수 REQ 14개 |
| 게이트 파일 불변성·동일 HEAD CI | 추적 파일 725개 동일, 15개 CI job success |

## 사용자 결정 대기

| 식별자 | 요청 파일 | 요청일 | 결정 없이 가능한 일 | 막힌 일 |
|---|---|---|---|---|
| D-10 | 기존 기획 결정 | — | M1·허용 의존성·자체 데이터 | 정식 제품 배포 라이선스 확정 |
| D-12 | 기존 기획 결정 | — | 저장소명 OpenHOI4·M1 | 최종 제품 공개 명칭 확정 |

D-14는 사용자 지시로 PUBLIC으로 확정했다. 01 §9에서 M1 착수를 막는 OPEN은 없다. OPEN-09는 M3, 다른 OPEN은 M4~M6에서 확인한다.

## 알려진 문제와 미완료 범위

| 항목 | 이유·범위 |
|---|---|
| 원래 CLI 모델/인증 미지원 | 사용자 승인 앱 대체 경로로 M0 진행. 원본은 docs/worklog/evidence/WP-06에 보존 |
| 후속 기능 미구현 | M0는 run·빈 시나리오·시간 셸. 전체 validate/AI/bench/repro·저장·경제·전투·지도·멀티는 후속 WP |
| 제품별 브라우저 검증 | 3 엔진과 Windows Chrome·Edge 확인. 실제 Safari·Opera·Brave·Whale 미확인 |
| --open 확인 범위 | Windows launcher exit 0 확인. 시각적 브라우저 탐색 성공 판정은 하지 않음 |
| WP-03-verify 잔여 제외 파일 | Git 등록은 제거됐으나 target/tools 일부가 삭제 실패로 남았다. 제품 변경·미병합 자료를 강제 삭제하지 않음 |
| 초기 앱 자동 worktree | 실제 채팅 ID 미확인, 위 폴더·브랜치 보존. 현재 제품 작업 없음 |

## 사용자 추가 지시

저장소는 PUBLIC https://github.com/peppone-choi/OpenHOI4 이며 한국어 설명·게임 중심 README를 적용했다. 서버 3 OS·브라우저 호환·실행 편의를 검증했다. GPU 지도는 Three.js WebGPU 우선·WebGL2 대체로 WP-08/M1에서 구현하며 가용 기능·초기화 실패·백엔드별 제한을 검사한다. OpenGL은 브라우저 WebGL/OpenGL ES 경로로 활용한다. M0에 GPU 지도가 구현됐다고 주장하지 않는다.

최종 게임 개발 완료(M6 이후) 시 docs/, tools/, AGENTS.md를 정리한다. 현재는 M0 운영·검증 증거를 유지하며 마지막 정리에서 Git 이력·CI 아티팩트와 필요한 빌드 도구 참조를 보존한다.

## 다음 세션이 처음 할 일 (3개 이내)

1. M0 게이트·최신 main CI·Git 상태를 확인한다. 판정표를 다시 추정하지 않는다.
2. M1 요청이 있으면 WP-07·WP-10·WP-12 묶음과 WP-08/09/11 후속 의존을 읽고 계획한다. 주요 GPU 렌더링 요구를 유지한다.
3. 정식 배포 전 D-10·D-12·OPEN-09를 확인하고 최종 개발 완료 시 정리 지시를 적용한다.